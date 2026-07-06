# HMAT State Report — 2026-04-22
# Generated for handoff to new Claude chat session

---

## 1. Test Suite Status

- Total tests: **224** (221 passed / 0 failed / 3 ignored)
  - `codegen_tests`: 35 passed
  - `driver_tests`: 9 passed
  - `lexer_tests`: 69 passed
  - `parser_tests`: 67 passed + 2 ignored
  - `type_checker_tests`: 38 passed
  - doc tests: 3 passed + 1 ignored
- cargo check: **CLEAN**
- cargo clippy `-D warnings`: **CLEAN**

### Failing Tests
None.

### Ignored Tests (expected — not failures)
| Test | Reason |
|------|--------|
| `ai_chat_first_error_is_import` | `ai_chat.hm` uses Phase 3+ constructs (`import`, `ai`, `flow`). Un-ignore when import parsing lands. |
| `core_language_first_error_is_compound_assign` | `core_language.hm` uses Phase 2+ constructs. First unsupported token: `+=`. Un-ignore when compound assignment parses. |
| `lexer::tokenize` (doc test line 48) | Marked `ignore` in doc comment; placeholder example. |

---

## 2. Phase Status

### Completed
- **Phase 0 ✅** — Rust workspace, lexer (significant indentation, all spec v0.3 tokens), recursive-descent parser (fn decls, let/return/exprs, `[T]` generics, `or Fail` / `or nil` return types, full precedence climb), AST + pretty printer, type checker for primitives (E100–E110), foundation audit vs spec v0.3.
- **Phase 1 ✅** — C11 codegen (`hmatc/src/codegen/c.rs`), clang driver (`hmatc/src/driver.rs`), `hmatc examples/hello_world.hm` produces `hello_world.exe` that prints `Hello, HMAT!`. Security hardening applied (CWE-427, arg-smuggling, `\xNN` run-on).

### Current Phase
- **Phase 2 🔲** — Core Language
- Current task in `CLAUDE.md` is still labelled Phase 1 (stale). `CHANGELOG.md` already records Phase 1 as shipped and lists Phase 2 scope under `### Next`. **Action:** update `CLAUDE.md` §"Current Task" to Phase 2 before next pipeline run.

### Upcoming
- **Phase 3:** AI-native constructs (`ai model`, `await`, flows, LLM calls).
- **Phase 4:** Toolchain (`hpkg`, `hfmt`, `hmat-lsp`, VS Code ext, WASM target, REPL).
- **Phase 5:** Data-science stdlib (tensor, frame, learn, nn, stats, plot, data, pretrained).

---

## 3. Compiler Component Status

| Component          | Status | Test Count | Notes |
|--------------------|--------|-----------:|-------|
| Lexer              | ✅     | 69 | `logos` + indent injector. All spec v0.3 keywords. |
| Parser             | ✅     | 67 + 2 ign. | Phase 0 surface only — no shape/type/on/flow/ai/import yet. |
| AST                | ✅     | (covered in parser + codegen) | `Program`/`Item::Function` only. Items for shape/type/flow/ai not yet added. |
| Type Checker       | ✅     | 38 | Primitives only. Two-pass fn resolution. E100–E110. |
| Semantic Analysis  | 🟡 partial | — | Only type checker; no ownership, no AI type validator. |
| C Codegen          | ✅     | 35 | Phase 1 scope: fn, let/return/expr, arithmetic, `print` → `printf`. |
| LLVM Codegen       | 🔲     | 0 | Placeholder in `--emit=llvm-ir`. Blocked on `inkwell` + LLVM 18+. |
| Driver / toolchain | ✅     | 9 | clang discovery (CWE-427 hardened), absolute-path outputs. |
| HIR / MIR          | 🔲     | 0 | Not started. `--emit=hir` prints "not implemented yet (Phase 2)". |

---

## 4. Example File Compatibility

| File              | Parses | Type-checks | Compiles | Runs | Output | Notes |
|-------------------|--------|-------------|----------|------|--------|-------|
| `hello_world.hm`  | ✅ | ✅ | ✅ | ✅ | `Hello, HMAT!` | Phase 1 milestone — green. |
| `core_language.hm`| ❌ | — | — | — | `E010: expected a top-level declaration (fn), found identifier` | Bare top-level bindings (`name = "HMAT"`), `shape`, `type`, `on`, `+=`, generics, collections — all Phase 2. |
| `ai_chat.hm`      | ❌ | — | — | — | `E010: expected a top-level declaration (fn), found from` | `from … import`, `ai model`, `async`, `flow`, `await` — Phase 3. |
| `data_science.hm` | ❌ | — | — | — | `E010: expected newline after let statement, found [` | `tensor[...]` literal syntax, `frame.csv`, `|` pipe queries — Phase 5. |

---

## 5. Spec Alignment (spec/0.2/grammar.md, versioned v0.3)

| Spec Feature                        | Lexed | Parsed | Type-checked | Lowered | Notes |
|-------------------------------------|:-----:|:------:|:------------:|:-------:|-------|
| `fn` keyword                        | ✅ | ✅ | ✅ | ✅ | Full Phase 0/1 support. |
| `let` / bare binding                | ✅ | ✅ (let only) | ✅ | ✅ | Bare top-level bindings (constants) not parsed — see §6. |
| `mut` bindings / params             | ✅ | ✅ | partial | ✅ | No mutation check beyond C `const` qualifier. |
| `shape` declarations                | ✅ | ❌ | ❌ | ❌ | Keyword reserved; parser bails. |
| `type` declarations (sum types)     | ✅ | ❌ | ❌ | ❌ | Reserved only. |
| `on` pattern matching               | ✅ | ❌ | ❌ | ❌ | Reserved only. |
| `=>` match arms / closures          | ✅ (FatArrow) | ❌ | — | — | Token exists, no grammar rule uses it. |
| `T or Fail` return type             | ✅ | ✅ | ❌ | ❌ | Parsed into `TypeRef.is_fallible`; ignored in checker/codegen. |
| `T or nil` return type              | ✅ | ✅ | ❌ | ❌ | Parsed into `TypeRef.is_nilable`; ignored downstream. |
| `fail` statement                    | ✅ | ❌ | — | — | Reserved only. |
| `or` fallback operator              | ✅ | ✅ (as logical) | ✅ (bool only) | ✅ | Context-sensitive "fallback" meaning not implemented — always logical. |
| `[T]` generics                      | ✅ | ✅ (return type + type args) | partial | — | `fn max[T](a: T, b: T)` generic **parameter lists** not yet parsed. |
| `x => expr` closures                | ✅ | ❌ | — | — | Token exists; no closure grammar. |
| `flow` declarations                 | ✅ | ❌ | — | — | Reserved only. |
| `ai model(...)` declarations        | ✅ (`ai`, `model`) | ❌ | — | — | Reserved only. |
| Significant indentation             | ✅ | ✅ | n/a | n/a | Indent/Dedent injection in lexer. |
| `import` / `from … import`          | ✅ | ❌ | — | — | Reserved; parser emits E010. |
| `async` / `await`                   | ✅ | ❌ | — | — | Reserved only. |
| `for` / `while` / `break` / `continue` | ✅ | ❌ | — | — | Reserved only. |
| `if`/`elif`/`else` stmt             | ✅ | ❌ | — | — | Reserved only. No ternary `if … else` either. |
| Compound assignment (`+=` …)        | ✅ | ❌ | — | — | Lexer tokens; parser rejects. |
| Ranges (`..`, `..=`)                | ✅ | ❌ | — | — | Lexer tokens only. |
| String interpolation (f-strings)    | ✅ (body captured) | ✅ (as opaque `FStr` literal) | ✅ (typed `str`) | partial (emitted as plain C string — no `{expr}` splitting yet) | Phase 2 target. |
| Collections `[…]` / `{…}` / `{k:v}` | ✅ (delimiters) | ❌ | — | — | Literal forms not parsed. |
| Decorators `@derive` / `@test`      | ✅ (`@` token) | ❌ | — | — | Reserved only. |

---

## 6. Known Issues

### Blocking (must address before / during Phase 2)
1. **CLAUDE.md "Current Task" is stale** — still says Phase 1. Update to Phase 2 scope before the next pipeline runs (the pipeline reads this section).
2. **Bare top-level bindings not parsed** (`name = "HMAT"`) — spec §3.3 `constant_decl`. Blocks `core_language.hm` at the very first line even before Phase 2 core features.
3. **Security F-2 (HIGH) — parser recursion unbounded** (`security/SESSION-2026-04-21.md`). No `MAX_EXPR_DEPTH`/`RecursionLimit` in parser. Carried from 2026-04-20. A 5 000-deep parens input crashes `hmatc` with a stack overflow.
4. **Security F-9 (HIGH) — type-checker recursion unbounded** (`security/SESSION-2026-04-21.md`). Same class as F-2 but in `Checker::check_expression`. Defense-in-depth cap recommended even after F-2 lands (LSP / fuzzer paths bypass parser limits).

### Non-blocking
1. **`--emit=hir`** prints a stub — fine; Phase 2 will wire a real HIR.
2. **`--emit=llvm-ir`** prints a placeholder; blocked on upstream `inkwell` LLVM 18+ support — documented.
3. **F-3 (LOW) — no size cap on source read** (`read_to_string`). Minor DoS surface (memory exhaustion).
4. **F-4 (INFO) — `cargo audit` / `cargo deny` not installed.**
5. **F-6 (MEDIUM) — no fuzz harness** for lexer/parser.
6. **F-7 (LOW) — `\xHH` escape semantics undocumented in spec.**
7. **Duplicate fn definitions silently overwrite** in the type checker (called out in code comment). Not security, but should become an E11x diagnostic.
8. **F-strings parsed as opaque** — `{expr}` bodies not split yet; codegen emits them as plain C strings. Plan: Phase 2 when interpolation ships.
9. **`or` operator only implements logical semantics**; spec §5.1 context-sensitive "fallback" behavior (`divide(10, 0) or 0.0`) needs the fallible/nilable runtime first.

### Security — carried open
Findings F-2, F-9, F-3, F-4, F-6, F-7 from `security/SESSION-2026-04-21.md` remain unresolved. F-2 and F-9 are the only HIGH items still open.

---

## 7. Git Status

```
On branch main
Your branch is up to date with 'origin/main'.

Changes not staged for commit:
	modified:   .gitignore

Untracked files:
	.claude/
```

```
38fa6e5 feat: Phase 1 complete — Hello World compiles via C codegen + clang
9477723 feat: Phase 0 complete — type checker + foundation audit + spec alignment
e9717bd feat: Phase 0 — Parser (AST nodes + recursive descent)
760483b feat: Phase 0 — workspace scaffold + lexer (hmatc)
```

Untracked build artefact on disk (not in git): `hello_world.exe` at repo root (produced by audit run — `.gitignore` already covers `*.exe`).

---

## 8. What Was Built in Last Session

Phase 1 — C codegen + clang driver (commit `38fa6e5`, 2026-04-21):

- `hmatc/src/codegen/c.rs` — ~300-line C11 emitter. Type map `int → int64_t`, `float → double`, `bool → bool`, `str → const char *`, `() → void`. `print(x)` lowered to `printf("%s\n", x)`. `^` lowered to `pow()`. `let` with annotation respected; without annotation uses `__auto_type`. `fn main()` auto-gets `int` return + `return 0;`.
- `hmatc/src/codegen/mod.rs` — public `emit_c(&Program) -> String`.
- `hmatc/src/main.rs` — `compile()` pipeline (source → C → disk → clang `-O2 -std=c11` → native binary), `find_clang()`, `--emit=c`, `--emit=llvm-ir` placeholder, `-o` override, `exe_suffix()`.
- `hmatc/src/driver.rs` (new, security-hardened) — `find_on_path_in` walks `$PATH` manually, skips empty/relative/CWD entries; `validate_output_name` rejects names starting with `-` (E005); `absolute_output_paths` anchors `.c` and binary at absolute CWD.
- `hmatc/src/error.rs` — `ClangNotFound`, `ClangFailed(i32)`, `InvalidOutputName(String)` (E005).
- `hmatc/tests/codegen_tests.rs` (35 tests) — type mapping, literals, escapes, operators, `main()` shape.
- `hmatc/tests/driver_tests.rs` (9 tests) — PATH lookup positive/negative, CWD skip, arg-smuggling rejection, absolute-path anchoring.
- **End-to-end**: `hmatc examples/hello_world.hm` → `hello_world.exe` → `Hello, HMAT!`. Milestone **v0.2.0** achieved.
- **Security fixes** applied same session (see `security/SESSION-2026-04-21.md` addendum): CWE-427 clang PATH trust, arg-smuggling via filename/`-o`, `\xNN` run-on in C string escapes (swapped to octal).

This audit session itself made no code changes.

---

## 9. Recommended Next Action

**Update `CLAUDE.md` §"Current Task" to Phase 2 + address security blockers, then start Phase 2 core-language surface.**

Concrete first slice (one pipeline run):

1. **Security hardening (F-2 + F-9)** — land recursion caps in parser (`MAX_EXPR_DEPTH = 256`, `ParseError::RecursionLimit` code E016) and type checker (`MAX_TYPE_DEPTH = 512`, `TypeError::RecursionLimit` code E111). Add two regression tests (parser overflow, checker overflow on hand-built deep AST). Unblocks untrusted-input safety for LSP/REPL later.
2. **Parser: top-level constant bindings** — spec §3.3 `constant_decl ::= IDENTIFIER '=' expression NEWLINE`. Tiny win; immediately unblocks the first block of `core_language.hm`.
3. **Parser: `if` / `elif` / `else` statement + ternary `expr if cond else expr`** — small surface, huge expressivity. Prerequisites for `fn area(...)` with guards.
4. **Parser: `shape` declarations** (fields only; methods inline but no `self` semantics yet). First new top-level `Item` variant beyond `Function`.
5. **Codegen: `shape` → C `struct` with pass-by-value semantics**. Minimal — no methods yet.

Files that change: `hmatc/src/parser/mod.rs`, `hmatc/src/ast/mod.rs` (new Item variants + `ShapeDecl` + `IfStmt`), `hmatc/src/semantic/types.rs` (handle new AST, register shape types), `hmatc/src/codegen/c.rs` (emit `struct`, handle `if/elif/else`). Tests in all four `tests/*.rs` files.

Milestone gate: a focused `examples/phase2_shapes.hm` (smaller than `core_language.hm`) compiles and runs.

---

## 10. Open Questions / Decisions Needed

1. **CLAUDE.md vs CHANGELOG phase labels disagree.** Pipeline reads CLAUDE.md "Current Task" — is it okay for the audit/handoff chat to rewrite it to Phase 2 now, or does Hassan want to confirm scope first?
2. **Spec version label.** Files live at `spec/0.2/` but the documents inside say "v0.3". Rename the directory to `spec/0.3/` or update the documents' headers? (CHANGELOG already talks about "spec v0.3".)
3. **Phase 2 ordering.** The roadmap lists functions/closures/structs/enums/pattern matching/generics/ownership/Result/Option/string interpolation as all part of Phase 2. Do we ship them as one big branch (12+ weeks) or as sub-phases 2a/2b/2c with releases in between?
4. **`examples/core_language.hm` is the Phase 2 milestone** per its header comment — should the milestone definition of done be "this file compiles and prints"? That pins scope nicely but also locks us into `shape`, `type`, `on`, pipe-style queries, generics, comprehensions, and pattern guards all at once. Alternative: introduce a smaller `phase2_*.hm` gating example per sub-phase.
5. **Security: should F-2/F-9 block Phase 2 feature work**, or ride alongside? Recommendation: land the recursion caps as the **first** commit of Phase 2 — cheap, closes both HIGH items, and any new recursive descent rule in Phase 2 inherits protection for free.
6. **LLVM IR backend timing.** Watch `inkwell` upstream for LLVM 18+ support. Do we also track `cranelift` as a backup path so the language is not single-vendor?
7. **Ownership model (Phase 2 "lite" per CLAUDE.md).** Spec docs `spec/0.2/ownership.md` not read in this audit — need to confirm it still matches the "ergonomic, invisible borrows" decision D005/D016 before codegen has to respect it.
