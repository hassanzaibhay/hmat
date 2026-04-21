//! Driver-level helpers for the `hmatc` binary.
//!
//! These helpers live in the library crate so integration tests can exercise
//! them directly. Everything here deals with *process boundaries* — turning
//! untrusted bytes (PATH entries, filenames, CLI args) into paths we are
//! willing to hand to `clang`.
//!
//! # Security posture
//!
//! - [`find_clang`] never calls `Command::new("clang")` with a bare name.
//!   A bare name forces Rust's `CreateProcessW` / `execvp` fallback to walk
//!   PATH itself, and on Windows that walk *includes the current working
//!   directory* — a classic CWE-427 (Untrusted Search Path) foothold.
//!   Instead we walk PATH manually, skip CWD explicitly, require absolute
//!   entries, and return an absolute `PathBuf`.
//! - [`validate_output_name`] refuses filenames that begin with `-`. Without
//!   this, `hmatc -- -oevil.exe.hm` would generate `-oevil.exe.c` and feed
//!   it to `clang`, which would read it as the `-o` flag. Reject early.
//! - [`absolute_output_paths`] always prefixes the emitted `.c` and the
//!   output binary with an absolute directory so the final `clang` command
//!   cannot be influenced by whatever directory we happen to be in.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::CompilerError;

/// Standard LLVM-on-Windows install location, used as a fallback when PATH
/// lookup fails. Phase 1 documents this path explicitly in CLAUDE.md.
const WINDOWS_CLANG_FALLBACK: &str = r"C:\Program Files\LLVM\bin\clang.exe";

/// Locates `clang` and returns an absolute path to it.
///
/// Lookup order:
///   1. walk `$PATH` manually (see [`find_on_path`]),
///   2. the documented Windows fallback (`C:\Program Files\LLVM\bin\clang.exe`).
///
/// Returns [`CompilerError::ClangNotFound`] when neither succeeds. The
/// returned path is always absolute — callers must hand it to
/// `Command::new` as-is and must not re-resolve it.
pub fn find_clang() -> Result<PathBuf, CompilerError> {
    if let Some(p) = find_on_path("clang") {
        return Ok(p);
    }
    let fallback = PathBuf::from(WINDOWS_CLANG_FALLBACK);
    if fallback.is_absolute() && fallback.is_file() {
        return Ok(fallback);
    }
    Err(CompilerError::ClangNotFound)
}

/// Walks the process `$PATH` looking for an executable named `name`,
/// returning an absolute `PathBuf` on success.
///
/// Convenience wrapper around [`find_on_path_in`] that reads `$PATH` and
/// `current_dir()` from the process. Tests should call `find_on_path_in`
/// directly so they can inject values instead of mutating global state.
pub fn find_on_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let cwd = std::env::current_dir().ok();
    find_on_path_in(&path_var, cwd.as_deref(), name)
}

/// Testable core of [`find_on_path`]. Walks `path_var` (a `$PATH`-style
/// `OS_PATH_SEPARATOR`-joined string) looking for `name`, returning the
/// first absolute, existing candidate.
///
/// Entries are skipped when:
///   - they are empty (historical shorthand for CWD — never trust),
///   - they are not absolute (relative entries are implicitly CWD-rooted),
///   - they canonicalize to `cwd` (an attacker can plant `clang.exe` in a
///     directory they control and prepend it to PATH).
///
/// On Windows, the executable name is tried with each `PATHEXT`-style
/// suffix (`.exe`, `.bat`, `.cmd`, `.com`) in addition to the bare name.
pub fn find_on_path_in(
    path_var: &OsStr,
    cwd: Option<&Path>,
    name: &str,
) -> Option<PathBuf> {
    let canonical_cwd = cwd.and_then(|c| c.canonicalize().ok());
    let exts: &[&str] = if cfg!(windows) {
        &[".exe", ".bat", ".cmd", ".com", ""]
    } else {
        &[""]
    };

    for entry in std::env::split_paths(path_var) {
        // Empty PATH entry — historical shell shorthand for CWD. Skip.
        if entry.as_os_str().is_empty() {
            continue;
        }
        // Relative entries resolve against CWD — same attack surface. Skip.
        if !entry.is_absolute() {
            continue;
        }
        // Canonicalized-equal-to-CWD also means CWD. Skip.
        if let Some(canon_cwd) = canonical_cwd.as_ref() {
            if let Ok(canon_entry) = entry.canonicalize() {
                if &canon_entry == canon_cwd {
                    continue;
                }
            }
        }

        for ext in exts {
            let candidate = entry.join(format!("{name}{ext}"));
            if candidate.is_file() {
                // Prefer the canonical form so the absolute PathBuf we hand
                // to Command::new is stable across CWD changes.
                return Some(candidate.canonicalize().unwrap_or(candidate));
            }
        }
    }
    None
}

/// Rejects output names that would be interpreted as a flag by `clang`.
///
/// `clang` has no POSIX-style `--` end-of-options separator, so a file
/// whose name starts with `-` is read as a flag. `hmatc` refuses to build
/// under such a name rather than pass it through unchanged.
pub fn validate_output_name(name: &str) -> Result<(), CompilerError> {
    if name.starts_with('-') {
        return Err(CompilerError::InvalidOutputName(name.to_string()));
    }
    Ok(())
}

/// Builds absolute paths for the generated `.c` file and the output binary.
///
/// Both paths are rooted at `cwd`. The stem is validated up-front, and any
/// caller-supplied override's *file name* is also validated — an attacker
/// who controls `-o` should not be able to hand us `-oevil.exe` and have
/// it forwarded as a flag.
pub fn absolute_output_paths(
    cwd: &Path,
    stem: &str,
    exe_suffix: &str,
    output_override: Option<&Path>,
) -> Result<(PathBuf, PathBuf), CompilerError> {
    validate_output_name(stem)?;

    let c_path = cwd.join(format!("{stem}.c"));

    let exe_path = match output_override {
        Some(p) => {
            let name = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            validate_output_name(name)?;
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                cwd.join(p)
            }
        }
        None => cwd.join(format!("{stem}{exe_suffix}")),
    };

    Ok((c_path, exe_path))
}

/// Platform-appropriate binary suffix. `.exe` on Windows, empty elsewhere.
pub fn exe_suffix() -> &'static str {
    if cfg!(windows) {
        ".exe"
    } else {
        ""
    }
}
