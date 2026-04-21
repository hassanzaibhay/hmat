//! Shared compiler-wide error types.
//!
//! `CompilerError` is the single error type returned from driver-level
//! operations (read source, lex, eventually parse/check/codegen). Each phase
//! has its own finer-grained error enum (e.g. [`crate::lexer::LexError`])
//! that is wrapped here via `From`.

use std::io;
use thiserror::Error;

use crate::lexer::LexError;
use crate::parser::ParseError;
use crate::semantic::TypeError;

/// Top-level error type for the `hmatc` driver.
#[derive(Debug, Error)]
pub enum CompilerError {
    /// I/O failure reading or writing a source or output file.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Lexing failed.
    #[error("lex error: {0}")]
    Lex(#[from] LexError),

    /// Parsing failed.
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),

    /// Type checking found one or more errors. Each is surfaced with its
    /// code and help string by the driver before this top-level error is
    /// returned; this variant carries the count so `main` can exit with
    /// a sensible message.
    #[error("type check failed with {0} error(s)")]
    Type(usize),
}

impl From<Vec<TypeError>> for CompilerError {
    fn from(errors: Vec<TypeError>) -> Self {
        CompilerError::Type(errors.len())
    }
}
