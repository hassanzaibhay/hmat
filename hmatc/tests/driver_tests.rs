//! Driver-layer regression tests for the 2026-04-21 security audit fixes.
//!
//! Each test here maps to a finding in `security/SESSION-2026-04-21.md`:
//!   - FIX 1 (HIGH, CWE-427): `find_on_path_in` must skip CWD and require
//!     absolute entries so a planted `clang.exe` in the run directory
//!     cannot hijack the compile.
//!   - FIX 2 (MEDIUM): output names beginning with `-` must be rejected
//!     before they are forwarded to `clang`.
//!   - FIX 2 (MEDIUM): the emitted `.c` and output binary must sit under
//!     an absolute directory, never a bare relative path.

use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use hmatc::driver;
use hmatc::CompilerError;

/// Join PATH entries with the platform separator the same way the shell does.
fn join_path(entries: &[&Path]) -> OsString {
    let sep = if cfg!(windows) { ";" } else { ":" };
    let joined = entries
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(sep);
    OsString::from(joined)
}

// ---------------------------------------------------------------------------
// FIX 1 — PATH walking skips the current working directory (CWE-427)
// ---------------------------------------------------------------------------

#[test]
fn find_on_path_skips_cwd_even_when_it_appears_in_path() {
    // Build a PATH whose only entry is the temp-dir CWD, drop a fake
    // "clang" binary inside, and confirm the lookup refuses to find it.
    let tmp = env::temp_dir().join("hmatc_cwe427_cwd");
    std::fs::create_dir_all(&tmp).unwrap();

    let fake_name = if cfg!(windows) {
        "fake_hmatc_clang.exe"
    } else {
        "fake_hmatc_clang"
    };
    let fake = tmp.join(fake_name);
    std::fs::write(&fake, b"not a real clang").unwrap();

    let path_var = join_path(&[&tmp]);
    let found = driver::find_on_path_in(&path_var, Some(&tmp), "fake_hmatc_clang");

    assert!(
        found.is_none(),
        "CWD must be skipped during PATH walk, got {found:?}"
    );

    // Best-effort cleanup — leaving the file behind is harmless.
    let _ = std::fs::remove_file(&fake);
}

#[test]
fn find_on_path_ignores_relative_path_entries() {
    // A PATH entry like "." or "bin" must never be searched — otherwise a
    // checked-in repo directory could poison the toolchain resolution.
    let relative = PathBuf::from(".");
    let path_var = join_path(&[&relative]);
    let found = driver::find_on_path_in(&path_var, None, "clang");

    assert!(
        found.is_none(),
        "relative PATH entry must be skipped, got {found:?}"
    );
}

#[test]
fn find_on_path_ignores_empty_entries() {
    // An empty PATH entry ("::" on Unix, ";;" on Windows) is historical
    // shorthand for CWD. Never trust it.
    let path_var = if cfg!(windows) {
        OsString::from(";;")
    } else {
        OsString::from("::")
    };
    let found = driver::find_on_path_in(&path_var, None, "clang");

    assert!(
        found.is_none(),
        "empty PATH entries must be skipped, got {found:?}"
    );
}

#[test]
fn find_on_path_returns_absolute_when_binary_exists() {
    // Drop a dummy "binary" into a temp dir, point PATH at that temp dir
    // (from a *different* CWD), and confirm the lookup returns an
    // absolute PathBuf. This is the positive path that proves the
    // CWD-skip logic does not also break the normal case.
    let tmp = env::temp_dir().join("hmatc_cwe427_abs");
    std::fs::create_dir_all(&tmp).unwrap();

    let fake_name = if cfg!(windows) {
        "hmatc_abs_probe.exe"
    } else {
        "hmatc_abs_probe"
    };
    let fake = tmp.join(fake_name);
    std::fs::write(&fake, b"stub").unwrap();

    // Use a CWD that is *not* `tmp` so the skip rule does not apply.
    let cwd = env::temp_dir();
    let path_var = join_path(&[&tmp]);
    let found = driver::find_on_path_in(&path_var, Some(&cwd), "hmatc_abs_probe");

    match found {
        Some(p) => assert!(p.is_absolute(), "resolved path must be absolute: {p:?}"),
        None => panic!("expected to locate {fake:?} via PATH walk"),
    }

    let _ = std::fs::remove_file(&fake);
}

// ---------------------------------------------------------------------------
// FIX 2 — Output-name validation (E005)
// ---------------------------------------------------------------------------

#[test]
fn leading_dash_in_stem_is_rejected_with_e005() {
    let err = driver::validate_output_name("-oevil").unwrap_err();
    let msg = format!("{err}");
    assert!(
        matches!(err, CompilerError::InvalidOutputName(ref s) if s == "-oevil"),
        "expected InvalidOutputName variant, got {err:?}"
    );
    assert!(
        msg.contains("E005"),
        "error message must carry the E005 code, got: {msg}"
    );
}

#[test]
fn normal_stem_passes_validation() {
    driver::validate_output_name("hello_world").expect("ordinary names must pass");
    driver::validate_output_name("a.b.c").expect("dots in stems are fine");
    driver::validate_output_name("_leading_underscore").expect("underscores are fine");
}

#[test]
fn output_override_with_leading_dash_is_rejected() {
    let cwd = env::temp_dir();
    let bad_override = PathBuf::from("-oevil.exe");
    let result =
        driver::absolute_output_paths(&cwd, "hello", ".exe", Some(&bad_override));

    match result {
        Err(CompilerError::InvalidOutputName(name)) => assert_eq!(name, "-oevil.exe"),
        other => panic!("expected E005 rejection, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// FIX 2 — Generated paths are always absolute
// ---------------------------------------------------------------------------

#[test]
fn generated_paths_are_anchored_at_cwd() {
    let cwd = env::temp_dir();
    let (c_path, exe_path) =
        driver::absolute_output_paths(&cwd, "hello", ".exe", None).expect("valid");

    assert!(c_path.is_absolute(), ".c path must be absolute: {c_path:?}");
    assert!(
        exe_path.is_absolute(),
        "exe path must be absolute: {exe_path:?}"
    );
    assert!(
        c_path.starts_with(&cwd),
        ".c path must live under the given CWD: {c_path:?} vs {cwd:?}"
    );
    assert!(
        exe_path.starts_with(&cwd),
        "exe path must live under the given CWD: {exe_path:?} vs {cwd:?}"
    );
    assert_eq!(c_path.file_name().unwrap(), "hello.c");
    assert_eq!(exe_path.file_name().unwrap(), "hello.exe");
}

#[test]
fn relative_output_override_is_promoted_to_absolute() {
    // A user-supplied `-o out.bin` must be rooted under CWD, never left as
    // a bare relative path that would resolve against wherever clang runs.
    let cwd = env::temp_dir();
    let over = PathBuf::from("out.bin");
    let (_, exe_path) =
        driver::absolute_output_paths(&cwd, "hello", ".exe", Some(&over)).expect("valid");

    assert!(
        exe_path.is_absolute(),
        "relative override must become absolute, got {exe_path:?}"
    );
    assert_eq!(exe_path.file_name().unwrap(), "out.bin");
    assert!(exe_path.starts_with(&cwd));
}
