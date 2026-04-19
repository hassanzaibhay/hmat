# HMAT Language Specification v0.2
# Ownership and Memory Model
# Author: Hassan Zaib Hayat <hassanzaibhayatske@gmail.com>

---

## 1. Design Philosophy

HMAT's memory model is inspired by Rust's ownership system but redesigned for ergonomics.
The goal: **memory safety without a garbage collector, and without making developers fight the compiler.**

The three rules that cause Rust fatigue — explicit lifetimes, fighting the borrow checker on common patterns, complex self-referential structs — are addressed directly in HMAT's design.

**HMAT's ownership principles:**
1. Every value has exactly one owner at a time
2. When the owner goes out of scope, the value is freed
3. Borrows are tracked by the compiler — no dangling references possible
4. Lifetime annotations are almost never needed (inferred in 95%+ of cases)
5. Common patterns (returning references, self-referential structs) work without ceremony

---

## 2. Ownership Basics

### 2.1 Move Semantics

By default, assigning or passing a value **moves** it. After a move, the original variable is invalid.

```hmat
let s = "hello"
let t = s           # s is moved into t
print(s)            # ERROR E201: use of moved value `s`
print(t)            # OK
```

```hmat
fn take_ownership(s: str):
    print(s)

let greeting = "hello"
take_ownership(greeting)     # greeting is moved
print(greeting)              # ERROR E201: use of moved value
```

### 2.2 Copy Types

Primitive types implement `Copy`. They are copied on assignment, not moved.

Copy types: `int`, `i8`–`i128`, `uint`, `u8`–`u128`, `float`, `f32`, `f64`, `bool`, `char`, `byte`, `()`, fixed-size arrays of Copy types, tuples of Copy types.

```hmat
let x = 42
let y = x       # x is copied, not moved
print(x)        # OK — x still valid
print(y)        # OK
```

### 2.3 Clone

Non-copy types can be explicitly cloned:

```hmat
let s = "hello"
let t = s.clone()   # deep copy — s and t are independent
print(s)            # OK
print(t)            # OK
```

Clone is always explicit. There is no implicit cloning.

---

## 3. Borrowing

Instead of moving, you can **borrow** a value. A borrow is a reference — it does not transfer ownership.

### 3.1 Immutable Borrows

```hmat
fn print_length(s: &str):
    print(s.length())

let greeting = "hello"
print_length(&greeting)     # borrow greeting — greeting is still valid
print(greeting)             # OK — greeting was only borrowed
```

Multiple immutable borrows can coexist:

```hmat
let s = "hello"
let r1 = &s
let r2 = &s
let r3 = &s
print(r1)   # OK
print(r2)   # OK
print(r3)   # OK
```

### 3.2 Mutable Borrows

```hmat
fn append_world(s: &mut str):
    s.push(" world")

let mut greeting = "hello"
append_world(&mut greeting)
print(greeting)    # "hello world"
```

**Mutable borrow rule:** Only one mutable borrow may exist at a time, and no immutable borrows may coexist with it.

```hmat
let mut s = "hello"
let r1 = &s           # immutable borrow
let r2 = &mut s       # ERROR E202: cannot borrow `s` as mutable
                      # because it is also borrowed as immutable
```

```hmat
let mut s = "hello"
let r1 = &mut s
let r2 = &mut s      # ERROR E203: cannot borrow `s` as mutable more than once
```

### 3.3 Borrow Scope (Non-Lexical Lifetimes)

Borrows end as soon as they are last used — not at the end of the block.
This makes common patterns work without fighting the compiler.

```hmat
let mut s = "hello"
let r1 = &s
print(r1)             # last use of r1 — borrow ends here
let r2 = &mut s       # OK — r1 is no longer active
r2.push(" world")
print(s)              # OK
```

---

## 4. Lifetimes

In HMAT, **lifetime annotations are almost never written explicitly.**
The compiler infers them in the vast majority of cases.

### 4.1 The Three Lifetime Elision Rules

These rules cover 95%+ of real code:

**Rule 1:** Each reference parameter gets its own lifetime.
```hmat
fn first_word(s: &str) -> &str:   # compiler infers: same lifetime as s
```

**Rule 2:** If there is exactly one input reference, the output lifetime matches it.
```hmat
fn first(s: &str) -> &str:
    # output lives as long as s — inferred automatically
    return s[0..1]
```

**Rule 3:** If one of the parameters is `&self` or `&mut self`, the output lifetime matches `self`.
```hmat
struct Parser:
    source: str

    fn current_token(&self) -> &str:
        # output lives as long as self — inferred automatically
        return self.source[self.pos..self.pos+1]
```

### 4.2 When You DO Need Lifetime Annotations

Only when returning a reference from a function with multiple input references and the compiler cannot determine which input the output borrows from:

```hmat
# The compiler doesn't know if the result borrows from a or b
fn longer<'a>(a: &'a str, b: &'a str) -> &'a str:
    if a.length() > b.length():
        return a
    return b
```

This is the rare case. In practice, restructure to return owned values when this occurs.

**HMAT design goal:** If you find yourself writing lifetime annotations more than once a week, report it — that's a compiler bug or a design gap we need to fix.

---

## 5. Ownership in Structs

### 5.1 Owned Fields

```hmat
struct User:
    name: str       # owned — User owns this string
    age: int        # Copy — no ownership concern
    email: str      # owned

# User frees name and email when it goes out of scope
```

### 5.2 Reference Fields (Rare)

Structs with reference fields are rare in HMAT. Prefer owned data in structs.
When needed:

```hmat
struct StrSplit<'a>:
    source: &'a str
    delimiter: &'a str
    position: int
```

---

## 6. The Ownership Checker (Lite)

HMAT's ownership checker is intentionally simpler than Rust's. It catches real memory bugs without false positives on common safe patterns.

### What It Catches

- Use after move
- Use after free (via borrow rules)
- Double free (impossible by design)
- Data races (in async/concurrent code)
- Null pointer dereference (Option<T> prevents null)

### What It Allows That Rust Doesn't

HMAT allows these common patterns without ceremony:

```hmat
# Pattern 1: Return reference to local struct field (common, safe)
struct Config:
    host: str
    port: int

    fn get_host(&self) -> &str:
        return &self.host   # OK — lifetime inferred correctly

# Pattern 2: Builder pattern without fighting lifetimes
struct Builder:
    name: str
    value: int

    fn set_name(mut self, name: str) -> Builder:
        self.name = name
        return self     # OK — self is moved, not borrowed

# Pattern 3: Iterator returning references
for item in &collection:   # borrows collection, item is &T
    print(item)
```

---

## 7. Move vs Copy: Decision Table

| Type                        | Behavior | Why                           |
|-----------------------------|----------|-------------------------------|
| `int`, `float`, `bool`, `char` | Copy  | Cheap to copy                 |
| Fixed-size arrays of Copy types | Copy  | Cheap to copy                |
| `str`                       | Move     | Heap-allocated                |
| `[T]` (dynamic array)       | Move     | Heap-allocated                |
| `{K: V}` (map)              | Move     | Heap-allocated                |
| Structs with Copy fields only | Copy  | `@derive(Copy)` required      |
| Structs with any owned field | Move    | Cannot copy heap data         |
| `Option<T>`, `Result<T,E>`  | Follows T | Transparent wrapper          |
| `Model` (AI model handle)   | Move     | Non-trivial resource          |

---

## 8. Memory Regions

In addition to heap and stack, HMAT supports **regions** — a way to allocate a group of values together and free them all at once.

```hmat
# Region allocates from a single bump allocator
# All allocations in the region are freed together when region ends
region r:
    let a = r.alloc(SomeStruct { ... })
    let b = r.alloc(AnotherStruct { ... })
    process(a, b)
# a and b freed here — single deallocation
```

Regions are useful for:
- Request handling (allocate for a request, free all at end)
- Parsing (allocate AST nodes, free when done)
- AI pipeline stages (allocate intermediate results, free after output)

---

## 9. Unsafe Memory Operations

The `unsafe` block is required for:
- Raw pointer creation and dereference
- Calling C FFI functions
- Manual memory allocation/deallocation
- Casting between non-trivially related types

```hmat
# SAFETY: ptr comes from a Box::into_raw call and hasn't been freed
# INVARIANT: we call this function exactly once per allocation
unsafe:
    let val = *ptr
    drop_raw(ptr)
```

Every unsafe block must have:
1. A `# SAFETY:` comment explaining why it is safe
2. An `# INVARIANT:` comment if there are conditions that must hold

---

## 10. Common Ownership Errors and Fixes

```
error[E201]: use of moved value: `data`
  --> main.hm:8:12
   |
5  |     let other = data
   |                 ---- value moved here
8  |     process(data)
   |             ^^^^ value used here after move
   |
   = help: clone the value before moving: `let other = data.clone()`
   = note: `str` does not implement `Copy`

error[E202]: cannot borrow `s` as mutable because it is also borrowed as immutable
  --> main.hm:5:14
   |
3  |     let r1 = &s
   |              -- immutable borrow occurs here
5  |     let r2 = &mut s
   |              ^^^^^^ mutable borrow occurs here
6  |     print(r1)
   |           -- immutable borrow later used here
   |
   = help: move the immutable borrow after the mutable borrow ends

error[E203]: cannot borrow `items` as mutable more than once at a time
  --> main.hm:9:14
   |
7  |     let first = &mut items
   |                 ---------- first mutable borrow occurs here
9  |     let second = &mut items
   |                  ^^^^^^^^^^ second mutable borrow occurs here
   |
   = help: you can only have one mutable borrow at a time
```
