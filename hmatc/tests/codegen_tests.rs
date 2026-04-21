//! Codegen integration tests.
//!
//! These tests run the front-end (lex + parse + type-check) and then
//! the C backend, verifying the emitted source textually. They do NOT
//! shell out to `clang` — the compile-and-run path is covered by the
//! `hmatc examples/hello_world.hm` smoke test described in CLAUDE.md,
//! not the unit suite (we do not want the test run to require clang).

use hmatc::{codegen, lexer, parser, semantic};

/// Run the front-end and return generated C. Panics with a readable
/// message if any stage errors — tests are expected to supply valid
/// programs.
fn c_source(src: &str) -> String {
    let tokens = lexer::tokenize(src).expect("lex");
    let program = parser::parse(tokens).expect("parse");
    semantic::check(&program).expect("type-check");
    codegen::emit_c(&program)
}

#[test]
fn hello_world_emits_printf() {
    let src = "fn main():\n    print(\"Hello, HMAT!\")\n";
    let out = c_source(src);

    assert!(
        out.contains("#include <stdio.h>"),
        "missing stdio include:\n{out}"
    );
    assert!(
        out.contains("int main(void)"),
        "expected `int main(void)` signature, got:\n{out}"
    );
    assert!(
        out.contains(r#"printf("%s\n", "Hello, HMAT!")"#),
        "expected printf lowering of print(), got:\n{out}"
    );
    assert!(
        out.contains("return 0;"),
        "main should auto-return 0:\n{out}"
    );
}

#[test]
fn let_binding_uses_auto_type_without_annotation() {
    let src = "fn main():\n    let x = 42\n    print(\"ok\")\n";
    let out = c_source(src);

    assert!(
        out.contains("const __auto_type x = ((int64_t)42LL);"),
        "expected __auto_type for un-annotated let, got:\n{out}"
    );
}

#[test]
fn let_binding_respects_int_annotation() {
    let src = "fn main():\n    let x: int = 42\n    print(\"ok\")\n";
    let out = c_source(src);

    assert!(
        out.contains("const int64_t x = ((int64_t)42LL);"),
        "expected int64_t for `int` annotation, got:\n{out}"
    );
}

#[test]
fn mut_binding_drops_const_qualifier() {
    let src = "fn main():\n    let mut x: int = 0\n    print(\"ok\")\n";
    let out = c_source(src);

    assert!(
        out.contains("int64_t x = ((int64_t)0LL);"),
        "expected mutable let to omit const, got:\n{out}"
    );
    assert!(
        !out.contains("const int64_t x"),
        "mutable let should not be const:\n{out}"
    );
}

#[test]
fn arithmetic_is_parenthesised() {
    let src = "fn main():\n    let x = 1 + 2 * 3\n    print(\"ok\")\n";
    let out = c_source(src);

    assert!(
        out.contains("(((int64_t)1LL) + (((int64_t)2LL) * ((int64_t)3LL)))"),
        "expected fully parenthesised arithmetic, got:\n{out}"
    );
}

#[test]
fn pow_lowers_to_libm() {
    // `a^b` in HMAT is exponentiation, not XOR.
    let src = "fn main():\n    let x: float = 2.0 ^ 3.0\n    print(\"ok\")\n";
    let out = c_source(src);

    assert!(
        out.contains("pow(2.0, 3.0)"),
        "expected `^` to lower to `pow()`, got:\n{out}"
    );
    assert!(out.contains("#include <math.h>"), "missing math.h include");
}

#[test]
fn string_escapes_survive_roundtrip() {
    let src = "fn main():\n    print(\"line1\\nline2\")\n";
    let out = c_source(src);

    assert!(
        out.contains(r#"printf("%s\n", "line1\nline2")"#),
        "expected escape to be preserved literally in C, got:\n{out}"
    );
}

#[test]
fn empty_main_still_returns_zero() {
    // A function body is never empty in HMAT (INDENT requires content),
    // but we can still exercise the auto-return path with a lone call.
    let src = "fn main():\n    print(\"hi\")\n";
    let out = c_source(src);

    assert!(
        out.contains("return 0;"),
        "main without explicit return should auto-return 0:\n{out}"
    );
}

#[test]
fn parameters_get_typed_signatures() {
    let src = concat!(
        "fn greet(name: str) -> str:\n",
        "    return name\n",
        "\n",
        "fn main():\n",
        "    print(\"hi\")\n",
    );
    let out = c_source(src);

    assert!(
        out.contains("const char * greet(const char * name)"),
        "expected typed C parameter, got:\n{out}"
    );
}

#[test]
fn bool_literals_lower_to_stdbool() {
    let src = "fn main():\n    let flag: bool = true\n    print(\"ok\")\n";
    let out = c_source(src);

    assert!(
        out.contains("#include <stdbool.h>"),
        "missing stdbool include"
    );
    assert!(
        out.contains("const bool flag = true;"),
        "expected C `true`, got:\n{out}"
    );
}

// ===========================================================================
// Arithmetic operators — Sub, Div, Mod
// ===========================================================================

#[test]
fn subtraction_division_modulo_lower_to_c_operators() {
    let src = concat!(
        "fn main():\n",
        "    let a: int = 5 - 3\n",
        "    let b: int = 6 / 2\n",
        "    let c: int = 7 % 3\n",
        "    print(\"ok\")\n",
    );
    let out = c_source(src);

    assert!(
        out.contains("(((int64_t)5LL) - ((int64_t)3LL))"),
        "expected subtraction to lower to `-`, got:\n{out}"
    );
    assert!(
        out.contains("(((int64_t)6LL) / ((int64_t)2LL))"),
        "expected division to lower to `/`, got:\n{out}"
    );
    assert!(
        out.contains("(((int64_t)7LL) % ((int64_t)3LL))"),
        "expected modulo to lower to `%`, got:\n{out}"
    );
}

// ===========================================================================
// Comparison operators
// ===========================================================================

#[test]
fn comparison_eq_neq_lower_to_c() {
    let src = concat!(
        "fn main():\n",
        "    let a: bool = 1 == 1\n",
        "    let b: bool = 2 != 3\n",
        "    print(\"ok\")\n",
    );
    let out = c_source(src);

    assert!(
        out.contains("((int64_t)1LL) == ((int64_t)1LL)"),
        "expected `==` in C output, got:\n{out}"
    );
    assert!(
        out.contains("((int64_t)2LL) != ((int64_t)3LL)"),
        "expected `!=` in C output, got:\n{out}"
    );
}

#[test]
fn comparison_ordering_lower_to_c() {
    let src = concat!(
        "fn main():\n",
        "    let a: bool = 1 < 2\n",
        "    let b: bool = 2 <= 2\n",
        "    let c: bool = 3 > 2\n",
        "    let d: bool = 3 >= 3\n",
        "    print(\"ok\")\n",
    );
    let out = c_source(src);

    assert!(
        out.contains("((int64_t)1LL) < ((int64_t)2LL)"),
        "expected `<` in C output, got:\n{out}"
    );
    assert!(
        out.contains("((int64_t)2LL) <= ((int64_t)2LL)"),
        "expected `<=` in C output, got:\n{out}"
    );
    assert!(
        out.contains("((int64_t)3LL) > ((int64_t)2LL)"),
        "expected `>` in C output, got:\n{out}"
    );
    assert!(
        out.contains("((int64_t)3LL) >= ((int64_t)3LL)"),
        "expected `>=` in C output, got:\n{out}"
    );
}

// ===========================================================================
// Logical operators — `and` → &&, `or` → ||
// ===========================================================================

#[test]
fn logical_and_lowers_to_double_ampersand() {
    let src = "fn main():\n    let b: bool = true and false\n    print(\"ok\")\n";
    let out = c_source(src);

    assert!(
        out.contains("(true && false)"),
        "expected `and` to lower to `&&`, got:\n{out}"
    );
}

#[test]
fn logical_or_lowers_to_double_pipe() {
    let src = "fn main():\n    let b: bool = true or false\n    print(\"ok\")\n";
    let out = c_source(src);

    assert!(
        out.contains("(true || false)"),
        "expected `or` to lower to `||`, got:\n{out}"
    );
}

// ===========================================================================
// Unary operators — `-` and `not`
// ===========================================================================

#[test]
fn unary_neg_wraps_operand_in_parens() {
    let src = "fn main():\n    let x: int = -1\n    print(\"ok\")\n";
    let out = c_source(src);

    assert!(
        out.contains("(-((int64_t)1LL))"),
        "expected unary `-` to emit `(-expr)`, got:\n{out}"
    );
}

#[test]
fn unary_not_wraps_operand_in_bang() {
    let src = "fn main():\n    let b: bool = not true\n    print(\"ok\")\n";
    let out = c_source(src);

    assert!(
        out.contains("(!true)"),
        "expected `not` to emit `(!expr)`, got:\n{out}"
    );
}

// ===========================================================================
// Float literals and type annotation
// ===========================================================================

#[test]
fn float_literal_preserves_decimal_point() {
    // format_float must include a decimal point so clang treats the value as
    // a floating-point constant, not an integer.
    let src = "fn main():\n    let x: float = 3.14\n    print(\"ok\")\n";
    let out = c_source(src);

    assert!(
        out.contains("const double x = 3.14;"),
        "expected float literal to keep its decimal, got:\n{out}"
    );
}

#[test]
fn float_one_point_zero_keeps_decimal() {
    // `1.0` must not be emitted as bare `1` — that changes the C type.
    let src = "fn main():\n    let x: float = 1.0\n    print(\"ok\")\n";
    let out = c_source(src);

    assert!(
        out.contains("const double x = 1.0;"),
        "expected `1.0` (not `1`) in float literal, got:\n{out}"
    );
}

#[test]
fn f64_annotation_maps_to_double() {
    // Alias `f64` must lower to the same C type as `float`.
    let src = "fn main():\n    let x: f64 = 2.5\n    print(\"ok\")\n";
    let out = c_source(src);

    assert!(
        out.contains("const double x = 2.5;"),
        "expected f64 → C `double`, got:\n{out}"
    );
}

#[test]
fn f32_annotation_maps_to_c_float() {
    let src = "fn main():\n    let x: f32 = 1.0\n    print(\"ok\")\n";
    let out = c_source(src);

    assert!(
        out.contains("const float x = 1.0;"),
        "expected f32 → C `float`, got:\n{out}"
    );
}

// ===========================================================================
// Bool false literal
// ===========================================================================

#[test]
fn false_literal_lowers_to_c_false() {
    let src = "fn main():\n    let flag: bool = false\n    print(\"ok\")\n";
    let out = c_source(src);

    assert!(
        out.contains("const bool flag = false;"),
        "expected `false` literal, got:\n{out}"
    );
}

// ===========================================================================
// f-string — treated as plain string in Phase 1
// ===========================================================================

#[test]
fn fstring_treated_as_plain_string_in_phase1() {
    // Phase 1 does not split f-string interpolations; the body is emitted
    // verbatim as a C string literal via the same escape_c_string path.
    let src = "fn main():\n    let msg = f\"hello\"\n    print(\"ok\")\n";
    let out = c_source(src);

    assert!(
        out.contains("\"hello\""),
        "expected f-string body emitted as plain C string, got:\n{out}"
    );
}

// ===========================================================================
// Identifier as expression — variable reference
// ===========================================================================

#[test]
fn identifier_in_expression_emits_name() {
    let src = concat!(
        "fn main():\n",
        "    let x: int = 1\n",
        "    let y: int = x\n",
        "    print(\"ok\")\n",
    );
    let out = c_source(src);

    assert!(
        out.contains("const int64_t y = x;"),
        "expected identifier `x` to appear as-is in C RHS, got:\n{out}"
    );
}

// ===========================================================================
// Return statement with value
// ===========================================================================

#[test]
fn return_with_value_emits_return_stmt() {
    let src = concat!(
        "fn add(a: int, b: int) -> int:\n",
        "    return a + b\n",
        "\n",
        "fn main():\n",
        "    print(\"ok\")\n",
    );
    let out = c_source(src);

    assert!(
        out.contains("return (a + b);"),
        "expected `return <expr>;` for return-with-value, got:\n{out}"
    );
}

// ===========================================================================
// Non-main unit function → void return type
// ===========================================================================

#[test]
fn non_main_unit_function_emits_void_return_type() {
    let src = concat!(
        "fn helper(x: int):\n",
        "    print(\"hi\")\n",
        "\n",
        "fn main():\n",
        "    print(\"ok\")\n",
    );
    let out = c_source(src);

    assert!(
        out.contains("void helper(int64_t x)"),
        "expected void return type for unit function, got:\n{out}"
    );
}

// ===========================================================================
// User-defined function call
// ===========================================================================

#[test]
fn user_defined_call_emits_callee_and_args() {
    let src = concat!(
        "fn add(a: int, b: int) -> int:\n",
        "    return a + b\n",
        "\n",
        "fn main():\n",
        "    let r: int = add(1, 2)\n",
        "    print(\"ok\")\n",
    );
    let out = c_source(src);

    assert!(
        out.contains("add(((int64_t)1LL), ((int64_t)2LL))"),
        "expected user-defined call with typed args, got:\n{out}"
    );
}

// ===========================================================================
// Multiple parameters in one function signature
// ===========================================================================

#[test]
fn multiple_params_emit_correct_c_signature() {
    let src = concat!(
        "fn add(a: int, b: int) -> int:\n",
        "    return a + b\n",
        "\n",
        "fn main():\n",
        "    print(\"ok\")\n",
    );
    let out = c_source(src);

    assert!(
        out.contains("int64_t add(int64_t a, int64_t b)"),
        "expected multi-param C signature, got:\n{out}"
    );
}

// ===========================================================================
// Grouped expression — extra parens layer in C
// ===========================================================================

#[test]
fn grouped_expr_adds_extra_parens_in_c() {
    // Grouped node wraps an already-parenthesised binary in one more layer,
    // so `(1 + 2)` in HMAT produces `(((...1...) + (...2...)))` in C.
    let src = "fn main():\n    let x: int = (1 + 2)\n    print(\"ok\")\n";
    let out = c_source(src);

    assert!(
        out.contains("((((int64_t)1LL) + ((int64_t)2LL)))"),
        "expected extra parens from Grouped AST node, got:\n{out}"
    );
}

// ===========================================================================
// String escape round-trips through escape_c_string
// ===========================================================================

#[test]
fn string_with_embedded_quote_is_escaped() {
    // HMAT source: "say \"hi\"" — lexer decodes \" → "; codegen re-encodes.
    let src = "fn main():\n    print(\"say \\\"hi\\\"\")\n";
    let out = c_source(src);

    assert!(
        out.contains(r#"printf("%s\n", "say \"hi\"")"#),
        "expected \" in string to be escaped as \\\" in C, got:\n{out}"
    );
}

#[test]
fn string_with_backslash_is_escaped() {
    // HMAT source: "back\\slash" — lexer decodes \\ → \; codegen re-encodes.
    let src = "fn main():\n    print(\"back\\\\slash\")\n";
    let out = c_source(src);

    assert!(
        out.contains(r#"printf("%s\n", "back\\slash")"#),
        "expected backslash to be doubled to \\\\ in C, got:\n{out}"
    );
}

#[test]
fn string_with_tab_escape_roundtrips() {
    // HMAT source: "a\tb" — lexer decodes \t → tab char; codegen re-encodes
    // it as the two-character C escape `\t`.
    let src = "fn main():\n    print(\"a\\tb\")\n";
    let out = c_source(src);

    assert!(
        out.contains(r#"printf("%s\n", "a\tb")"#),
        "expected tab char to be emitted as \\t in C, got:\n{out}"
    );
}

// ===========================================================================
// ends_with_return — explicit unit `return` suppresses auto `return 0;`
// ===========================================================================

#[test]
fn explicit_unit_return_in_main_suppresses_auto_return_zero() {
    // When main() ends with an explicit `return` (no value), the emitter must
    // NOT add a second `return 0;`.
    let src = "fn main():\n    print(\"hi\")\n    return\n";
    let out = c_source(src);

    assert!(
        out.contains("return;"),
        "expected explicit `return;`:\n{out}"
    );
    assert_eq!(
        out.matches("return").count(),
        1,
        "expected exactly one `return` in main (no auto `return 0;`), got:\n{out}"
    );
}

// ===========================================================================
// Integer width aliases — map_type coverage
// ===========================================================================

#[test]
fn i32_annotation_maps_to_int32_t() {
    let src = "fn main():\n    let x: i32 = 1\n    print(\"ok\")\n";
    let out = c_source(src);

    assert!(
        out.contains("const int32_t x = ((int64_t)1LL);"),
        "expected i32 → int32_t, got:\n{out}"
    );
}

// ===========================================================================
// nil literal → NULL  (bypasses type-checker — nil cannot type-check in Phase 0)
// ===========================================================================

/// Runs lex + parse + codegen, skipping the type-checker. Used only for
/// constructs that are syntactically valid but cannot yet satisfy Phase 0
/// typing (e.g. bare `nil` with no nilable annotation).
fn c_source_raw(src: &str) -> String {
    let tokens = lexer::tokenize(src).expect("lex");
    let program = parser::parse(tokens).expect("parse");
    codegen::emit_c(&program)
}

#[test]
fn nil_literal_lowers_to_null() {
    // `nil` cannot pass the Phase 0 type-checker without a nilable annotation,
    // so we bypass semantic analysis and verify only the codegen output.
    let src = "fn main():\n    let x = nil\n    print(\"ok\")\n";
    let out = c_source_raw(src);

    assert!(
        out.contains("const __auto_type x = NULL;"),
        "expected `nil` to emit C `NULL`, got:\n{out}"
    );
}
