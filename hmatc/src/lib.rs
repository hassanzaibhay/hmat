//! The HMAT compiler (`hmatc`) as a library.
//!
//! This crate is consumed by the `hmatc` binary and by integration tests.
//! Phase 0 exposes the [`lexer`] module; later phases will add [`parser`],
//! [`ast`], semantic analysis, and codegen.
//!
//! # Example
//! ```
//! let tokens = hmatc::lexer::tokenize("let x = 1\n").unwrap();
//! assert!(!tokens.is_empty());
//! ```

pub mod ast;
pub mod codegen;
pub mod driver;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod semantic;

pub use error::CompilerError;
