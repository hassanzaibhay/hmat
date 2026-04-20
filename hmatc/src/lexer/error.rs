//! Lexer error types.
//!
//! All lexer failures carry a byte [`Span`] into the original source so that
//! later diagnostic rendering can reproduce the exact offending snippet. The
//! compiler's error formatter (not implemented in this module) is responsible
//! for mapping spans to line/column positions and rendering the caret.

use std::ops::Range;
use thiserror::Error;

/// Byte span into the source string (half-open, UTF-8 byte offsets).
pub type Span = Range<usize>;

/// Errors produced by the HMAT lexer.
///
/// These map to spec error codes in the `E001–E099` range reserved for
/// syntax-level diagnostics (see `spec/0.2/grammar.md` §9).
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum LexError {
    /// Encountered a byte sequence that does not start any known token.
    /// Emitted with error code `E001`.
    #[error("E001: unknown token `{snippet}` at bytes {}..{}", .span.start, .span.end)]
    InvalidToken { span: Span, snippet: String },

    /// A numeric or string literal matched the opening pattern but failed to
    /// convert — overflow, malformed escape, unterminated string, etc.
    /// Emitted with error code `E002`.
    #[error(
        "E002: malformed {kind} literal `{snippet}` at bytes {}..{}",
        .span.start,
        .span.end
    )]
    MalformedLiteral {
        span: Span,
        kind: &'static str,
        snippet: String,
    },

    /// Indentation decreased to a level that was never opened.
    /// Emitted with error code `E003`.
    #[error(
        "E003: inconsistent indentation at byte {} — expected column {expected}, found column {found}",
        .span.start
    )]
    IndentMismatch {
        span: Span,
        expected: usize,
        found: usize,
    },
}

impl LexError {
    /// Returns the span this error points to, for diagnostic rendering.
    pub fn span(&self) -> &Span {
        match self {
            LexError::InvalidToken { span, .. } => span,
            LexError::MalformedLiteral { span, .. } => span,
            LexError::IndentMismatch { span, .. } => span,
        }
    }

    /// Returns the four-character diagnostic code (e.g. `E001`).
    pub fn code(&self) -> &'static str {
        match self {
            LexError::InvalidToken { .. } => "E001",
            LexError::MalformedLiteral { .. } => "E002",
            LexError::IndentMismatch { .. } => "E003",
        }
    }

    /// Returns an actionable fix hint, matching the Dev Agent error philosophy
    /// of "every error suggests what to do next".
    pub fn help(&self) -> &'static str {
        match self {
            LexError::InvalidToken { .. } => {
                "this character is not part of HMAT syntax — remove it or check for a typo"
            }
            LexError::MalformedLiteral { kind, .. } => match *kind {
                "integer" => {
                    "check for overflow (HMAT `int` is 64-bit) or invalid digits for the base"
                }
                "float" => "floats need digits on both sides of the dot, e.g. `3.14` not `3.`",
                "string" => {
                    "check escape sequences — valid escapes are \\n \\t \\r \\\\ \\\" \\' \\0 \\xHH"
                }
                _ => "check the literal format against spec §2.5",
            },
            LexError::IndentMismatch { .. } => {
                "align this line to match a previous indentation level (use 4 spaces per level)"
            }
        }
    }
}
