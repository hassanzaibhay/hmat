//! `hmatc` — the HMAT compiler driver.
//!
//! Phase 1 responsibilities: read a source file (or stdin), run the
//! lexer + parser + type checker, then either dump the requested
//! intermediate representation or lower to C and invoke `clang` to
//! produce a native binary.

use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use clap::{Parser, ValueEnum};
use hmatc::codegen;
use hmatc::driver;
use hmatc::lexer;
use hmatc::parser;
use hmatc::semantic;
use hmatc::CompilerError;

/// The `hmatc` command-line interface.
#[derive(Parser, Debug)]
#[command(
    name = "hmatc",
    version,
    about = "The HMAT programming language compiler",
    long_about = "Compiles .hm source files. Pass --emit to dump \
                  intermediate representations instead of generating a binary."
)]
struct Cli {
    /// Source file to compile. Reads from stdin when omitted.
    file: Option<PathBuf>,

    /// Emit an intermediate representation and exit (instead of full compile).
    #[arg(long, value_enum)]
    emit: Option<EmitKind>,

    /// Override the output binary path. Default: `<stem>.exe` (or `<stem>`
    /// on non-Windows hosts), placed in the current working directory.
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

/// What to dump to stdout when `--emit` is set.
#[derive(ValueEnum, Clone, Debug)]
enum EmitKind {
    /// The token stream (lexer output).
    Tokens,
    /// The abstract syntax tree (parser output).
    Ast,
    /// The high-level IR. Not yet implemented.
    Hir,
    /// Generated C source (Phase 1 backend).
    C,
    /// LLVM IR. Blocked on `inkwell` supporting LLVM 18+.
    LlvmIr,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), CompilerError> {
    let source = read_source(cli.file.as_deref())?;

    match cli.emit {
        Some(EmitKind::Tokens) => emit_tokens(&source),
        Some(EmitKind::Ast) => emit_ast(&source),
        Some(EmitKind::Hir) => {
            eprintln!("error: --emit=hir is not implemented yet (Phase 2)");
            Ok(())
        }
        Some(EmitKind::C) => emit_c(&source),
        Some(EmitKind::LlvmIr) => {
            println!("LLVM IR: switch inkwell to llvm18-0 feature when inkwell supports LLVM 18+");
            Ok(())
        }
        None => compile(&source, cli.file.as_deref(), cli.output.as_deref()),
    }
}

/// Reads source from a file path, or from stdin when `None` is given.
fn read_source(path: Option<&Path>) -> Result<String, CompilerError> {
    match path {
        Some(p) => Ok(std::fs::read_to_string(p)?),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

/// Runs the lexer and prints each token in a fixed, greppable format:
/// `Token[<index>]: <variant>(<payload>)? @ <start>..<end>`.
fn emit_tokens(source: &str) -> Result<(), CompilerError> {
    let tokens = lexer::tokenize(source)?;
    for (i, (tok, span)) in tokens.iter().enumerate() {
        let location = format_span(tok, span);
        println!("Token[{i}]: {:<24} @ {location}", format_token(tok));
    }
    Ok(())
}

fn format_token(tok: &lexer::Token) -> String {
    use lexer::Token::*;
    match tok {
        IntLiteral(n) => format!("IntLiteral({n})"),
        FloatLiteral(f) => format!("FloatLiteral({f})"),
        StringLiteral(s) => format!("StringLiteral({s:?})"),
        FString(s) => format!("FString({s:?})"),
        Identifier(s) => format!("Identifier({s})"),
        other => format!("{other:?}"),
    }
}

fn format_span(tok: &lexer::Token, span: &lexer::Span) -> String {
    use lexer::Token::*;
    match tok {
        Indent | Dedent => "(injected)".to_string(),
        _ => format!("{}..{}", span.start, span.end),
    }
}

/// Runs the lexer + parser + type checker and prints the AST as an
/// indented tree. Any type errors are reported to stderr before the
/// driver returns a failure exit code.
fn emit_ast(source: &str) -> Result<(), CompilerError> {
    let program = front_end(source)?;
    print!("{}", program.pretty_print());
    Ok(())
}

/// Runs the full front-end and prints the generated C source to stdout.
fn emit_c(source: &str) -> Result<(), CompilerError> {
    let program = front_end(source)?;
    print!("{}", codegen::emit_c(&program));
    Ok(())
}

/// Lexes, parses, and type-checks a source buffer. Type-checker errors
/// are printed with their code + help hint before the error bubbles up.
fn front_end(source: &str) -> Result<hmatc::ast::Program, CompilerError> {
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(tokens)?;
    if let Err(errors) = semantic::check(&program) {
        for e in &errors {
            eprintln!("error[{}]: {}", e.code(), e);
            eprintln!("   = help: {}", e.help());
        }
        return Err(CompilerError::Type(errors.len()));
    }
    Ok(program)
}

/// Full-pipeline compile: source → C → native binary via `clang`.
fn compile(
    source: &str,
    input_path: Option<&Path>,
    output_override: Option<&Path>,
) -> Result<(), CompilerError> {
    let program = front_end(source)?;
    let c_source = codegen::emit_c(&program);

    let stem = input_path
        .and_then(|p| p.file_stem())
        .and_then(|s| s.to_str())
        .unwrap_or("a");

    // Absolute paths only — see driver.rs for the argument-smuggling
    // threat model this defends against.
    let cwd = std::env::current_dir()?;
    let (c_path, exe_path) =
        driver::absolute_output_paths(&cwd, stem, driver::exe_suffix(), output_override)?;

    std::fs::write(&c_path, &c_source)?;

    // Resolve clang to an absolute PathBuf before invoking Command::new —
    // never hand a bare "clang" to the OS loader (CWE-427).
    let clang = driver::find_clang()?;
    let status = Command::new(&clang)
        .arg(&c_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("-O2")
        .arg("-std=c11")
        .arg("-Wno-parentheses-equality")
        .status()?;

    if !status.success() {
        // Keep the .c file around on failure so the user can inspect it.
        return Err(CompilerError::ClangFailed(status.code().unwrap_or(-1)));
    }

    // Remove the intermediate on success — the binary is the product.
    // Best-effort; leaving it on disk is harmless.
    let _ = std::fs::remove_file(&c_path);

    eprintln!("hmatc: wrote {}", exe_path.display());
    Ok(())
}
