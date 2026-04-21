//! Integration tests for the HMAT type checker.
//!
//! Two categories:
//!   - **Happy path** — well-formed inputs that must type-check with zero
//!     errors.
//!   - **Error path** — malformed inputs that must produce a `TypeError`
//!     with the right code and a non-empty `help()` string (Zen #2:
//!     errors are teachers).
//!
//! The Phase 0 milestone — `hmatc --emit=ast hello_world.hm` type-checks
//! cleanly — is the first test in this file.

use hmatc::lexer::tokenize;
use hmatc::parser::parse;
use hmatc::semantic::{check, TypeError};

/// Short-hand: lex + parse + type-check; expect success.
fn ok(source: &str) {
    let tokens = tokenize(source).expect("lex should succeed");
    let program = parse(tokens).expect("parse should succeed");
    if let Err(errors) = check(&program) {
        panic!(
            "expected type check to succeed, got {} error(s): {:#?}",
            errors.len(),
            errors
        );
    }
}

/// Short-hand: lex + parse + type-check; expect failure. Returns every
/// error so tests can assert on codes or counts.
fn err(source: &str) -> Vec<TypeError> {
    let tokens = tokenize(source).expect("lex should succeed");
    let program = parse(tokens).expect("parse should succeed");
    check(&program).expect_err("type check should fail")
}

/// Assert that `errs` contains an error with the given E-code.
fn has_code(errs: &[TypeError], code: &str) -> bool {
    errs.iter().any(|e| e.code() == code)
}

// ======================================================================
// Milestone — hello_world.hm type-checks cleanly
// ======================================================================

#[test]
fn hello_world_type_checks() {
    let src = "fn main():\n    print(\"Hello, HMAT!\")\n";
    ok(src);
}

// ======================================================================
// Happy path — primitives and literals
// ======================================================================

#[test]
fn let_infers_int_from_literal() {
    ok("fn main():\n    let x = 42\n");
}

#[test]
fn let_infers_float_from_literal() {
    ok("fn main():\n    let x = 3.14\n");
}

#[test]
fn let_infers_str_from_literal() {
    ok("fn main():\n    let s = \"hello\"\n");
}

#[test]
fn let_infers_bool_from_literal() {
    ok("fn main():\n    let b = true\n");
}

#[test]
fn let_with_matching_annotation() {
    ok("fn main():\n    let x: int = 1\n    let y: float = 1.5\n    let s: str = \"hi\"\n    let b: bool = false\n");
}

#[test]
fn fstring_has_str_type() {
    ok("fn main():\n    let s: str = f\"hi {x}\"\n");
}

// ======================================================================
// Happy path — operators
// ======================================================================

#[test]
fn int_arithmetic_type_checks() {
    ok("fn main():\n    let x: int = 1 + 2 * 3 - 4\n");
}

#[test]
fn float_arithmetic_type_checks() {
    ok("fn main():\n    let x: float = 1.0 / 2.0 + 3.0\n");
}

#[test]
fn power_is_numeric_and_returns_same_type() {
    ok("fn main():\n    let x: int = 2 ^ 3\n    let y: float = 2.0 ^ 3.0\n");
}

#[test]
fn string_concatenation_allowed() {
    ok("fn main():\n    let s: str = \"a\" + \"b\"\n");
}

#[test]
fn comparisons_return_bool() {
    ok("fn main():\n    let b: bool = 1 < 2\n    let c: bool = 1.0 >= 2.0\n    let d: bool = \"a\" == \"b\"\n");
}

#[test]
fn logical_ops_on_bool() {
    ok("fn main():\n    let b: bool = true and false or not true\n");
}

#[test]
fn unary_neg_on_numeric() {
    ok("fn main():\n    let x: int = -5\n    let y: float = -3.14\n");
}

#[test]
fn grouped_expressions_transparent() {
    ok("fn main():\n    let x: int = (1 + 2) * 3\n");
}

// ======================================================================
// Happy path — functions
// ======================================================================

#[test]
fn function_with_return_type_checks() {
    ok("fn add(a: int, b: int) -> int:\n    return a + b\n");
}

#[test]
fn function_returning_bool() {
    ok("fn is_pos(x: int) -> bool:\n    return x > 0\n");
}

#[test]
fn unit_return_no_explicit_type() {
    ok("fn greet(name: str):\n    print(name)\n");
}

#[test]
fn user_defined_function_callable() {
    let src = "\
fn double(x: int) -> int:
    return x + x

fn main():
    let y: int = double(21)
";
    ok(src);
}

#[test]
fn params_visible_in_body() {
    ok("fn f(a: int, b: int) -> int:\n    let c: int = a + b\n    return c\n");
}

#[test]
fn explicit_return_none_in_unit_fn() {
    ok("fn f():\n    return\n");
}

// ======================================================================
// Error path — let bindings
// ======================================================================

#[test]
fn annotation_mismatch_str_vs_int() {
    let errs = err("fn main():\n    let x: int = \"hello\"\n");
    assert!(has_code(&errs, "E100"), "expected E100, got {errs:?}");
    assert!(!errs[0].help().is_empty());
}

#[test]
fn annotation_mismatch_int_vs_float() {
    let errs = err("fn main():\n    let x: int = 1.5\n");
    assert!(has_code(&errs, "E100"));
}

#[test]
fn nil_without_annotation_cannot_infer() {
    let errs = err("fn main():\n    let x = nil\n");
    assert!(has_code(&errs, "E101"));
    assert!(!errs[0].help().is_empty());
}

#[test]
fn unknown_type_in_annotation() {
    let errs = err("fn main():\n    let x: flibble = 1\n");
    assert!(has_code(&errs, "E110"));
}

// ======================================================================
// Error path — operators
// ======================================================================

#[test]
fn add_int_and_str_is_error() {
    let errs = err("fn main():\n    let x = 1 + \"two\"\n");
    assert!(has_code(&errs, "E102"));
    assert!(!errs[0].help().is_empty());
}

#[test]
fn int_plus_float_is_error_no_coercion() {
    let errs = err("fn main():\n    let x = 1 + 2.0\n");
    assert!(has_code(&errs, "E102"));
}

#[test]
fn logical_and_on_non_bool_is_error() {
    let errs = err("fn main():\n    let x = 1 and 2\n");
    assert!(has_code(&errs, "E102"));
}

#[test]
fn not_on_int_is_error() {
    let errs = err("fn main():\n    let x = not 5\n");
    assert!(has_code(&errs, "E103"));
    assert!(!errs[0].help().is_empty());
}

#[test]
fn neg_on_str_is_error() {
    let errs = err("fn main():\n    let x = -\"hi\"\n");
    assert!(has_code(&errs, "E103"));
}

// ======================================================================
// Error path — identifiers and calls
// ======================================================================

#[test]
fn undefined_variable_reports_e104() {
    let errs = err("fn main():\n    let x = y + 1\n");
    assert!(has_code(&errs, "E104"));
}

#[test]
fn undefined_function_reports_e105() {
    let errs = err("fn main():\n    unknown_fn()\n");
    assert!(has_code(&errs, "E105"));
}

#[test]
fn wrong_arg_count_reports_e106() {
    let errs = err("fn main():\n    print(\"a\", \"b\")\n");
    assert!(has_code(&errs, "E106"));
}

#[test]
fn arg_type_mismatch_reports_e107() {
    let errs = err("fn main():\n    print(42)\n");
    assert!(has_code(&errs, "E107"));
}

// ======================================================================
// Error path — returns
// ======================================================================

#[test]
fn return_type_mismatch_reports_e109() {
    let errs = err("fn get_num() -> int:\n    return \"hi\"\n");
    assert!(has_code(&errs, "E109"));
    assert!(!errs[0].help().is_empty());
}

#[test]
fn empty_return_in_int_fn_is_error() {
    let errs = err("fn get_num() -> int:\n    return\n");
    assert!(has_code(&errs, "E109"));
}

#[test]
fn return_value_in_unit_fn_is_error() {
    let errs = err("fn greet():\n    return 1\n");
    assert!(has_code(&errs, "E109"));
}

// ======================================================================
// Sanity — every variant has a help()
// ======================================================================

#[test]
fn every_error_has_help_text() {
    // Collect one instance of each code by crafting sources that trigger them.
    let sources = [
        "fn main():\n    let x: int = \"hi\"\n", // E100
        "fn main():\n    let x = nil\n",         // E101
        "fn main():\n    let x = 1 + \"a\"\n",   // E102
        "fn main():\n    let x = not 5\n",       // E103
        "fn main():\n    let x = y\n",           // E104
        "fn main():\n    zzz()\n",               // E105
        "fn main():\n    print(\"a\", \"b\")\n", // E106
        "fn main():\n    print(42)\n",           // E107
        "fn f() -> int:\n    return \"hi\"\n",   // E109
        "fn main():\n    let x: flibble = 1\n",  // E110
    ];
    for src in sources {
        let errs = err(src);
        for e in &errs {
            assert!(!e.help().is_empty(), "code {} has empty help()", e.code());
        }
    }
}
