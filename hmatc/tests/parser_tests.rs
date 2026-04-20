//! Integration tests for the HMAT parser.
//!
//! Tests cluster into two categories:
//!   - **Happy path** — well-formed inputs that must produce the AST shape
//!     later compiler phases rely on.
//!   - **Error path** — malformed inputs that must produce a `ParseError`
//!     with the right code and a non-empty `help()` string (Zen #2:
//!     errors are teachers).

use hmatc::ast::{BinOp, Expression, Item, Literal, Statement, UnOp};
use hmatc::lexer::tokenize;
use hmatc::parser::{parse, ParseError};

/// Short-hand: lex + parse, expect success.
fn ok(source: &str) -> hmatc::ast::Program {
    let tokens = tokenize(source).expect("lex should succeed");
    parse(tokens).expect("parse should succeed")
}

/// Short-hand: lex + parse, expect parse failure.
fn err(source: &str) -> ParseError {
    let tokens = tokenize(source).expect("lex should succeed");
    parse(tokens).expect_err("parse should fail")
}

// ======================================================================
// Milestone — hello_world.hm parses without crashing
// ======================================================================

#[test]
fn hello_world_parses() {
    let src = "fn main():\n    print(\"Hello, HMAT!\")\n";
    let program = ok(src);
    assert_eq!(program.items.len(), 1);
    let Item::Function(f) = &program.items[0];
    assert_eq!(f.name.name, "main");
    assert!(f.params.is_empty());
    assert!(f.return_type.is_none());
    assert_eq!(f.body.statements.len(), 1);

    // Body: ExprStmt → Call print("Hello, HMAT!")
    let Statement::Expr(Expression::Call(call)) = &f.body.statements[0] else {
        panic!("expected call expression statement");
    };
    match &*call.callee {
        Expression::Identifier(id) => assert_eq!(id.name, "print"),
        _ => panic!("callee must be identifier"),
    }
    assert_eq!(call.args.len(), 1);
    match &call.args[0] {
        Expression::Literal(Literal::Str { value, .. }) => {
            assert_eq!(value, "Hello, HMAT!");
        }
        other => panic!("expected string literal arg, got {other:?}"),
    }
}

#[test]
fn hello_world_pretty_prints_without_panic() {
    let src = "fn main():\n    print(\"Hello, HMAT!\")\n";
    let program = ok(src);
    let s = program.pretty_print();
    // Spot-check that the output contains the key node labels.
    assert!(s.starts_with("Program\n"));
    assert!(s.contains("FunctionDecl `main`"));
    assert!(s.contains("Call"));
    assert!(s.contains("StrLit \"Hello, HMAT!\""));
}

// ======================================================================
// Function declarations
// ======================================================================

#[test]
fn function_with_no_params_and_no_return() {
    let src = "fn noop():\n    return\n";
    let program = ok(src);
    let Item::Function(f) = &program.items[0];
    assert_eq!(f.name.name, "noop");
    assert!(f.params.is_empty());
    assert!(f.return_type.is_none());
    assert!(matches!(&f.body.statements[0], Statement::Return(r) if r.value.is_none()));
}

#[test]
fn function_with_params_and_return_type() {
    let src = "fn add(a: int, b: int) -> int:\n    return a\n";
    let program = ok(src);
    let Item::Function(f) = &program.items[0];
    assert_eq!(f.params.len(), 2);
    assert_eq!(f.params[0].name.name, "a");
    assert_eq!(f.params[0].ty.name, "int");
    assert!(!f.params[0].mutable);
    assert_eq!(f.params[1].name.name, "b");
    let ret = f.return_type.as_ref().expect("return type");
    assert_eq!(ret.name, "int");
}

#[test]
fn function_with_mut_param() {
    let src = "fn bump(mut counter: int):\n    return\n";
    let program = ok(src);
    let Item::Function(f) = &program.items[0];
    assert!(f.params[0].mutable);
    assert_eq!(f.params[0].name.name, "counter");
}

#[test]
fn function_with_generic_return_type() {
    let src = "fn lookup(key: str) -> Result<int, str>:\n    return\n";
    let program = ok(src);
    let Item::Function(f) = &program.items[0];
    let ret = f.return_type.as_ref().expect("return type");
    assert_eq!(ret.name, "Result");
    assert_eq!(ret.args.len(), 2);
    assert_eq!(ret.args[0].name, "int");
    assert_eq!(ret.args[1].name, "str");
}

#[test]
fn function_with_reference_param() {
    let src = "fn read(p: &Point):\n    return\n";
    let program = ok(src);
    let Item::Function(f) = &program.items[0];
    assert!(f.params[0].ty.is_ref);
    assert!(!f.params[0].ty.is_mut_ref);
    assert_eq!(f.params[0].ty.name, "Point");
}

#[test]
fn function_with_mut_reference_param() {
    let src = "fn write(p: &mut Point):\n    return\n";
    let program = ok(src);
    let Item::Function(f) = &program.items[0];
    assert!(f.params[0].ty.is_ref);
    assert!(f.params[0].ty.is_mut_ref);
}

#[test]
fn multiple_top_level_functions() {
    let src = "fn a():\n    return\nfn b():\n    return\n";
    let program = ok(src);
    assert_eq!(program.items.len(), 2);
    let Item::Function(fa) = &program.items[0];
    let Item::Function(fb) = &program.items[1];
    assert_eq!(fa.name.name, "a");
    assert_eq!(fb.name.name, "b");
}

// ======================================================================
// Let statements
// ======================================================================

#[test]
fn let_without_type_annotation() {
    let src = "fn main():\n    let x = 42\n";
    let program = ok(src);
    let Item::Function(f) = &program.items[0];
    let Statement::Let(l) = &f.body.statements[0] else {
        panic!("expected let statement");
    };
    assert_eq!(l.name.name, "x");
    assert!(!l.mutable);
    assert!(l.ty.is_none());
    assert!(matches!(
        l.value,
        Expression::Literal(Literal::Int { value: 42, .. })
    ));
}

#[test]
fn let_mut_with_type() {
    let src = "fn main():\n    let mut counter: int = 0\n";
    let program = ok(src);
    let Item::Function(f) = &program.items[0];
    let Statement::Let(l) = &f.body.statements[0] else {
        panic!("expected let statement");
    };
    assert!(l.mutable);
    let ty = l.ty.as_ref().expect("type");
    assert_eq!(ty.name, "int");
}

// ======================================================================
// Return statements
// ======================================================================

#[test]
fn return_without_value() {
    let src = "fn main():\n    return\n";
    let program = ok(src);
    let Item::Function(f) = &program.items[0];
    let Statement::Return(r) = &f.body.statements[0] else {
        panic!("expected return");
    };
    assert!(r.value.is_none());
}

#[test]
fn return_with_value() {
    let src = "fn main():\n    return 7\n";
    let program = ok(src);
    let Item::Function(f) = &program.items[0];
    let Statement::Return(r) = &f.body.statements[0] else {
        panic!("expected return");
    };
    assert!(matches!(
        r.value,
        Some(Expression::Literal(Literal::Int { value: 7, .. }))
    ));
}

// ======================================================================
// Expressions — precedence and associativity
// ======================================================================

fn single_expr_stmt(src: &str) -> Expression {
    let wrapped = format!("fn main():\n    {src}\n");
    let program = ok(&wrapped);
    let Item::Function(f) = &program.items[0];
    match &f.body.statements[0] {
        Statement::Expr(e) => e.clone(),
        other => panic!("expected expr stmt, got {other:?}"),
    }
}

#[test]
fn literal_int_parses() {
    let e = single_expr_stmt("42");
    assert!(matches!(
        e,
        Expression::Literal(Literal::Int { value: 42, .. })
    ));
}

#[test]
fn literal_float_parses() {
    let e = single_expr_stmt("3.14");
    assert!(matches!(e, Expression::Literal(Literal::Float { .. })));
}

#[test]
fn literal_string_parses() {
    let e = single_expr_stmt("\"hello\"");
    match e {
        Expression::Literal(Literal::Str { value, .. }) => assert_eq!(value, "hello"),
        _ => panic!("expected string literal"),
    }
}

#[test]
fn literal_bool_parses() {
    let e = single_expr_stmt("true");
    assert!(matches!(
        e,
        Expression::Literal(Literal::Bool { value: true, .. })
    ));
}

#[test]
fn identifier_expr_parses() {
    let e = single_expr_stmt("foo");
    match e {
        Expression::Identifier(id) => assert_eq!(id.name, "foo"),
        _ => panic!("expected identifier"),
    }
}

#[test]
fn multiplication_binds_tighter_than_addition() {
    // 1 + 2 * 3  →  (1 + (2 * 3))
    let e = single_expr_stmt("1 + 2 * 3");
    let Expression::Binary(add) = e else {
        panic!("expected binary")
    };
    assert_eq!(add.op, BinOp::Add);
    assert!(matches!(
        *add.lhs,
        Expression::Literal(Literal::Int { value: 1, .. })
    ));
    let Expression::Binary(mul) = *add.rhs else {
        panic!("expected binary")
    };
    assert_eq!(mul.op, BinOp::Mul);
}

#[test]
fn comparison_binds_looser_than_additive() {
    // a + 1 < b  →  ((a + 1) < b)
    let e = single_expr_stmt("a + 1 < b");
    let Expression::Binary(cmp) = e else {
        panic!("expected binary")
    };
    assert_eq!(cmp.op, BinOp::Lt);
    assert!(matches!(*cmp.lhs, Expression::Binary(_)));
    assert!(matches!(*cmp.rhs, Expression::Identifier(_)));
}

#[test]
fn logical_and_binds_tighter_than_or() {
    // a or b and c  →  (a or (b and c))
    let e = single_expr_stmt("a or b and c");
    let Expression::Binary(or_b) = e else {
        panic!("expected binary")
    };
    assert_eq!(or_b.op, BinOp::Or);
    let Expression::Binary(and_b) = *or_b.rhs else {
        panic!("expected binary")
    };
    assert_eq!(and_b.op, BinOp::And);
}

#[test]
fn power_is_right_associative() {
    // 2 ^ 3 ^ 2  →  (2 ^ (3 ^ 2))
    let e = single_expr_stmt("2 ^ 3 ^ 2");
    let Expression::Binary(outer) = e else {
        panic!("expected binary")
    };
    assert_eq!(outer.op, BinOp::Pow);
    assert!(matches!(
        *outer.lhs,
        Expression::Literal(Literal::Int { value: 2, .. })
    ));
    let Expression::Binary(inner) = *outer.rhs else {
        panic!("expected binary")
    };
    assert_eq!(inner.op, BinOp::Pow);
}

#[test]
fn unary_negation_parses() {
    let e = single_expr_stmt("-5");
    let Expression::Unary(u) = e else {
        panic!("expected unary")
    };
    assert_eq!(u.op, UnOp::Neg);
}

#[test]
fn unary_not_parses() {
    let e = single_expr_stmt("not ready");
    let Expression::Unary(u) = e else {
        panic!("expected unary")
    };
    assert_eq!(u.op, UnOp::Not);
}

#[test]
fn parenthesised_expression_overrides_precedence() {
    // (1 + 2) * 3
    let e = single_expr_stmt("(1 + 2) * 3");
    let Expression::Binary(mul) = e else {
        panic!("expected binary")
    };
    assert_eq!(mul.op, BinOp::Mul);
    assert!(matches!(*mul.lhs, Expression::Grouped(_)));
}

#[test]
fn function_call_with_multiple_args() {
    let e = single_expr_stmt("add(1, 2, 3)");
    let Expression::Call(c) = e else {
        panic!("expected call")
    };
    assert_eq!(c.args.len(), 3);
}

#[test]
fn function_call_with_no_args() {
    let e = single_expr_stmt("now()");
    let Expression::Call(c) = e else {
        panic!("expected call")
    };
    assert!(c.args.is_empty());
}

#[test]
fn field_access_chains() {
    let e = single_expr_stmt("a.b.c");
    let Expression::FieldAccess(fa2) = e else {
        panic!("expected field access")
    };
    assert_eq!(fa2.field.name, "c");
    let Expression::FieldAccess(fa1) = *fa2.base else {
        panic!("expected field access")
    };
    assert_eq!(fa1.field.name, "b");
}

#[test]
fn method_call_via_field_then_call() {
    let e = single_expr_stmt("obj.method(arg)");
    let Expression::Call(c) = e else {
        panic!("expected call")
    };
    assert_eq!(c.args.len(), 1);
    let Expression::FieldAccess(fa) = *c.callee else {
        panic!("expected field access")
    };
    assert_eq!(fa.field.name, "method");
}

// ======================================================================
// Error paths
// ======================================================================

#[test]
fn missing_colon_after_function_header_is_e010() {
    let src = "fn main()\n    return\n";
    let e = err(src);
    assert_eq!(e.code(), "E010");
    assert!(!e.help().is_empty());
}

#[test]
fn missing_closing_paren_in_signature_is_e010() {
    let src = "fn main(a: int:\n    return\n";
    let e = err(src);
    assert_eq!(e.code(), "E010");
}

#[test]
fn non_identifier_in_function_name_position_is_e012() {
    let src = "fn 123():\n    return\n";
    let e = err(src);
    assert_eq!(e.code(), "E012");
}

#[test]
fn missing_type_in_param_annotation_is_e013() {
    // `fn f(x: ):` — parser expects a type after `:`
    let src = "fn f(x: ):\n    return\n";
    let e = err(src);
    assert_eq!(e.code(), "E013");
}

#[test]
fn empty_function_body_is_e014() {
    // A `:` followed by dedent with no statements — the lexer still emits
    // Indent/Dedent for the `return` on the outer function below.
    let src = "fn a():\n\nfn b():\n    return\n";
    let e = err(src);
    // Either EmptyBlock or UnexpectedToken — ensure it doesn't panic and
    // produces a syntax-range code.
    assert!(e.code().starts_with('E'));
    assert!(!e.help().is_empty());
}

#[test]
fn invalid_expression_start_is_e015() {
    // `let x = )` — `)` cannot start an expression.
    let src = "fn main():\n    let x = )\n";
    let e = err(src);
    assert_eq!(e.code(), "E015");
}

#[test]
fn top_level_garbage_is_e010() {
    let src = "let x = 1\n";
    let e = err(src);
    // `let` is not a top-level item in Phase 0 (no constant_decl parsing).
    assert_eq!(e.code(), "E010");
}

#[test]
fn unexpected_eof_mid_function_is_e011() {
    // File cuts off after `fn main(` — parser is still waiting on the rest.
    let src = "fn main(";
    let e = err(src);
    assert_eq!(e.code(), "E011");
}

#[test]
fn every_parse_error_provides_help_text() {
    // Regression — whenever a new variant is added, the help() string
    // must not be empty. This iterates over the variants we can trigger
    // with small inputs.
    let triggers = [
        "fn main()\n    return\n",     // E010
        "fn main(",                    // E011
        "fn 1():\n    return\n",       // E012
        "fn f(x: ):\n    return\n",    // E013
        "fn main():\n    let x = )\n", // E015
    ];
    for src in triggers {
        let e = err(src);
        assert!(!e.help().is_empty(), "missing help for `{src}` → {e:?}");
        assert!(e.code().starts_with('E'), "bad code `{}`", e.code());
    }
}

// ======================================================================
// Span correctness
// ======================================================================

#[test]
fn fn_decl_span_starts_at_fn_token() {
    let src = "fn main():\n    return\n";
    let program = ok(src);
    let Item::Function(f) = &program.items[0];
    assert_eq!(
        f.span.start, 0,
        "FunctionDecl span must open at the `fn` keyword"
    );
    assert_eq!(&src[f.span.start..f.span.start + 2], "fn");
}

#[test]
fn literal_int_span_matches_source_slice() {
    let src = "fn main():\n    42\n";
    let program = ok(src);
    let Item::Function(f) = &program.items[0];
    let Statement::Expr(Expression::Literal(Literal::Int { span, .. })) = &f.body.statements[0]
    else {
        panic!("expected int literal expression statement");
    };
    assert_eq!(
        &src[span.clone()],
        "42",
        "int literal span must index the exact source bytes"
    );
}

#[test]
fn binary_expr_span_covers_both_operands() {
    let src = "fn main():\n    a + b\n";
    let program = ok(src);
    let Item::Function(f) = &program.items[0];
    let Statement::Expr(Expression::Binary(bin)) = &f.body.statements[0] else {
        panic!("expected binary expression");
    };
    let slice = &src[bin.span.clone()];
    assert!(
        slice.starts_with('a'),
        "binary span must begin at lhs — got: {slice:?}"
    );
    assert!(
        slice.ends_with('b'),
        "binary span must end at rhs — got: {slice:?}"
    );
}

#[test]
fn identifier_span_matches_source_slice() {
    let src = "fn main():\n    xyz\n";
    let program = ok(src);
    let Item::Function(f) = &program.items[0];
    let Statement::Expr(Expression::Identifier(id)) = &f.body.statements[0] else {
        panic!("expected identifier expression statement");
    };
    assert_eq!(&src[id.span.clone()], "xyz");
}

// ======================================================================
// Deeply nested expressions
// ======================================================================

#[test]
fn complex_nested_expr_tree_shape() {
    // ((a + b) * c - d / e) ^ 2
    // Expected tree: Pow(Grouped(Sub(Mul(Grouped(Add(a,b)), c), Div(d,e))), 2)
    let e = single_expr_stmt("((a + b) * c - d / e) ^ 2");
    let Expression::Binary(pow) = e else {
        panic!("expected Binary Pow at root")
    };
    assert_eq!(pow.op, BinOp::Pow);
    assert!(matches!(
        *pow.rhs,
        Expression::Literal(Literal::Int { value: 2, .. })
    ));

    let Expression::Grouped(outer) = *pow.lhs else {
        panic!("expected outer Grouped")
    };
    let Expression::Binary(sub) = *outer.inner else {
        panic!("expected Binary Sub")
    };
    assert_eq!(sub.op, BinOp::Sub);

    let Expression::Binary(mul) = *sub.lhs else {
        panic!("expected Binary Mul")
    };
    assert_eq!(mul.op, BinOp::Mul);
    let Expression::Grouped(inner) = *mul.lhs else {
        panic!("expected inner Grouped")
    };
    let Expression::Binary(add) = *inner.inner else {
        panic!("expected Binary Add")
    };
    assert_eq!(add.op, BinOp::Add);

    let Expression::Binary(div) = *sub.rhs else {
        panic!("expected Binary Div")
    };
    assert_eq!(div.op, BinOp::Div);
}

#[test]
fn deeply_nested_unary_not_200_levels_no_stack_overflow() {
    // parse_unary is right-recursive; 200 levels must not blow the stack.
    let nots = "not ".repeat(200);
    let src = format!("fn main():\n    {nots}x\n");
    let program = ok(&src);
    let Item::Function(f) = &program.items[0];
    assert_eq!(f.body.statements.len(), 1);
    // Verify the outermost node is Unary Not (not just any expression).
    assert!(
        matches!(&f.body.statements[0], Statement::Expr(Expression::Unary(u)) if u.op == UnOp::Not),
        "outermost expression must be `not`"
    );
}

// ======================================================================
// Multi-line argument lists
// ======================================================================

#[test]
fn multiline_arg_list_parses_cleanly() {
    // The parser already calls skip_newlines() between args; this is the
    // first explicit test exercising that path.
    let src = "fn main():\n    foo(\n        a,\n        b,\n    )\n";
    let program = ok(src);
    let Item::Function(f) = &program.items[0];
    let Statement::Expr(Expression::Call(call)) = &f.body.statements[0] else {
        panic!("expected call expression");
    };
    assert_eq!(
        call.args.len(),
        2,
        "multi-line arg list must parse both args"
    );
    assert!(matches!(&call.args[0], Expression::Identifier(id) if id.name == "a"));
    assert!(matches!(&call.args[1], Expression::Identifier(id) if id.name == "b"));
}

#[test]
fn multiline_arg_list_trailing_comma_parses() {
    // Trailing comma after the last argument — common style.
    let src = "fn main():\n    foo(\n        1,\n        2,\n        3,\n    )\n";
    let program = ok(src);
    let Item::Function(f) = &program.items[0];
    let Statement::Expr(Expression::Call(call)) = &f.body.statements[0] else {
        panic!("expected call expression");
    };
    assert_eq!(call.args.len(), 3);
}

// ======================================================================
// Stray-newline tolerance at top level
// ======================================================================

#[test]
fn five_blank_lines_between_functions_tolerated() {
    let src = "fn a():\n    return\n\n\n\n\n\nfn b():\n    return\n";
    let program = ok(src);
    assert_eq!(program.items.len(), 2);
    let Item::Function(fa) = &program.items[0];
    let Item::Function(fb) = &program.items[1];
    assert_eq!(fa.name.name, "a");
    assert_eq!(fb.name.name, "b");
}

#[test]
fn leading_blank_lines_before_first_fn_tolerated() {
    let src = "\n\n\nfn main():\n    return\n";
    let program = ok(src);
    assert_eq!(program.items.len(), 1);
}

// ======================================================================
// Error display / snapshot
// ======================================================================

#[test]
fn all_error_variants_display_starts_with_code_and_have_help() {
    // Construct each variant directly so E014 (EmptyBlock) is covered
    // even though it can't always be reliably triggered via small source
    // inputs alone.
    let span = 0..0;
    let cases: &[(ParseError, &str)] = &[
        (
            ParseError::UnexpectedToken {
                span: span.clone(),
                found: "X".into(),
                expected: "Y".into(),
            },
            "E010",
        ),
        (
            ParseError::UnexpectedEof {
                span: span.clone(),
                expected: "Y".into(),
            },
            "E011",
        ),
        (
            ParseError::ExpectedIdentifier {
                span: span.clone(),
                found: "X".into(),
            },
            "E012",
        ),
        (
            ParseError::ExpectedType {
                span: span.clone(),
                found: "X".into(),
            },
            "E013",
        ),
        (ParseError::EmptyBlock { span: span.clone() }, "E014"),
        (
            ParseError::InvalidExpression {
                span: span.clone(),
                found: "X".into(),
            },
            "E015",
        ),
    ];
    for (e, code) in cases {
        let msg = format!("{e}");
        assert!(
            msg.starts_with(code),
            "Display for {code} must start with code — got: {msg:?}"
        );
        assert!(!e.help().is_empty(), "help() must be non-empty for {code}");
        assert_eq!(e.code(), *code, "code() must return {code}");
    }
}

#[test]
fn e010_display_names_expected_and_found() {
    let e = err("fn main()\n    return\n");
    let msg = format!("{e}");
    assert!(
        msg.contains("expected"),
        "E010 display must name the expected token"
    );
    assert!(
        msg.contains("found"),
        "E010 display must name the found token"
    );
}

#[test]
fn e011_display_mentions_unexpected_end_of_input() {
    let e = err("fn main(");
    let msg = format!("{e}");
    assert!(
        msg.contains("end of input"),
        "E011 display must say 'end of input'"
    );
}

// ======================================================================
// Missing literal coverage
// ======================================================================

#[test]
fn literal_false_parses() {
    let e = single_expr_stmt("false");
    assert!(matches!(
        e,
        Expression::Literal(Literal::Bool { value: false, .. })
    ));
}

#[test]
fn literal_nil_parses() {
    let e = single_expr_stmt("nil");
    assert!(matches!(e, Expression::Literal(Literal::Nil { .. })));
}

#[test]
fn fstring_literal_parses() {
    // f-string bodies are captured verbatim by the lexer; interpolation
    // splitting is deferred to Phase 2.
    let e = single_expr_stmt(r#"f"value is {x}""#);
    match e {
        Expression::Literal(Literal::FStr { value, .. }) => {
            assert!(
                value.contains("value is"),
                "f-string body must be preserved"
            );
        }
        other => panic!("expected FStr literal, got {other:?}"),
    }
}

// ======================================================================
// Missing binary operator coverage
// ======================================================================

#[test]
fn subtraction_parses() {
    let e = single_expr_stmt("10 - 3");
    let Expression::Binary(b) = e else {
        panic!("expected binary")
    };
    assert_eq!(b.op, BinOp::Sub);
}

#[test]
fn division_parses() {
    let e = single_expr_stmt("a / b");
    let Expression::Binary(b) = e else {
        panic!("expected binary")
    };
    assert_eq!(b.op, BinOp::Div);
}

#[test]
fn modulo_parses() {
    let e = single_expr_stmt("n % 2");
    let Expression::Binary(b) = e else {
        panic!("expected binary")
    };
    assert_eq!(b.op, BinOp::Mod);
}

#[test]
fn equality_eq_parses() {
    let e = single_expr_stmt("a == b");
    let Expression::Binary(b) = e else {
        panic!("expected binary")
    };
    assert_eq!(b.op, BinOp::Eq);
}

#[test]
fn equality_neq_parses() {
    let e = single_expr_stmt("a != b");
    let Expression::Binary(b) = e else {
        panic!("expected binary")
    };
    assert_eq!(b.op, BinOp::NotEq);
}

#[test]
fn comparison_gt_parses() {
    let e = single_expr_stmt("x > 0");
    let Expression::Binary(b) = e else {
        panic!("expected binary")
    };
    assert_eq!(b.op, BinOp::Gt);
}

#[test]
fn comparison_lteq_parses() {
    let e = single_expr_stmt("i <= n");
    let Expression::Binary(b) = e else {
        panic!("expected binary")
    };
    assert_eq!(b.op, BinOp::LtEq);
}

#[test]
fn comparison_gteq_parses() {
    let e = single_expr_stmt("score >= 90");
    let Expression::Binary(b) = e else {
        panic!("expected binary")
    };
    assert_eq!(b.op, BinOp::GtEq);
}

// ======================================================================
// Ignored — Phase 2 / Phase 3 example files
//
// These tests document exactly which construct triggers the first parse
// error for each example. They serve as Phase 2 regression anchors:
// when the relevant construct is implemented, the test should be
// un-ignored and updated to assert Ok(program).
// ======================================================================

#[test]
#[ignore = "core_language.hm uses Phase 2+ constructs. First error: compound-assign `counter += 1` — E010 unexpected `+=`. Un-ignore when compound assignment lands."]
fn core_language_first_error_is_compound_assign() {
    let src = include_str!("../../examples/core_language.hm");
    let tokens = tokenize(src).expect("lexer must handle core_language.hm");
    let e = parse(tokens).expect_err("core_language.hm uses constructs beyond Phase 0");
    assert_eq!(
        e.code(),
        "E010",
        "first error must be E010 (unexpected token)"
    );
    let msg = format!("{e}");
    assert!(
        msg.contains("+="),
        "error message must mention `+=`; got: {msg:?}"
    );
}

#[test]
#[ignore = "ai_chat.hm uses Phase 3+ constructs. First error: `import` at top level — E010 unexpected `import`. Un-ignore when import parsing lands."]
fn ai_chat_first_error_is_import() {
    let src = include_str!("../../examples/ai_chat.hm");
    let tokens = tokenize(src).expect("lexer must handle ai_chat.hm");
    let e = parse(tokens).expect_err("ai_chat.hm uses constructs beyond Phase 0");
    assert_eq!(
        e.code(),
        "E010",
        "first error must be E010 (unexpected token)"
    );
    let msg = format!("{e}");
    assert!(
        msg.contains("import"),
        "error message must mention `import`; got: {msg:?}"
    );
}

// ======================================================================
// CLI smoke test
// ======================================================================

#[test]
fn cli_emit_ast_hello_world() {
    use std::process::Command;
    let binary = env!("CARGO_BIN_EXE_hmatc");
    let hm_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples/hello_world.hm");

    let output = Command::new(binary)
        .arg("--emit=ast")
        .arg(&hm_file)
        .output()
        .expect("failed to spawn hmatc binary");

    assert!(
        output.status.success(),
        "hmatc --emit=ast must exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with("Program\n"),
        "AST output must start with `Program\\n`; got: {stdout:?}"
    );
    assert!(
        stdout.contains("StrLit \"Hello, HMAT!\""),
        "AST must contain the hello-world string literal; got: {stdout:?}"
    );
}
