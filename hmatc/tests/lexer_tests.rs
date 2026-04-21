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
        ("on", Token::On),
        ("shape", Token::Shape),
        ("type", Token::Type),
        ("flow", Token::Flow),
        ("fail", Token::Fail),
        ("pub", Token::Pub),
        ("async", Token::Async),
        ("await", Token::Await),
        ("ai", Token::Ai),
        ("model", Token::Model),
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
    assert_eq!(
        toks("0 1 42 1000"),
        vec![
            Token::IntLiteral(0),
            Token::IntLiteral(1),
            Token::IntLiteral(42),
            Token::IntLiteral(1000),
            Token::Newline,
        ]
    );
}

#[test]
fn integer_literals_with_underscores() {
    assert_eq!(
        toks("1_000_000"),
        vec![Token::IntLiteral(1_000_000), Token::Newline]
    );
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
#[allow(clippy::approx_constant)] // 3.14 here is a literal under test, not an approximation of π.
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
fn arrows_and_sigils() {
    let t = toks("-> => & | ! ~ @");
    assert_eq!(
        t,
        vec![
            Token::Arrow,
            Token::FatArrow,
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
    // Spec v0.3 §3.8: `ai name = model("provider/model"): ...`
    let src = r#"ai assistant = model("anthropic/claude-3-5-sonnet")"#;
    let t = toks(src);
    assert_eq!(t[0], Token::Ai);
    assert_eq!(t[1], Token::Identifier("assistant".into()));
    assert_eq!(t[2], Token::Eq);
    assert_eq!(t[3], Token::Model);
    assert_eq!(t[4], Token::LParen);
    assert_eq!(
        t[5],
        Token::StringLiteral("anthropic/claude-3-5-sonnet".into())
    );
    assert_eq!(t[6], Token::RParen);
}

#[test]
fn fail_keyword_tokenizes() {
    // Spec v0.3 §4: `fail "msg"` is the raise-fail statement.
    let src = r#"fail "oops""#;
    let t = toks(src);
    assert_eq!(t[0], Token::Fail);
    assert_eq!(t[1], Token::StringLiteral("oops".into()));
}

#[test]
fn shape_on_flow_keywords_tokenize() {
    // Spec v0.3 §3.4 / §3.6 / §3.7.
    let t = toks("shape on flow");
    assert_eq!(t[0], Token::Shape);
    assert_eq!(t[1], Token::On);
    assert_eq!(t[2], Token::Flow);
}

// ======================================================================
// Escape sequences — exhaustive coverage
// ======================================================================

#[test]
fn all_standard_escapes_in_solo_strings() {
    // Each standard escape, alone in a string, decodes to the right char.
    let cases: &[(&str, &str)] = &[
        (r#""\n""#, "\n"),
        (r#""\t""#, "\t"),
        (r#""\r""#, "\r"),
        (r#""\\""#, "\\"),
        (r#""\"""#, "\""),
        (r#""\"""#, "\""),
        (r#""\0""#, "\0"),
    ];
    for (src, expected) in cases {
        let t = toks(src);
        if let Token::StringLiteral(s) = &t[0] {
            assert_eq!(s.as_str(), *expected, "escape in solo string, input: {src}");
        } else {
            panic!("expected StringLiteral, got {:?} for input {src}", t[0]);
        }
    }
}

#[test]
fn escape_at_start_of_string() {
    let t = toks(r#""\nhello""#);
    if let Token::StringLiteral(s) = &t[0] {
        assert_eq!(s, "\nhello");
    } else {
        panic!("expected StringLiteral");
    }
}

#[test]
fn escape_at_end_of_string() {
    let t = toks(r#""hello\n""#);
    if let Token::StringLiteral(s) = &t[0] {
        assert_eq!(s, "hello\n");
    } else {
        panic!("expected StringLiteral");
    }
}

#[test]
fn escape_in_middle_of_string() {
    let t = toks(r#""hel\nlo""#);
    if let Token::StringLiteral(s) = &t[0] {
        assert_eq!(s, "hel\nlo");
    } else {
        panic!("expected StringLiteral");
    }
}

#[test]
fn adjacent_escapes_all_decoded() {
    let t = toks(r#""\n\t\r\\\"\0""#);
    if let Token::StringLiteral(s) = &t[0] {
        assert_eq!(s, "\n\t\r\\\"\0");
    } else {
        panic!("expected StringLiteral");
    }
}

#[test]
fn hex_escape_at_start_of_string() {
    let t = toks(r#""\x41BC""#);
    if let Token::StringLiteral(s) = &t[0] {
        assert_eq!(s, "ABC");
    } else {
        panic!("expected StringLiteral");
    }
}

#[test]
fn hex_escape_at_end_of_string() {
    let t = toks(r#""AB\x43""#);
    if let Token::StringLiteral(s) = &t[0] {
        assert_eq!(s, "ABC");
    } else {
        panic!("expected StringLiteral");
    }
}

#[test]
fn adjacent_hex_escapes() {
    // \x41\x42\x43 == "ABC"
    let t = toks(r#""\x41\x42\x43""#);
    if let Token::StringLiteral(s) = &t[0] {
        assert_eq!(s, "ABC");
    } else {
        panic!("expected StringLiteral");
    }
}

#[test]
fn hex_escape_lowercase_digits() {
    // \x61 == 'a'
    let t = toks(r#""\x61""#);
    if let Token::StringLiteral(s) = &t[0] {
        assert_eq!(s, "a");
    } else {
        panic!("expected StringLiteral");
    }
}

// ======================================================================
// Malformed escape sequences and literals
// ======================================================================

#[test]
fn invalid_escape_z_produces_malformed_literal() {
    let err = tokenize(r#""\z""#).expect_err("\\z is not a valid escape");
    assert!(matches!(err, LexError::MalformedLiteral { .. }));
    assert_eq!(err.code(), "E002");
}

#[test]
fn incomplete_hex_escape_no_digits_produces_malformed_literal() {
    // \x with no hex digits following — string ends immediately after \x
    let err = tokenize(r#""\x""#).expect_err("\\x with no digits is malformed");
    assert!(matches!(err, LexError::MalformedLiteral { .. }));
    assert_eq!(err.code(), "E002");
}

#[test]
fn incomplete_hex_escape_one_digit_produces_malformed_literal() {
    // \x4 — only one hex digit, second is end-of-string
    let err = tokenize(r#""\x4""#).expect_err("\\x4 with only one digit is malformed");
    assert!(matches!(err, LexError::MalformedLiteral { .. }));
    assert_eq!(err.code(), "E002");
}

#[test]
fn hex_escape_with_non_hex_chars_produces_malformed_literal() {
    // \xZZ — Z is not a valid hex digit
    let err = tokenize(r#""\xZZ""#).expect_err("\\xZZ has invalid hex chars");
    assert!(matches!(err, LexError::MalformedLiteral { .. }));
    assert_eq!(err.code(), "E002");
}

#[test]
fn malformed_escape_error_has_help_text() {
    let err = tokenize(r#""\z""#).expect_err("should fail");
    let help = err.help();
    assert!(!help.is_empty(), "error must always include a help message");
    assert!(
        help.contains("\\xHH"),
        "help should mention valid escapes, got: {help}"
    );
}

// ======================================================================
// Integer overflow
// ======================================================================

#[test]
fn integer_overflow_produces_malformed_literal() {
    // 23 nines — far exceeds i64::MAX (~9.2e18)
    let err = tokenize("99999999999999999999999").expect_err("overflow should error");
    assert!(
        matches!(
            err,
            LexError::MalformedLiteral {
                kind: "integer",
                ..
            }
        ),
        "expected MalformedLiteral(integer), got: {err:?}"
    );
    assert_eq!(err.code(), "E002");
}

#[test]
fn integer_overflow_error_has_help_text() {
    let err = tokenize("99999999999999999999999").expect_err("should fail");
    let help = err.help();
    assert!(
        help.contains("64-bit") || help.contains("overflow"),
        "help should mention 64-bit overflow, got: {help}"
    );
}

// ======================================================================
// Unicode / non-ASCII identifiers
// ======================================================================

#[test]
fn non_ascii_char_lambda_produces_invalid_token() {
    // Spec §2.4: identifiers are ASCII only in v0.2
    let err = tokenize("λ").expect_err("λ is not valid ASCII");
    assert!(matches!(err, LexError::InvalidToken { .. }));
    assert_eq!(err.code(), "E001");
}

#[test]
fn ascii_prefix_then_non_ascii_produces_invalid_token() {
    // "caf" lexes as identifier, then "é" is an unknown character
    let err = tokenize("café").expect_err("é is not valid HMAT");
    assert!(matches!(err, LexError::InvalidToken { .. }));
    assert_eq!(err.code(), "E001");
}

#[test]
fn non_ascii_identifier_does_not_panic() {
    // Ensures the lexer fails gracefully (no panic, no stack overflow)
    for s in ["α", "中", "🚀", "ñ", "ü", "ß"] {
        let result = tokenize(s);
        assert!(result.is_err(), "non-ASCII `{s}` must fail, not succeed");
        assert!(!matches!(
            result.unwrap_err(),
            LexError::IndentMismatch { .. }
        ));
    }
}

// ======================================================================
// Indentation edge cases
// ======================================================================

#[test]
fn mixed_tab_space_indent_uses_four_column_tab() {
    // \t + 2 spaces = 4 + 2 = 6 columns — lexer accepts it (hfmt normalises)
    let src = "fn main():\n\t  let x = 1\n";
    let t = toks(src);
    assert!(
        t.contains(&Token::Indent),
        "6-column indent should open a block"
    );
    assert!(t.contains(&Token::Dedent), "block must be closed");
}

#[test]
fn tab_only_indent_matches_four_space_indent() {
    // Single tab == 4 spaces for indent purposes
    let tab_src = "fn main():\n\tlet x = 1\n";
    let space_src = "fn main():\n    let x = 1\n";
    assert_eq!(
        toks(tab_src),
        toks(space_src),
        "tab indent must tokenize identically to 4-space indent"
    );
}

#[test]
fn deeply_nested_indentation_20_levels_no_panic() {
    // 20 nested if-blocks — no stack overflow, balanced indent/dedent
    let mut src = String::new();
    for depth in 0..20 {
        src.push_str(&format!("{}if x:\n", "    ".repeat(depth)));
    }
    src.push_str(&format!("{}let v = 1\n", "    ".repeat(20)));

    let t = toks(&src);
    let indent_count = t.iter().filter(|x| matches!(x, Token::Indent)).count();
    let dedent_count = t.iter().filter(|x| matches!(x, Token::Dedent)).count();
    assert_eq!(indent_count, 20, "should open 20 indent levels");
    assert_eq!(dedent_count, 20, "every indent must be closed");
}

#[test]
fn windows_crlf_line_endings_tokenize_correctly() {
    let crlf = "fn main():\r\n    let x = 1\r\n";
    let lf = "fn main():\n    let x = 1\n";
    assert_eq!(
        toks(crlf),
        toks(lf),
        "\\r\\n should produce the same tokens as \\n"
    );
}

#[test]
fn crlf_only_source_produces_no_tokens() {
    assert!(toks("\r\n\r\n").is_empty());
}

#[test]
fn mismatched_dedent_in_second_block() {
    // Two functions; the second has a bad dedent within its body
    let src = "\
fn a():
    let x = 1
fn b():
    if y:
        let z = 2
      let w = 3
";
    let err = tokenize(src).expect_err("bad dedent inside b() must error");
    assert!(matches!(err, LexError::IndentMismatch { .. }));
    assert_eq!(err.code(), "E003");
}

#[test]
fn mismatched_dedent_error_has_help_text() {
    let src = "fn main():\n    if x:\n        let y = 1\n      let z = 2\n";
    let err = tokenize(src).expect_err("should fail");
    let help = err.help();
    assert!(
        help.contains("4 spaces") || help.contains("indent"),
        "help should guide the user, got: {help}"
    );
}

// ======================================================================
// Spec gap decisions
// ======================================================================

// Decision: `<<` / `>>` are NOT tokenized as single tokens in v0.2.
// They emit two `<`/`>` tokens so `Vec<Vec<int>>` can close cleanly.
// The parser combines them in arithmetic contexts if needed.
// This test documents and locks in that decision.
#[test]
fn shift_operators_emit_two_lt_gt_tokens() {
    let t = toks("a << b");
    assert_eq!(
        t,
        vec![
            Token::Identifier("a".into()),
            Token::Lt,
            Token::Lt,
            Token::Identifier("b".into()),
            Token::Newline,
        ],
        "<<  must lex as two Lt tokens in v0.2"
    );

    let t2 = toks("a >> b");
    assert_eq!(
        t2,
        vec![
            Token::Identifier("a".into()),
            Token::Gt,
            Token::Gt,
            Token::Identifier("b".into()),
            Token::Newline,
        ],
        ">> must lex as two Gt tokens in v0.2"
    );
}

// Decision: `3.` is NOT a float literal — trailing dot tokenizes as Dot.
// This is an intentional narrowing of the spec grammar so that `3.max()`
// can be parsed as a method call (IntLiteral → Dot → Identifier).
// Already tested in `trailing_dot_is_int_then_dot_not_float`; this test
// adds the full method-call form and documents the decision explicitly.
#[test]
fn trailing_dot_method_call_tokenizes_as_int_dot_identifier_call() {
    let t = toks("3.max()");
    assert_eq!(
        t,
        vec![
            Token::IntLiteral(3),
            Token::Dot,
            Token::Identifier("max".into()),
            Token::LParen,
            Token::RParen,
            Token::Newline,
        ],
        "3.max() must not treat `3.` as a float — intentional spec narrowing"
    );
}

// `elif` is a reserved keyword; test a full if/elif/else keyword chain.
// Parser tests will verify AST structure once the parser is implemented.
#[test]
fn if_elif_else_keywords_tokenize_in_chain() {
    let src = "if x:\n    a\nelif y:\n    b\nelse:\n    c\n";
    let t = toks(src);
    assert!(t.contains(&Token::If));
    assert!(t.contains(&Token::Elif));
    assert!(t.contains(&Token::Else));
    // The keywords must appear in order
    let kw_positions: Vec<_> = t
        .iter()
        .enumerate()
        .filter(|(_, tok)| matches!(tok, Token::If | Token::Elif | Token::Else))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(kw_positions.len(), 3, "must find if, elif, else");
    assert!(
        kw_positions[0] < kw_positions[1] && kw_positions[1] < kw_positions[2],
        "if must precede elif which must precede else"
    );
}

// ======================================================================
// Example files — end-to-end tokenization
// ======================================================================

#[test]
fn core_language_example_tokenizes_without_errors() {
    let src = include_str!("../../examples/core_language.hm");
    let result = tokenize(src);
    assert!(
        result.is_ok(),
        "core_language.hm must lex cleanly, got: {:?}",
        result.unwrap_err()
    );
    let tokens = result.unwrap();
    // Spot-check: must contain fn, type, for, or keywords (core_language.hm
    // uses HMAT-native `shape`/`type`/`on`; `struct`/`enum`/`match` are
    // reserved but absent from this example).
    let has = |tok: &Token| tokens.iter().any(|(t, _)| t == tok);
    assert!(has(&Token::Fn), "must contain `fn`");
    assert!(has(&Token::Type), "must contain `type`");
    assert!(has(&Token::For), "must contain `for`");
    assert!(has(&Token::Or), "must contain `or`");
    // Balanced indent/dedent
    let indents = tokens
        .iter()
        .filter(|(t, _)| matches!(t, Token::Indent))
        .count();
    let dedents = tokens
        .iter()
        .filter(|(t, _)| matches!(t, Token::Dedent))
        .count();
    assert_eq!(
        indents, dedents,
        "core_language.hm indent/dedent must balance"
    );
}

#[test]
fn ai_chat_example_tokenizes_without_errors() {
    let src = include_str!("../../examples/ai_chat.hm");
    let result = tokenize(src);
    assert!(
        result.is_ok(),
        "ai_chat.hm must lex cleanly, got: {:?}",
        result.unwrap_err()
    );
    let tokens = result.unwrap();
    // Spot-check: must contain ai, model, from, async, await keywords
    // (`load` is a reserved token but is not used in this example).
    let has = |tok: &Token| tokens.iter().any(|(t, _)| t == tok);
    assert!(has(&Token::Ai), "must contain `ai`");
    assert!(has(&Token::Model), "must contain `model`");
    assert!(has(&Token::From), "must contain `from`");
    assert!(has(&Token::Async), "must contain `async`");
    assert!(has(&Token::Await), "must contain `await`");
    // Balanced indent/dedent
    let indents = tokens
        .iter()
        .filter(|(t, _)| matches!(t, Token::Indent))
        .count();
    let dedents = tokens
        .iter()
        .filter(|(t, _)| matches!(t, Token::Dedent))
        .count();
    assert_eq!(indents, dedents, "ai_chat.hm indent/dedent must balance");
}
