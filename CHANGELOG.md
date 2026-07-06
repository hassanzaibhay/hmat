# CHANGELOG

All notable changes to the HMAT programming language are documented here.

Format: `[Unreleased]` for work in progress, `[vX.Y.Z - YYYY-MM-DD]` for releases.  
This file is updated after every development session.

---

## [Unreleased]

### Status
Phase 1 — Hello World Compiles. C codegen via clang landed. `hmatc examples/hello_world.hm` produces a native binary.

### Added (2026-04-19 — Parser session)
- `hmatc/src/ast/mod.rs` — AST node types for Phase 0: `Program`, `Item`,
  `FunctionDecl`, `Param`, `TypeRef`, `Block`, `Statement`, `LetStmt`,
  `ReturnStmt`, `Identifier`, `Expression` (Literal / Identifier / Call /
  FieldAccess / Binary / Unary / Grouped), `Literal` (Int / Float / Str /
  FStr / Bool / Nil), `BinOp`, `UnOp`. Every node carries a byte span.
- `hmatc/src/ast/mod.rs` — indent-based AST pretty printer
  (`Program::pretty_print()`), consumed by the `--emit=ast` CLI.
- `hmatc/src/parser/mod.rs` — hand-rolled recursive descent parser over
  `Vec<(Token, Span)>`. Recognises: top-level `fn` declarations, optional
  return types (named / generic / `&T` / `&mut T`), parameters (`[mut] name: ty`),
  `let` / `return` / expression statements inside `INDENT ... DEDENT` blocks,
  full precedence climb from `or` → `and` → `==`/`!=` → `<`/`<=`/`>`/`>=` →
  `+`/`-` → `*`/`/`/`%` → unary `-`/`not` → right-assoc `^` → postfix
  (`.field`, `(args)`) → primary (literals, identifiers, `( expr )`).
  Newlines are skipped inside `(...)` so multi-line arg/param lists parse.
- `ParseError` with codes `E010`–`E015`: `UnexpectedToken`, `UnexpectedEof`,
  `ExpectedIdentifier`, `ExpectedType`, `EmptyBlock`, `InvalidExpression`.
  Every variant has a `help()` message — Zen #2: errors are teachers.
- `CompilerError::Parse` variant wires `ParseError` into the driver error
  path.
- `hmatc --emit=ast <file>` implemented — runs lexer + parser and prints
  an indented AST tree.
- `hmatc/tests/parser_tests.rs` — 38 integration tests covering: the
  `hello_world.hm` milestone (parse + pretty-print), function
  declarations (no-params / typed / mut params / generic return / `&T` /
  `&mut T` / multiple top-level fns), let & return statement shapes,
  literal / identifier / call / field-access / method-chain / unary /
  grouped expression parses, precedence (`*` tighter than `+`, comparison
  looser than additive, `and` tighter than `or`, right-assoc `^`), and
  six error cases each asserting a specific `E0xx` code plus non-empty
  `help()` text.

### Verified (2026-04-19)
- `cargo build` — clean.
- `cargo test --workspace` — **108 passed, 0 failed** (68 lexer + 38
  parser + 2 doc tests).
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
  Pre-existing `approx_constant` lint on the `3.14` float-lexer literal
  silenced with a local `#[allow(clippy::approx_constant)]` and a
  comment explaining the literal is under test, not approximating π.
- Milestone: `hmatc --emit=ast examples/hello_world.hm` prints the AST
  without crashing.

### Fixed
- Lexer tests: 19 failures in `hmatc/tests/lexer_tests.rs` caused by test
  expectations omitting the trailing `Newline` token the lexer correctly emits
  for single-line inputs. Token-list assertions gained a final `Token::Newline`;
  count-based assertions (`float_literals`, `float_with_exponent`,
  `string_escapes_are_decoded`, `f_string_preserves_interior_for_parser`) were
  incremented by one. Lexer code itself was not changed. `cargo test
  --workspace` now reports 37 passed, 0 failed.

### Added
- `hmatc` Cargo workspace (`Cargo.toml`) with the `hmatc` binary+library crate
- Lexer: complete tokenization of all Phase 0 HMAT tokens per spec v0.2 §2
  (keywords, identifiers, int/float/hex/bin/oct literals with `_` separators,
  string literals with escape processing including `\xHH`, f-string bodies,
  all arithmetic/comparison/assignment/arrow/borrow/bitwise-unary operators,
  all delimiters including `..`, `..=`, `::`)
- Lexer: significant-whitespace handling — `Indent`/`Dedent` injection over
  a 4-column indent model (tabs count as 4), blank and comment-only lines do
  not affect indentation, trailing dedents emitted at EOF
- Lexer: friendly `LexError` with codes (`E001` unknown token, `E002`
  malformed literal, `E003` indent mismatch), span tracking, and help text
- CLI: `hmatc --emit=tokens` prints the token stream in a greppable format;
  `--emit=ast|hir|llvm-ir` stubbed with "not implemented yet" messages
- Stubs for `parser` and `ast` modules so the workspace compiles end-to-end
- Shared `CompilerError` enum (`error.rs`) wrapping IO + lex errors
- `hmatc/tests/lexer_tests.rs` — 30+ integration tests covering every token
  category, indent edge cases, span correctness, and end-to-end hello-world
  tokenization

### Completed (pre-session)
- Spec v0.2: language grammar, type system, ownership model, AI constructs
- Initial documentation scaffolding
- Project architecture defined

### Known Issues
- (resolved) `cargo test --workspace` — previously unverified on the author's
  machine due to missing Windows SDK libs. Now runs green: 37 passed, 0 failed.

### Documentation (this session)
- `docs/getting-started.md` — installation, 5-minute tour, cross-reference table
- `docs/syntax-guide.md` — complete language reference (variables, functions, structs,
  enums, traits, generics, closures, match, loops, ownership syntax, imports, unsafe)
- `docs/ownership-guide.md` — ownership model, shared/mutable borrows, lifetime inference,
  regions, clone, common errors and what they mean
- `docs/error-handling.md` — Result/Option, `?` operator, custom error types,
  `.map_err()`, when to use panic
- `docs/ai-guide.md` — model declarations, chat/summarize/classify/embed/extract,
  pipelines, parallel calls, security rules, HmatCode type
- `docs/async-guide.md` — async functions, join/race/spawn, timeouts, patterns
- `docs/toolchain.md` — hmatc flags, hpkg, hfmt, hmat-lsp, REPL (current + planned)
- `examples/README.md` — index of all examples with phase and status
- `README.md` — rewritten as a full public project homepage

### Added (2026-04-21 — Type Checker session)
- `hmatc/src/semantic/types.rs` — Phase 0 type checker (~450 lines):
  - `Type` enum: `Int`, `Float`, `Str`, `Bool`, `Unit`, `Nil`, `Unknown`
  - `check(&Program)` public entry point — one-pass tree walk, collects all
    errors rather than stopping at the first
  - Two-pass function resolution: pass 1 registers all signatures (enables
    forward calls), pass 2 checks bodies
  - Primitive type inference for all literal kinds (int, float, str, bool, nil)
  - `let` binding checking: infers type from RHS; validates annotation if present;
    errors when `nil` has no annotation (E101)
  - Function argument checking: arity (E106), per-argument type (E107)
  - `return` statement checking against declared return type (E109)
  - Binary operator type checking — arithmetic on numeric types, comparison
    producing `bool`, logical `and`/`or` on booleans (E102)
  - Unary operator checking — `-` on numeric, `not` on bool (E103)
  - Identifier resolution with scope stack (E104)
  - Function call resolution (E105)
  - Type annotation parsing — all sized integer aliases (`i8`–`i128`,
    `u8`–`u128`), float aliases (`f32`, `f64`), unknown type names (E110)
  - `Unknown` sentinel suppresses cascading errors after the root cause fires
  - Phase 0 builtin: `print(str) -> ()` wired in without a stdlib loader
- `hmatc/src/semantic/mod.rs` — `pub use types::{check, TypeError, Type}`
- `hmatc/src/error.rs` — `CompilerError::Type` variant; `From<Vec<TypeError>>`
- `hmatc/src/main.rs` — `--emit=ast` pipeline now runs the type checker after
  parsing; each `TypeError` printed with its code and `help` line
- `hmatc/tests/type_checker_tests.rs` — 38 integration tests:
  happy-path coverage for all primitive types, let bindings, function calls,
  return types, operators; error-path coverage for every E-code E100–E110;
  milestone test `cli_emit_ast_hello_world` verifies the full pipeline
- `docs/type-system.md` — new: phase-0 type system guide (primitives,
  inference, annotations, operators, error reference)
- `CHANGELOG.md` — this entry

### Verified (2026-04-21)
- `cargo test --workspace` — **171 passed, 0 failed, 3 ignored**
  (68 lexer + 63 parser + 38 type checker + 2 doc tests)
- `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings
- `cargo fmt --check` — clean
- Milestone: `hmatc --emit=ast examples/hello_world.hm` type-checks with
  zero errors

### Audited (2026-04-21 — Foundation audit session)
- Foundation audit complete — lexer, parser, AST vs spec v0.3.
- See `audit/FOUNDATION-AUDIT-2026.md` for full findings.
- Three HIGH RISK spec-drift items found and fixed in the same session:
  - **Lexer keywords** aligned to spec v0.3: added `shape` / `on` / `flow` /
    `fail`; removed obsolete `match` / `struct` / `enum` / `trait` / `impl` /
    `load` / `pipeline` / `where` / `?` variants.
  - **Parser generics** now use `[T]` (spec v0.3 §2.7 / §3.1) instead of
    `<T>`. `<` and `>` remain comparison operators only.
  - **Return types** now accept fallible/optional suffixes per spec v0.3
    §3.1 / §4: `-> T or Fail`, `-> T or nil`, `-> T or Fail or nil`.
    `TypeRef` gained `is_fallible` / `is_nilable` flags; new
    `parse_return_type()` helper restricts these suffixes to return position.
- Test updates: lexer keyword cases refreshed; 4 new parser tests
  (`function_with_fallible_return_type`, `function_with_nilable_return_type`,
  `function_with_fallible_and_nilable_return_type`,
  `generic_return_type_with_single_param`); existing
  `function_with_generic_return_type` switched from `<...>` to `[...]`;
  `ai_model_decl_tokenizes` updated to spec-v0.3 form.

### Verified (2026-04-21 — post-audit)
- `cargo test --workspace` — **177 passed, 0 failed, 3 ignored**
  (69 lexer + 67 parser + 38 type checker + 3 doc tests)
- `cargo clippy --workspace --all-targets -- -D warnings` — clean
- Milestone: `hmatc --emit=ast examples/hello_world.hm` — still end-to-end green

### Added (2026-04-21 — Phase 1: C Codegen session)
- `hmatc/src/codegen/c.rs` — C11 emitter (~300 lines) that walks the AST and
  produces self-contained C source ready for `clang`:
  - Type mapping: `int → int64_t`, `float → double`, `bool → bool`,
    `str → const char *`, `() → void`. All sized integer aliases
    (`i8`–`i128`, `u8`–`u128`, `f32`, `f64`) mapped to correct C types.
  - Literals: integers suffixed `LL` for 64-bit safety; floats guaranteed to
    carry a decimal point via `{:?}`; strings and f-strings escape-encoded
    to ASCII-only C literals (UTF-8 via `\xNN`).
  - `print(x)` builtin lowered to `printf("%s\n", x)`.
  - `^` exponentiation lowered to `pow()` (libm).
  - `and`/`or` lowered to `&&`/`||`.
  - `let` bindings: explicit type annotation respected; unannotated bindings
    use `__auto_type` (clang/gcc extension) so clang infers from the RHS.
  - `mut` / immutable mapped to absent / `const` qualifier.
  - `fn main()` auto-receives `int` return type and a trailing `return 0;`
    when the source omits both.
- `hmatc/src/codegen/mod.rs` — public `emit_c(program: &Program) -> String`
  entry point; module-level doc explains the C-via-clang strategy and the
  planned migration to `inkwell` once it supports LLVM 18+.
- `hmatc/src/main.rs` — full compile pipeline:
  - `compile()`: source → C → disk → `clang -O2 -std=c11` → native binary.
    Intermediate `.c` file deleted on success; kept on failure for inspection.
  - `find_clang()`: searches `PATH` first, falls back to
    `C:\Program Files\LLVM\bin\clang.exe` on Windows.
  - `--emit=c` flag: prints generated C source to stdout (no binary produced).
  - `--emit=llvm-ir` flag: prints a placeholder explaining the inkwell
    LLVM 18+ gate rather than crashing.
  - `-o <path>` flag: override the output binary path.
  - `exe_suffix()`: appends `.exe` on Windows automatically.
- `hmatc/src/error.rs` — two new `CompilerError` variants:
  - `ClangNotFound` — friendly message with install instructions.
  - `ClangFailed(i32)` — reports clang's exit code; clang's own stderr is
    already visible on the terminal.

### Verified (2026-04-21 — Phase 1)
- `where clang` / `C:\Program Files\LLVM\bin\clang.exe` — present.
- `cargo test --workspace` — all green (count from prior session: 177 passed).
- `hmatc examples/hello_world.hm` — produces `hello_world.exe`.
- `.\hello_world.exe` — prints `Hello, HMAT!`.
- Milestone **v0.2.0** achieved: first end-to-end compile.

### Security (2026-04-21 — Phase 1 security fix session)
Fixes for the Phase 1 audit findings. See `security/SESSION-2026-04-21.md`
(Phase 1 Security Fix Session addendum) for the full write-up.

- **HIGH (CWE-427) — Clang resolved without trusting PATH.**
  `hmatc/src/driver.rs` (new) owns the toolchain lookup. `find_on_path_in`
  walks `$PATH` manually, skips empty / relative / CWD-equivalent entries,
  and returns an absolute `PathBuf`. `main.rs::compile` now hands the
  absolute path to `Command::new` — `Command::new("clang")` with a bare
  name is gone.
- **MEDIUM — Argument smuggling via filename / `-o` override.**
  `driver::validate_output_name` rejects any name starting with `-` with
  `CompilerError::InvalidOutputName` (code **E005**). Both the input-file
  stem and the `-o` override's file name are validated.
  `driver::absolute_output_paths` anchors the generated `.c` and the
  output binary at the absolute CWD — the compile command passed to
  `clang` never contains a bare relative path.
- **MEDIUM — `\xNN` run-on in `escape_c_string`.**
  `codegen/c.rs::escape_c_string` now emits non-ASCII / control bytes as
  three-digit octal escapes (`\NNN`). Hex escapes in C read unlimited
  hex digits and could fold adjacent characters into one wide code point.
  Octal escapes self-terminate at three digits. `\0` folded into the
  same branch (`\000`) for consistency.
- **LOW — `.gitignore`.** Added an "hmatc outputs" section ignoring
  `*.exe` and generated `*.c` so Phase 1 build artifacts stay out of
  git. (`hmatc` itself has no hand-written C — it's a Rust crate — so
  the blanket `*.c` rule is safe.)
- **Tests:** `hmatc/tests/driver_tests.rs` (new, 9 tests) covers both
  fixes: CWD-skip, relative-entry skip, empty-entry skip, positive
  lookup, stem validation, `-o` validation, absolute-path anchoring,
  relative-override promotion.
- **Verified:** `cargo test --workspace` — **221 passed / 0 failed /
  3 ignored** (35 codegen + 9 driver + 69 lexer + 67 parser + 38
  type-checker + 3 doc). `cargo clippy -- -D warnings` clean. End-to-end
  `hmatc examples/hello_world.hm` → `hello_world.exe` → `Hello, HMAT!`
  re-verified; the `hmatc: wrote …` line now reports an absolute path.

### Fixed (2026-07-06 — Repo hygiene session)
- **`hmatc/src/driver.rs` — `find_clang()`.** Replaced the hardcoded
  `C:\Program Files\LLVM\bin\clang.exe` fallback with an
  `%ProgramFiles%`-resolved path on Windows, so the fallback keeps working
  on any install drive/locale without baking in an absolute OS-specific
  path. `PATH` lookup remains the primary strategy on every OS; the
  fallback is only ever consulted on Windows.
- **`hmatc/src/error.rs` — `ClangNotFound` message.** Now OS-neutral
  ("install LLVM/clang and ensure `clang` is on your PATH").
- Docs (`docs/toolchain.md`, `docs/getting-started.md`) updated to match.

### Next
- Phase 2: functions, closures, structs (`shape`), sum types (`type`),
  pattern matching (`on`), generics, ownership checker (lite).
- LLVM IR backend (swap in `inkwell` once it supports LLVM 18+).
- Carried security work: F-2 / F-9 (parser + type-checker depth caps),
  F-6 (fuzz harness), F-4 (`cargo-audit` / `cargo-deny`), F-3 (source
  size cap), F-7 (`\xHH` spec clarification).

---

## Roadmap Milestones

| Milestone            | Description                          | Status      |
|----------------------|--------------------------------------|-------------|
| v0.1.0 — Foundation  | Lexer + Parser + basic type checker  | ✅ Done     |
| v0.2.0 — Hello World | Compiles and runs first program      | ✅ Done     |
| v0.3.0 — Core        | Functions, structs, enums, generics  | 🔲 Pending  |
| v0.4.0 — AI Native   | ai model, await, pipeline            | 🔲 Pending  |
| v0.5.0 — Toolchain   | hpkg, hfmt, hmat-lsp                 | 🔲 Pending  |
| v1.0.0 — Community   | Stdlib, registry, launch             | 🔲 Pending  |
