//! Integration tests for the HMAT lexer.
//!
//! Each test case exercises one axis of the tokenizer. Helpers at the bottom
//! of this file strip spans and collapse to bare [`Token`] variants for
//! concise, readable assertions — spans are only checked explicitly when the
//! test is about span behaviour.

use hmatc::lexer::{tokenize, LexError, Token};

/// Collects just the token variants (drops spans) for easy equality checks.
fn toks(source: &str) -> Vec<Token> {
    tokenize(source)
        .expect("tokenize should succeed")
        .into_iter()
        .map(|(t, _)| t)
        .collect()
}

// ======================================================================
// Empty / trivial inputs
// ======================================================================

#[test]
fn empty_source_produces_no_tokens() {
    assert!(toks("").is_empty());
}

#[test]
fn whitespace_only_source_produces_no_tokens() {
    assert!(toks("   \t \r\n  \n").is_empty());
}

#[test]
fn comment_only_source_produces_no_tokens() {
    assert!(toks("# just a comment\n# and another\n").is_empty());
}

// ======================================================================
// Keywords
// ======================================================================

#[test]
fn every_keyword_lexes_to_its_variant() {
    let cases = [
        ("fn", Token::Fn),
        ("let", Token::Let),
        ("mut", Token::Mut),
        ("return", Token::Return),
        ("if", Token::If),
        ("elif", Token::Elif),
        ("else", Token::Else),
        ("match", Token::Match),
        ("struct", Token::Struct),
        ("enum", Token::Enum),
        ("trait", Token::Trait),
        ("impl", Token::Impl),
        ("type", Token::Type),
        ("pub", Token::Pub),
        ("async", Token::Async),
        ("await", Token::Await),
        ("ai", Token::Ai),
        ("model", Token::Model),
        ("load", Token::Load),
        ("pipeline", Token::Pipeline),
        ("unsafe", Token::Unsafe),
        ("for", Token::For),
        ("in", Token::In),
        ("while", Token::While),
        ("break", Token::Break),
        ("continue", Token::Continue),
        ("import", Token::Import),
        ("from", Token::From),
        ("as", Token::As),
        ("self", Token::SelfKw),
        ("and", Token::And),
        ("or", Token::Or),
        ("not", Token::Not),
        ("is", Token::Is),
        ("where", Token::Where),
        ("true", Token::True),
        ("false", Token::False),
        ("nil", Token::Nil),
    ];
    for (src, expected) in cases {
        let t = toks(src);
        assert_eq!(t.first(), Some(&expected), "input `{src}` failed");
    }
}

#[test]
fn keywords_are_not_prefix_captured_in_identifiers() {
    // `function` starts with `fn` but must lex as a single identifier.
    let t = toks("function fnx letter async_op");
    assert_eq!(
        t,
        vec![
            Token::Identifier("function".into()),
            Token::Identifier("fnx".into()),
            Token::Identifier("letter".into()),
            Token::Identifier("async_op".into()),
            Token::Newline,
        ]
    );
}

// ======================================================================
// Identifiers
// ======================================================================

#[test]
fn identifiers_allow_underscores_and_digits() {
    let t = toks("_foo bar_42 CamelCase SCREAMING_CASE x");
    assert_eq!(
        t,
        vec![
            Token::Identifier("_foo".into()),
            Token::Identifier("bar_42".into()),
            Token::Identifier("CamelCase".into()),
            Token::Identifier("SCREAMING_CASE".into()),
            Token::Identifier("x".into()),
            Token::Newline,
        ]
    );
}

// ======================================================================
// Numeric literals
// ======================================================================

#[test]
fn decimal_integer_literals() {
    assert_eq!(toks("0 1 42 1000"), vec![
        Token::IntLiteral(0),
        Token::IntLiteral(1),
        Token::IntLiteral(42),
        Token::IntLiteral(1000),
        Token::Newline,
    ]);
}

#[test]
fn integer_literals_with_underscores() {
    assert_eq!(toks("1_000_000"), vec![Token::IntLiteral(1_000_000), Token::Newline]);
}

#[test]
fn hex_binary_octal_integer_literals() {
    assert_eq!(
        toks("0xFF 0xff 0x1_0 0b1010 0b1_1 0o17"),
        vec![
            Token::IntLiteral(0xFF),
            Token::IntLiteral(0xff),
            Token::IntLiteral(0x10),
            Token::IntLiteral(0b1010),
            Token::IntLiteral(0b11),
            Token::IntLiteral(0o17),
            Token::Newline,
        ]
    );
}

#[test]
fn float_literals() {
    let t = toks("3.14 0.0 1_000.5");
    assert_eq!(t.len(), 4);
    if let Token::FloatLiteral(f) = t[0] {
        assert!((f - 3.14).abs() < 1e-9);
    } else {
        panic!("expected float, got {:?}", t[0]);
    }
}

#[test]
fn float_with_exponent() {
    let t = toks("1.5e10 2.0E-3");
    assert_eq!(t.len(), 3);
}

#[test]
fn trailing_dot_is_int_then_dot_not_float() {
    // Required so that `3.max()` can parse as method call.
    let t = toks("3.max");
    assert_eq!(
        t,
        vec![
            Token::IntLiteral(3),
            Token::Dot,
            Token::Identifier("max".into()),
            Token::Newline,
        ]
    );
}

#[test]
fn range_operator_does_not_swallow_digits() {
    let t = toks("0..10");
    assert_eq!(
        t,
        vec![
            Token::IntLiteral(0),
            Token::DotDot,
            Token::IntLiteral(10),
            Token::Newline,
        ]
    );
}

#[test]
fn inclusive_range_tokenizes() {
    let t = toks("0..=10");
    assert_eq!(
        t,
        vec![
            Token::IntLiteral(0),
            Token::DotDotEq,
            Token::IntLiteral(10),
            Token::Newline,
        ]
    );
}

// ======================================================================
// String literals
// ======================================================================

#[test]
fn basic_string_literal() {
    assert_eq!(
        toks(r#""hello""#),
        vec![Token::StringLiteral("hello".into()), Token::Newline]
    );
}

#[test]
fn string_escapes_are_decoded() {
    // Source is `"line1\nline2\ttab\\slash\"quote"`.
    let src = r#""line1\nline2\ttab\\slash\"quote""#;
    let t = toks(src);
    assert_eq!(t.len(), 2);
    if let Token::StringLiteral(s) = &t[0] {
        assert_eq!(s, "line1\nline2\ttab\\slash\"quote");
    } else {
        panic!("expected string");
    }
}

#[test]
fn hex_escape_decodes_to_byte() {
    let src = r#""\x41\x42""#;
    let t = toks(src);
    if let Token::StringLiteral(s) = &t[0] {
        assert_eq!(s, "AB");
    } else {
        panic!("expected string");
    }
}

#[test]
fn empty_string_is_valid() {
    assert_eq!(
        toks(r#""""#),
        vec![Token::StringLiteral(String::new()), Token::Newline]
    );
}

#[test]
fn f_string_preserves_interior_for_parser() {
    // The lexer keeps the body intact minus escapes; the parser walks the
    // string later to split text and {expression} segments.
    let src = r#"f"Hello, {name}!""#;
    let t = toks(src);
    assert_eq!(t.len(), 2);
    if let Token::FString(body) = &t[0] {
        assert_eq!(body, "Hello, {name}!");
    } else {
        panic!("expected FString, got {:?}", t[0]);
    }
}

// ======================================================================
// Operators and delimiters
// ======================================================================

#[test]
fn all_arithmetic_operators() {
    let t = toks("+ - * / % ^");
    assert_eq!(
        t,
        vec![
            Token::Plus,
            Token::Minus,
            Token::Star,
            Token::Slash,
            Token::Percent,
            Token::Caret,
            Token::Newline,
        ]
    );
}

#[test]
fn comparison_operators_prefer_longest_match() {
    let t = toks("== != < <= > >= = =>");
    assert_eq!(
        t,
        vec![
            Token::EqEq,
            Token::NotEq,
            Token::Lt,
            Token::LtEq,
            Token::Gt,
            Token::GtEq,
            Token::Eq,
            Token::FatArrow,
            Token::Newline,
        ]
    );
}

#[test]
fn assignment_compound_operators() {
    let t = toks("+= -= *= /= %=");
    assert_eq!(
        t,
        vec![
            Token::PlusEq,
            Token::MinusEq,
            Token::StarEq,
            Token::SlashEq,
            Token::PercentEq,
            Token::Newline,
        ]
    );
}

#[test]
fn arrows_and_question_and_sigils() {
    let t = toks("-> => ? & | ! ~ @");
    assert_eq!(
        t,
        vec![
            Token::Arrow,
            Token::FatArrow,
            Token::Question,
            Token::Ampersand,
            Token::Pipe,
            Token::Bang,
            Token::Tilde,
            Token::At,
            Token::Newline,
        ]
    );
}

#[test]
fn all_delimiters() {
    let t = toks("( ) { } [ ] : :: , . .. ..=");
    assert_eq!(
        t,
        vec![
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::RBrace,
            Token::LBracket,
            Token::RBracket,
            Token::Colon,
            Token::ColonColon,
            Token::Comma,
            Token::Dot,
            Token::DotDot,
            Token::DotDotEq,
            Token::Newline,
        ]
    );
}

// ======================================================================
// Comments
// ======================================================================

#[test]
fn inline_comment_is_stripped() {
    let t = toks("let x = 1  # trailing comment\n");
    assert_eq!(
        t,
        vec![
            Token::Let,
            Token::Identifier("x".into()),
            Token::Eq,
            Token::IntLiteral(1),
            Token::Newline,
        ]
    );
}

// ======================================================================
// Indentation
// ======================================================================

#[test]
fn simple_function_body_injects_indent_and_dedent() {
    let src = "fn main():\n    let x = 1\n";
    let t = toks(src);
    assert_eq!(
        t,
        vec![
            Token::Fn,
            Token::Identifier("main".into()),
            Token::LParen,
            Token::RParen,
            Token::Colon,
            Token::Newline,
            Token::Indent,
            Token::Let,
            Token::Identifier("x".into()),
            Token::Eq,
            Token::IntLiteral(1),
            Token::Newline,
            Token::Dedent,
        ]
    );
}

#[test]
fn nested_blocks_produce_stacked_indent_dedent() {
    let src = "\
fn outer():
    if x:
        y = 1
";
    let t = toks(src);
    let indent_count = t.iter().filter(|x| matches!(x, Token::Indent)).count();
    let dedent_count = t.iter().filter(|x| matches!(x, Token::Dedent)).count();
    assert_eq!(indent_count, 2, "should have two indent levels");
    assert_eq!(dedent_count, 2, "every indent must be closed");
}

#[test]
fn dedent_to_outer_level() {
    let src = "\
fn a():
    let x = 1
fn b():
    let y = 2
";
    let t = toks(src);
    // Expect two Indent/Dedent pairs bracketing the two function bodies.
    let indents = t.iter().filter(|x| matches!(x, Token::Indent)).count();
    let dedents = t.iter().filter(|x| matches!(x, Token::Dedent)).count();
    assert_eq!(indents, 2);
    assert_eq!(dedents, 2);
}

#[test]
fn blank_and_comment_lines_do_not_affect_indentation() {
    let src = "\
fn main():
    let x = 1

    # a comment
    let y = 2
";
    let t = toks(src);
    // Exactly one indent, one dedent — blank and comment lines don't count.
    assert_eq!(t.iter().filter(|x| matches!(x, Token::Indent)).count(), 1);
    assert_eq!(t.iter().filter(|x| matches!(x, Token::Dedent)).count(), 1);
}

#[test]
fn mismatched_dedent_is_an_error() {
    // The inner line is indented by 6 spaces (between 4 and 8) — there is no
    // indent level of 6 to return to, so this is an error if the next line
    // dedents to an unseen level.
    let src = "\
fn main():
    if x:
        let y = 1
      let z = 2
";
    let err = tokenize(src).expect_err("should fail on inconsistent indent");
    assert!(matches!(err, LexError::IndentMismatch { .. }));
}

#[test]
fn tabs_count_as_four_columns() {
    // Tab-indented body should behave just like 4-space indent.
    let src = "fn main():\n\tlet x = 1\n";
    let t = toks(src);
    assert!(t.contains(&Token::Indent));
    assert!(t.contains(&Token::Dedent));
}

#[test]
fn eof_without_trailing_newline_closes_indents() {
    let src = "fn main():\n    let x = 1";
    let t = toks(src);
    // Must see a final Newline + Dedent even though the source ends mid-line.
    assert!(matches!(t.last(), Some(Token::Dedent)));
}

// ======================================================================
// Spans
// ======================================================================

#[test]
fn spans_point_to_the_exact_source_bytes() {
    let src = "let x = 42";
    let tokens = tokenize(src).unwrap();
    // `let` is bytes 0..3; `x` is 4..5; `=` is 6..7; `42` is 8..10.
    assert_eq!(tokens[0].1, 0..3);
    assert_eq!(tokens[1].1, 4..5);
    assert_eq!(tokens[2].1, 6..7);
    assert_eq!(tokens[3].1, 8..10);
}

// ======================================================================
// Errors
// ======================================================================

#[test]
fn invalid_character_produces_lex_error() {
    let err = tokenize("let x = $").expect_err("$ is not a valid token");
    assert!(matches!(err, LexError::InvalidToken { .. }));
    assert_eq!(err.code(), "E001");
}

// ======================================================================
// Real-world-ish sources
// ======================================================================

#[test]
fn hello_world_tokenizes_end_to_end() {
    let src = "fn main():\n    print(\"Hello, HMAT!\")\n";
    let t = toks(src);
    assert_eq!(t[0], Token::Fn);
    assert_eq!(t[1], Token::Identifier("main".into()));
    assert!(t.contains(&Token::Indent));
    assert!(t.contains(&Token::Dedent));
    assert!(t
        .iter()
        .any(|x| matches!(x, Token::StringLiteral(s) if s == "Hello, HMAT!")));
}

#[test]
fn ai_model_decl_tokenizes() {
    let src = r#"ai model assistant = load("anthropic/claude-3-5-sonnet")"#;
    let t = toks(src);
    assert_eq!(t[0], Token::Ai);
    assert_eq!(t[1], Token::Model);
    assert_eq!(t[2], Token::Identifier("assistant".into()));
    assert_eq!(t[3], Token::Eq);
    assert_eq!(t[4], Token::Load);
    assert_eq!(t[5], Token::LParen);
    assert_eq!(
        t[6],
        Token::StringLiteral("anthropic/claude-3-5-sonnet".into())
    );
    assert_eq!(t[7], Token::RParen);
}

#[test]
fn error_propagation_operator_tokenizes() {
    let src = "let x = foo()?";
    let t = toks(src);
    assert!(t.contains(&Token::Question));
}
