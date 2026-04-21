//! HMAT Semantic Analysis.
//!
//! Runs after the parser has produced an [`crate::ast::Program`]. Each sub-pass
//! inspects the tree and either succeeds (the tree is well-formed for that
//! concern) or returns a typed error.
//!
//! Phase 0 ships the [`types`] pass — type inference and type checking for
//! the primitive subset of HMAT. Later phases will add:
//!
//! - `ownership` — borrow / move / region checking (Phase 2).
//! - `ai_validator` — `ai model` / `flow` construct validation (Phase 3).
//!
//! The passes are independent and composable; the driver runs them in order
//! and stops at the first error.

pub mod types;

pub use types::{check, Type, TypeError};
