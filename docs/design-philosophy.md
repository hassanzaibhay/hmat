# HMAT — Design Philosophy

This document captures HMAT's identity: why it exists, what it optimizes for,
and the syntax choices that follow from that. It is the reference other docs
and the compiler roadmap point back to.

---

## What Is HMAT?

HMAT is an AI-native, general-purpose programming language being built as a
gift to the developer community.

**The North Star:**
> Simple like Python. Powerful like C++. Optimized like Rust. Scalable like Java. AI-native from day one.

---

## Core Philosophy

| Principle     | What It Means                                                              |
|---------------|----------------------------------------------------------------------------|
| **Simple**    | No ceremony. No boilerplate. If a beginner can't read it, it's wrong.      |
| **Safe**      | Memory safety without a GC tax. Ownership is ergonomic, not a punishment. |
| **Fast**      | Zero-cost abstractions. LLVM backend. Competes with C++ in hot paths.     |
| **AI-Native** | AI constructs are first-class citizens — not an afterthought library.     |
| **Scalable**  | Module system, toolchain, and package manager built to grow with teams.    |
| **Loved**     | Syntax that feels good to write. Errors that feel like a mentor, not a wall.|

---

## Language Identity

- **File extension:** `.hm`
- **Data Science:** First-class — tensor, frame, learn, nn, stats, plot built-in (no packages)
- **Compiler name:** `hmatc` (written in Rust)
- **Package manager:** `hpkg`
- **REPL:** `hmat` (interactive shell)
- **Formatter:** `hfmt`
- **LSP:** `hmat-lsp`
- **Spec version:** v0.2
- **Compiler backend:** LLVM IR (primary), WASM (secondary target)
- **Runtime:** None (compiled to native). Optional lightweight async runtime for AI/IO workloads.
- **Memory model:** Ownership + regions. Borrow checker lite — ergonomic, not Rust-strict.

---

## Zen of HMAT

1. Simple beats clever — always
2. Errors are teachers, not walls
3. Safety without ceremony
4. AI is first-class, not a plugin
5. Build for the developer in 2035
6. If a beginner can't read it in 60 seconds, it's wrong
7. No feature ships without a test
8. No test ships without a passing run

---

## Compiler Architecture

```
Source (.hm)
    |
    v
Lexer (tokenizer)
    |
    v
Parser (AST)
    |
    v
Semantic Analysis
  +-- Type Checker
  +-- Ownership Checker
  +-- AI Type Validator
    |
    v
HIR -> MIR -> LLVM IR -> Native Binary / WASM
```

See `spec/roadmap.md` for what's implemented today vs. planned.

---

## Quality Standards

- Every public API has documentation
- Every feature has tests before merge
- Compiler errors are human-readable — always suggest a fix
- No feature ships without a working example
- Security reviewed before any release
- If a new contributor can't read it in 60 seconds, refactor

---

## The Spirit of This Project

This language is being built with love — not to ship fast, but to ship right.
Every design decision should ask: "Will a developer using this in 2035 thank us for this choice?"
Build small. Build daily. Build with care.

— Hassan Zaib Hayat

---

## Syntax DNA

HMAT has its own unique identity. It must NOT look like Python, Rust, C++, Java, or any existing language.

```hmat
# Variables — no keyword noise
name     = "HMAT"
mut count = 0         # mut required for mutation — intentional friction

# Functions — no keyword, just signature
greet(name: str) -> str:
    "Hello, {name}!"

add(a: int, b: int) -> int: a + b    # single-line

# Fallible — 'or Fail' not Result<T,E>
divide(a: float, b: float) -> float or Fail:
    fail "zero" if b == 0.0
    a / b

# Data shapes — not struct/class
shape Point:
    x: float
    y: float
    distance(self, other: Point) -> float:
        ((self.x-other.x)^2 + (self.y-other.y)^2).sqrt()

# Sum types — not enum
type Shape:
    Circle(radius: float)
    Rect(width: float, height: float)

# Pattern match — 'on' not match/switch
area(s: Shape) -> float:
    on s:
        Circle(r)  => 3.14159 * r^2
        Rect(w, h) => w * h

# Error handling — 'or' not unwrap_or
result = divide(10, 0) or 0.0
val    = first([])     or -1

# Generics — [T] not <T>
max[T: Comparable](a: T, b: T) -> T:
    a if a > b else b

# Closures — x => expr
double = x => x * 2
add    = (a, b) => a + b

# AI — first-class language primitive
ai assistant = model("anthropic/claude-3-5-sonnet")
reply   = assistant.ask("What is HMAT?")
code    = assistant.gen("sort by frequency")
summary = assistant.think(document)

# Flow (pipeline) — not pipeline
flow analyze:
    input => clean => tokenize => classify

# Async
async fetch(url: str) -> str or Fail:
    resp = await http.get(url)
    resp.text()
```
