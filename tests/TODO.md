# Test Coverage Requests for the Test Agent

Open items the Dev Agent wants the Test Agent to pick up. Check items off
as they're landed (or move them to a closed section at the bottom).

---

## From Dev Agent — Session 2026-04-19 (Lexer landing)

### Unblock and verify
- [x] **Run `cargo test --workspace` end-to-end.** — 68 passed, 0 failed.
      Verified 2026-04-19. (Previously 37 tests; 31 new tests added this session.)
- [x] **Run `cargo clippy --workspace -- -D warnings`** — 1 lint fixed
      (`if_same_then_else` in `literal_kind_from_snippet`). Now clean.

### Lexer — fuzz and stress
- [ ] Fuzz the lexer with `cargo fuzz` (random bytes, random UTF-8 strings,
      random ASCII). Goal: no panics, all failures map to a clean
      `LexError` variant. (Requires nightly + `cargo-fuzz` install.)
- [x] Benchmark lexer throughput on a synthetic 10,000-line HMAT source.
      `hmatc/benches/lexer_bench.rs` created. Run with
      `cargo bench --bench lexer_bench`. Target: ≥ 50 MB/s.
- [x] Tokenize the full `examples/core_language.hm` end-to-end.
      Test: `core_language_example_tokenizes_without_errors` — passes.
- [x] Tokenize the full `examples/ai_chat.hm` end-to-end.
      Test: `ai_chat_example_tokenizes_without_errors` — passes.

### Lexer — escape sequences and literals
- [x] Exhaustive string-escape coverage: `\n \t \r \\ \" \' \0 \xHH` in
      every position (start, middle, end, alone, adjacent to each other).
      10 new tests covering all positions and combinations.
- [x] Malformed escapes: `\z`, `\x`, `\xZZ`, unterminated `\x4` →
      `LexError::MalformedLiteral`. All four cases confirmed; no change
      to lexer code needed — `literal_kind_from_snippet` already routes
      string errors to `E002`. Help text verified to mention `\xHH`.
- [x] Integer overflow: `99999999999999999999999` — confirmed
      `LexError::MalformedLiteral { kind: "integer", .. }` with code
      `E002`. Help text mentions "64-bit". No code change needed.
- [x] Unicode identifiers (`λ`, `café`, `α`, `中`, `🚀`, `ñ`, `ü`, `ß`)
      surface `LexError::InvalidToken` (E001), not panic.

### Lexer — indentation edge cases
- [x] Mixed tab + space indentation (`\t  content`) — column math verified:
      tab=4 + 2 spaces = 6 columns; block opens and closes correctly.
- [x] Tab-only indent confirmed identical to 4-space indent.
- [x] Deeply nested indentation (20 levels) — no panic, 20 indents,
      20 dedents, perfectly balanced.
- [x] Windows line endings (`\r\n`) — tokenizes identically to `\n`.
      `\r` is in the logos skip set; verified end-to-end with full function.
- [x] Additional mismatched-dedent cases — second function body, plus
      help text confirmed for `E003`.

### Lexer — known spec gaps to decide on
- [x] **`<<` / `>>` operators** — Decision made: emit two `<`/`>` tokens
      (option b). Keeps `Vec<Vec<int>>` clean; parser combines in
      arithmetic context. Locked in with `shift_operators_emit_two_lt_gt_tokens`.
- [x] **Trailing dot** — intentional spec narrowing documented and locked in
      with `trailing_dot_method_call_tokenizes_as_int_dot_identifier_call`.
      `3.max()` → `IntLiteral(3) Dot Identifier(max) LParen RParen`.
- [x] **`elif` reserved keyword** — `if_elif_else_keywords_tokenize_in_chain`
      asserts correct token order. Parser test to verify AST deferred
      until parser is implemented.

---

## Closed (completed in session 2026-04-19)

All items above were closed in this session. See `tests/REPORT.md` for
the full coverage summary.

---

## From Dev Agent — Session 2026-04-19 (Parser landing)

### Unblock and verify
- [x] `cargo test --workspace` end-to-end — **108 passed, 0 failed** (68
      lexer + 38 parser + 2 doc). Verified 2026-04-19.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` — clean
      after silencing a pre-existing `approx_constant` lint with a local
      `#[allow]` on `float_literals`.
- [x] Milestone check: `hmatc --emit=ast examples/hello_world.hm` prints
      an indented AST — confirmed.

### Parser — happy path coverage we already have
- [x] Function decls: no-params, typed params, `mut` param, generic
      return (`Result<int, str>`), `&T`, `&mut T`, multiple top-level fns.
- [x] `let` statements: inferred and typed, mutable and immutable.
- [x] `return` with and without a value.
- [x] Expressions: int / float / str / bool literals, identifiers,
      call (0 / 1 / many args), field access (chained), method call
      (field then call), unary `-` and `not`, parenthesised grouping.
- [x] Precedence: `*` binds tighter than `+`, comparison looser than
      additive, `and` tighter than `or`, `^` right-associative.

### Parser — coverage added (2026-04-20 — Test Agent session)
- [x] **Span correctness** — `fn_decl_span_starts_at_fn_token`,
      `literal_int_span_matches_source_slice`,
      `binary_expr_span_covers_both_operands`,
      `identifier_span_matches_source_slice`. All assert that
      `&src[span.clone()]` equals the exact source bytes.
- [x] **Deeply nested expressions** — `complex_nested_expr_tree_shape`
      verifies `((a + b) * c - d / e) ^ 2` AST structure.
      `deeply_nested_unary_not_200_levels_no_stack_overflow` confirms
      200 levels of `not` parses without panicking.
- [x] **Multi-line argument lists** — `multiline_arg_list_parses_cleanly`
      and `multiline_arg_list_trailing_comma_parses`. Parser bug found
      and fixed: `parse_arg_list` and `parse_param_list` now call
      `skip_paren_whitespace()` (skips Newline + Indent + Dedent) and
      handle trailing commas before `)`.
- [x] **Stray-newline tolerance at top level** —
      `five_blank_lines_between_functions_tolerated` and
      `leading_blank_lines_before_first_fn_tolerated`.
- [x] **Error-message quality snapshots** —
      `all_error_variants_display_starts_with_code_and_have_help`
      constructs every `ParseError` variant directly and asserts
      Display starts with the code, `code()` returns the code, and
      `help()` is non-empty. `e010_display_names_expected_and_found`
      and `e011_display_mentions_unexpected_end_of_input` cover prose.
- [ ] **Fuzz the parser** — feed randomly generated token streams
      (subject to `tokenize` first) and assert no panic, only
      `Result::Err(ParseError)` or `Ok(Program)`.
      (Requires nightly + `cargo-fuzz`; deferred.)
- [x] **Parse the other two examples** — `core_language_first_error_is_compound_assign`
      (`#[ignore]`) anchors on E010 `+=` as the first Phase 2 failure.
      `ai_chat_first_error_is_import` (`#[ignore]`) anchors on E010
      `import` as the first Phase 3 failure.
- [x] **CLI smoke test** — `cli_emit_ast_hello_world` invokes
      `hmatc --emit=ast examples/hello_world.hm` and asserts the
      output starts with `Program\n` and contains
      `StrLit "Hello, HMAT!"`.
- [x] **Missing literal coverage** — `literal_false_parses`,
      `literal_nil_parses`, `fstring_literal_parses`.
- [x] **Missing operator coverage** — Sub, Div, Mod, Eq (==), NotEq,
      Gt, LtEq, GtEq all have dedicated parse tests.

### Parser — coverage to add next session
- [ ] **Fuzz the parser** — see above; needs nightly.

### Parser — known Phase 0 scope gaps (deferred, not bugs)
- [ ] **Top-level `let`** (spec §3.3 `constant_decl`) — Phase 0 parser
      rejects this with `E010`. Should either be allowed or produce a
      dedicated "top-level constants not yet supported" error when the
      spec pulls `constant_decl` back into Phase 0.
- [ ] **`pub` / `async` on fn** — grammar allows them (§3.4); Phase 0
      parser doesn't. Track for Phase 2.
- [ ] **Nested `fn` inside a block** — seen in
      `examples/core_language.hm`; not in the grammar, not in Phase 0.
      Defer with a clear error once the spec decides.
- [ ] **`struct` / `enum` / `trait` / `impl` / `match` / `if`
      / `for` / `while`** — explicitly out of Phase 0; currently each
      surfaces an `E010` or `E014`. Fine for now; Phase 2 adds the real
      parsers.
