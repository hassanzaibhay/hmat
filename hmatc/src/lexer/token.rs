//! HMAT token definitions.
//!
//! The [`Token`] enum enumerates every terminal the HMAT lexer can produce.
//! Tokens are matched by the [`logos`] crate with the exception of
//! [`Token::Indent`] and [`Token::Dedent`], which are synthesised by the
//! post-processing pass in [`crate::lexer::tokenize`] based on significant
//! whitespace at the beginning of logical lines.
//!
//! The set of tokens here corresponds to spec v0.3 §2 (Lexical Rules).
//! See `spec/0.2/grammar.md` (the v0.3 spec lives at the v0.2 path).

use logos::Logos;

/// A single HMAT token.
///
/// Literal values are carried in the variant payload so that later compiler
/// stages can consume them without re-lexing. Keywords and punctuation are
/// carried as unit variants.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\f]+")]
#[logos(skip r"#[^\n]*")]
pub enum Token {
    // ===== Numeric literals =====
    /// Decimal integer. Supports `_` separators (e.g. `1_000_000`).
    #[regex(r"[0-9][0-9_]*", parse_dec_int)]
    /// Hexadecimal integer. `0x[0-9a-fA-F_]+`.
    #[regex(r"0x[0-9a-fA-F_]+", parse_hex_int)]
    /// Binary integer. `0b[01_]+`.
    #[regex(r"0b[01_]+", parse_bin_int)]
    /// Octal integer. `0o[0-7_]+`.
    #[regex(r"0o[0-7_]+", parse_oct_int)]
    IntLiteral(i64),

    /// Floating-point literal. Requires at least one digit before and after
    /// the decimal point so that `3.max()` tokenizes as `IntLiteral(3) Dot
    /// Identifier` rather than an invalid float.
    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*(?:[eE][+-]?[0-9]+)?", parse_float)]
    FloatLiteral(f64),

    // ===== String literals =====
    /// Double-quoted string with escape sequences processed.
    #[regex(r#""([^"\\\n]|\\.)*""#, parse_string_lit)]
    StringLiteral(String),

    /// Formatted string literal (`f"..."`). The interior is captured verbatim
    /// (minus the `f"` prefix, trailing `"`, and processed escapes); the
    /// parser is responsible for splitting text and `{expression}` segments.
    #[regex(r#"f"([^"\\\n]|\\.)*""#, parse_fstring_lit)]
    FString(String),

    // ===== Boolean / nil =====
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("nil")]
    Nil,

    // ===== Keywords =====
    #[token("fn")]
    Fn,
    #[token("let")]
    Let,
    #[token("mut")]
    Mut,
    #[token("return")]
    Return,
    #[token("if")]
    If,
    #[token("elif")]
    Elif,
    #[token("else")]
    Else,
    #[token("on")]
    On,
    #[token("shape")]
    Shape,
    #[token("type")]
    Type,
    #[token("flow")]
    Flow,
    #[token("fail")]
    Fail,
    #[token("pub")]
    Pub,
    #[token("async")]
    Async,
    #[token("await")]
    Await,
    #[token("ai")]
    Ai,
    #[token("model")]
    Model,
    #[token("unsafe")]
    Unsafe,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("while")]
    While,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("import")]
    Import,
    #[token("from")]
    From,
    #[token("as")]
    As,
    #[token("self")]
    SelfKw,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("not")]
    Not,
    #[token("is")]
    Is,

    // ===== Identifiers (must follow keywords; logos prefers literals) =====
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // ===== Operators =====
    // Arithmetic
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("^")]
    Caret,

    // Comparison
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token("<")]
    Lt,
    #[token("<=")]
    LtEq,
    #[token(">")]
    Gt,
    #[token(">=")]
    GtEq,

    // Assignment
    #[token("=")]
    Eq,
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("*=")]
    StarEq,
    #[token("/=")]
    SlashEq,
    #[token("%=")]
    PercentEq,

    // Arrows
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,

    // Borrow / bitwise / logical
    #[token("&")]
    Ampersand,
    #[token("|")]
    Pipe,
    #[token("!")]
    Bang,
    #[token("~")]
    Tilde,

    // Decorators
    #[token("@")]
    At,

    // ===== Delimiters =====
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    // Punctuation
    #[token("::")]
    ColonColon,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("..=")]
    DotDotEq,
    #[token("..")]
    DotDot,
    #[token(".")]
    Dot,

    // ===== Significant whitespace =====
    /// Logical end-of-statement. Emitted for the first physical newline after
    /// a content token; collapsed across blank and comment-only lines.
    #[token("\n")]
    Newline,

    /// Injected by the indent post-processor when the indentation level
    /// increases. Carries a zero-width span at the start of the indented line.
    Indent,

    /// Injected by the indent post-processor when the indentation level
    /// decreases. One `Dedent` is emitted per level closed; trailing dedents
    /// are also emitted at end-of-file for any still-open indent levels.
    Dedent,
}

// ===== Logos callbacks =====

fn parse_dec_int(lex: &mut logos::Lexer<Token>) -> Option<i64> {
    strip_underscores(lex.slice()).parse().ok()
}

fn parse_hex_int(lex: &mut logos::Lexer<Token>) -> Option<i64> {
    let body = strip_underscores(&lex.slice()[2..]);
    i64::from_str_radix(&body, 16).ok()
}

fn parse_bin_int(lex: &mut logos::Lexer<Token>) -> Option<i64> {
    let body = strip_underscores(&lex.slice()[2..]);
    i64::from_str_radix(&body, 2).ok()
}

fn parse_oct_int(lex: &mut logos::Lexer<Token>) -> Option<i64> {
    let body = strip_underscores(&lex.slice()[2..]);
    i64::from_str_radix(&body, 8).ok()
}

fn parse_float(lex: &mut logos::Lexer<Token>) -> Option<f64> {
    strip_underscores(lex.slice()).parse().ok()
}

fn parse_string_lit(lex: &mut logos::Lexer<Token>) -> Option<String> {
    let s = lex.slice();
    // Strip surrounding quotes; logos only matched when both exist.
    process_escapes(&s[1..s.len() - 1])
}

fn parse_fstring_lit(lex: &mut logos::Lexer<Token>) -> Option<String> {
    let s = lex.slice();
    // Strip leading `f"` and trailing `"`.
    process_escapes(&s[2..s.len() - 1])
}

fn strip_underscores(s: &str) -> String {
    s.chars().filter(|c| *c != '_').collect()
}

/// Processes HMAT string escape sequences per spec §2.5.
///
/// Recognised escapes: `\n \t \r \\ \" \' \0 \xHH`.
/// Returns `None` if the escape is malformed (unterminated or unknown).
fn process_escapes(s: &str) -> Option<String> {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next()? {
            'n' => out.push('\n'),
            't' => out.push('\t'),
            'r' => out.push('\r'),
            '\\' => out.push('\\'),
            '"' => out.push('"'),
            '\'' => out.push('\''),
            '0' => out.push('\0'),
            'x' => {
                let h1 = chars.next()?;
                let h2 = chars.next()?;
                let byte = u8::from_str_radix(&format!("{h1}{h2}"), 16).ok()?;
                out.push(byte as char);
            }
            _ => return None,
        }
    }
    Some(out)
}

impl Token {
    /// Returns a short, human-readable label for error messages.
    /// Keywords and punctuation render as their source form; literals as a
    /// category name.
    pub fn label(&self) -> &'static str {
        match self {
            Token::IntLiteral(_) => "integer literal",
            Token::FloatLiteral(_) => "float literal",
            Token::StringLiteral(_) => "string literal",
            Token::FString(_) => "f-string literal",
            Token::True => "`true`",
            Token::False => "`false`",
            Token::Nil => "`nil`",
            Token::Fn => "`fn`",
            Token::Let => "`let`",
            Token::Mut => "`mut`",
            Token::Return => "`return`",
            Token::If => "`if`",
            Token::Elif => "`elif`",
            Token::Else => "`else`",
            Token::On => "`on`",
            Token::Shape => "`shape`",
            Token::Type => "`type`",
            Token::Flow => "`flow`",
            Token::Fail => "`fail`",
            Token::Pub => "`pub`",
            Token::Async => "`async`",
            Token::Await => "`await`",
            Token::Ai => "`ai`",
            Token::Model => "`model`",
            Token::Unsafe => "`unsafe`",
            Token::For => "`for`",
            Token::In => "`in`",
            Token::While => "`while`",
            Token::Break => "`break`",
            Token::Continue => "`continue`",
            Token::Import => "`import`",
            Token::From => "`from`",
            Token::As => "`as`",
            Token::SelfKw => "`self`",
            Token::And => "`and`",
            Token::Or => "`or`",
            Token::Not => "`not`",
            Token::Is => "`is`",
            Token::Identifier(_) => "identifier",
            Token::Plus => "`+`",
            Token::Minus => "`-`",
            Token::Star => "`*`",
            Token::Slash => "`/`",
            Token::Percent => "`%`",
            Token::Caret => "`^`",
            Token::EqEq => "`==`",
            Token::NotEq => "`!=`",
            Token::Lt => "`<`",
            Token::LtEq => "`<=`",
            Token::Gt => "`>`",
            Token::GtEq => "`>=`",
            Token::Eq => "`=`",
            Token::PlusEq => "`+=`",
            Token::MinusEq => "`-=`",
            Token::StarEq => "`*=`",
            Token::SlashEq => "`/=`",
            Token::PercentEq => "`%=`",
            Token::Arrow => "`->`",
            Token::FatArrow => "`=>`",
            Token::Ampersand => "`&`",
            Token::Pipe => "`|`",
            Token::Bang => "`!`",
            Token::Tilde => "`~`",
            Token::At => "`@`",
            Token::LParen => "`(`",
            Token::RParen => "`)`",
            Token::LBrace => "`{`",
            Token::RBrace => "`}`",
            Token::LBracket => "`[`",
            Token::RBracket => "`]`",
            Token::ColonColon => "`::`",
            Token::Colon => "`:`",
            Token::Comma => "`,`",
            Token::DotDotEq => "`..=`",
            Token::DotDot => "`..`",
            Token::Dot => "`.`",
            Token::Newline => "newline",
            Token::Indent => "indent",
            Token::Dedent => "dedent",
        }
    }
}
