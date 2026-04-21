//! HMAT code generation.
//!
//! # Phase 1 backend — C via clang
//!
//! The long-term plan is LLVM IR via `inkwell`, but LLVM 22 on Windows
//! ships without `llvm-config.exe`, which `inkwell` needs to discover
//! the host toolchain. Rather than block the language on an upstream
//! fix, Phase 1 takes the same pragmatic path early Rust and Haxe took:
//! emit portable C source and hand it to `clang`. We swap in the native
//! LLVM IR backend as soon as `inkwell` supports LLVM 18+.
//!
//! The entry point [`emit_c`] consumes a type-checked [`Program`] and
//! returns the C source text. The driver is responsible for writing it
//! to disk and shelling out to the C compiler.

pub mod c;

use crate::ast::Program;

/// Emits C source text for a type-checked HMAT [`Program`].
///
/// The returned string is self-contained — it carries the `#include`
/// directives it needs and compiles with any C11 `clang`. It does not
/// depend on any HMAT runtime library (Phase 1 is runtime-free).
pub fn emit_c(program: &Program) -> String {
    c::emit(program)
}
