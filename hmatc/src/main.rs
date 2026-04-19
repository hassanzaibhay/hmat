//! `hmatc` — the HMAT compiler driver.
//!
//! Phase 0 responsibilities: read a source file (or stdin), run the lexer,
//! and emit the requested intermediate representation. Later phases will
//! plug parser, semantic analysis, and codegen into the same dispatch.

use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use hmatc::lexer;
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
}

/// What to dump to stdout when `--emit` is set.
#[derive(ValueEnum, Clone, Debug)]
enum EmitKind {
    /// The token stream (lexer output).
    Tokens,
    /// The abstract syntax tree (parser output). Not yet implemented.
    Ast,
    /// The high-level IR. Not yet implemented.
    Hir,
    /// LLVM IR (codegen output). Not yet implemented.
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
        Some(EmitKind::Ast) => {
            eprintln!("error: --emit=ast is not implemented yet (Phase 0)");
            Ok(())
        }
        Some(EmitKind::Hir) => {
            eprintln!("error: --emit=hir is not implemented yet (Phase 0)");
            Ok(())
        }
        Some(EmitKind::LlvmIr) => {
            eprintln!("error: --emit=llvm-ir is not implemented yet (Phase 0)");
            Ok(())
        }
        None => {
            // No emit flag and no codegen yet — default behaviour for Phase 0
            // is to report that full compilation isn't wired up.
            eprintln!(
                "hmatc: full compilation is not implemented yet. \
                 Use `--emit=tokens` to inspect lexer output."
            );
            Ok(())
        }
    }
}

/// Reads source from a file path, or from stdin when `None` is given.
fn read_source(path: Option<&std::path::Path>) -> Result<String, CompilerError> {
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
