# Test Coverage Requests for the Test Agent

Open items the Dev Agent wants the Test Agent to pick up. Check items off
as they're landed (or move them to a closed section at the bottom).

---

## From Dev Agent — Session 2026-04-19 (Lexer landing)

### Unblock and verify
- [ ] **Run `cargo test --workspace` end-to-end.** The author's machine is
      missing the Windows SDK libs (`kernel32.lib` etc.) so the current
      session could not verify the test suite. After installing Visual
      Studio Build Tools with the "Desktop development with C++" workload,
      run the suite and report any failures.
- [ ] **Run `cargo clippy --workspace -- -D warnings`** and open issues for
      any lints we want to accept as style rules.

### Lexer — fuzz and stress
- [ ] Fuzz the lexer with `cargo fuzz` (random bytes, random UTF-8 strings,
      random ASCII). Goal: no panics, all failures map to a clean
      `LexError` variant.
- [ ] Benchmark lexer throughput on a synthetic 10,000-line HMAT source.
      Target: ≥ 50 MB/s on a modern laptop. Track regressions in a
      `benches/` harness.
- [ ] Tokenize the full `examples/core_language.hm` end-to-end (no errors,
      sensible token count).
- [ ] Tokenize the full `examples/ai_chat.hm` end-to-end.

### Lexer — escape sequences and literals
- [ ] Exhaustive string-escape coverage: `\n \t \r \\ \" \' \0 \xHH` in
      every position (start, middle, end, alone, adjacent to each other).
- [ ] Malformed escapes: `\z`, `\x`, `\xZZ`, unterminated `\x4` →
      `LexError::MalformedLiteral` (currently surfaces as a generic error;
      the lexer may need a `MalformedEscape` distinction).
- [ ] Integer overflow: `99999999999999999999999` — today this returns
      `None` from `parse()` and the logos rule errors as `InvalidToken`
      without a `MalformedLiteral` context. Verify that error message is
      friendly, or add an explicit overflow check in `parse_dec_int`.
- [ ] Unicode identifiers (e.g. `λ`, `café`). Spec §2.4 says ASCII only in
      v0.2, so these should surface `LexError::InvalidToken`, not panic.

### Lexer — indentation edge cases
- [ ] Mixed tab + space indentation on the same line
      (e.g. `\t  content`) — verify the column math matches the "tab = 4
      cols" rule and that `hfmt` can later flag it.
- [ ] Deeply nested indentation (20 levels). No stack overflow, all indents
      matched by dedents.
- [ ] Windows line endings (`\r\n`) on every line. The current lexer skips
      `\r` as whitespace and treats `\n` as the newline token; verify this
      is correct end-to-end.
- [ ] Dedent to a level that was never opened from the middle of a block
      (current test only covers one placement; add more).

### Lexer — known spec gaps to decide on
- [ ] Spec §2.6 lists `<<` and `>>` as operators (bitwise shift). They are
      **not yet tokenized** because `>>` would conflict with generic
      closings like `Vec<Vec<int>>`. Decision pending — either (a) tokenize
      as `LShift`/`RShift` and have the parser split in type position (Rust
      approach), or (b) keep emitting two `<`/`>` tokens and let the parser
      combine. Please write a couple of representative tests for whichever
      choice is made.
- [ ] Spec §2.5 allows `FLOAT_LITERAL ::= DIGIT { DIGIT } '.' { DIGIT }`
      which permits `3.` as a float. The current lexer requires a digit
      after the dot so that `3.max()` parses as a method call. Please
      record this as an intentional narrowing and add a test asserting
      `3.max()` tokenizes as `IntLiteral(3) Dot Identifier`.
- [ ] `elif` is accepted by the lexer but §2.3 also lists it as reserved.
      Add a test that exercises a full `if/elif/else` chain once the parser
      exists.
