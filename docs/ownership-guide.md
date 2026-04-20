# Ownership Guide

HMAT is memory-safe without a garbage collector. It achieves this through an ownership system
inspired by Rust — but deliberately less strict. The goal: get 95% of the safety with 20% of
the learning curve.

If you've used Rust, you'll recognize these concepts. If you haven't, this guide introduces
them from scratch.

---

## The Core Idea

Every value in HMAT has exactly one **owner** at a time. When the owner goes out of scope,
the value is freed. No garbage collector needed — the compiler handles it at compile time.

```hmat
fn main():
    let s = "hello"    # s owns the string
    print(s)
# s goes out of scope — string is freed here
```

---

## Ownership Transfer (Move)

Assigning a value to a new binding **moves** ownership:

```hmat
let a = [1, 2, 3]    # a owns the list
let b = a            # ownership moves to b
# print(a)           # error: a no longer owns the list
print(b)             # fine
```

This is only relevant for heap-allocated types (strings, lists, structs). Primitive types
(`int`, `float`, `bool`) are always copied.

---

## Borrowing

You can lend a value without transferring ownership using a **borrow** (`&`):

```hmat
fn print_length(s: &str):    # borrows s
    print(s.len())

let message = "Hello, HMAT!"
print_length(&message)       # lend it
print(message)               # still owned by message — fine
```

### Rules for Borrowing

1. Any number of **shared borrows** (`&`) can exist at the same time.
2. Only one **mutable borrow** (`&mut`) can exist at a time.
3. Shared borrows and mutable borrows cannot coexist.

These rules prevent data races at compile time — even in single-threaded code, they prevent
subtle bugs where one reference sees inconsistent state.

---

## Mutable Borrows

To modify a value through a borrow, use `&mut`:

```hmat
fn increment(n: &mut int):
    *n += 1              # dereference to access the value

let mut counter = 0
increment(&mut counter)
print(counter)           # 1
```

You can only pass `&mut` if the binding is declared with `mut`.

---

## Lifetime Inference

Unlike Rust, HMAT infers lifetimes automatically in the vast majority of cases.
You do not write lifetime annotations. The compiler figures it out.

```hmat
fn longest(a: &str, b: &str) -> &str:
    if a.len() > b.len(): a else: b
# Compiler infers: the returned &str lives as long as the shorter of a or b
```

In rare cases involving complex generic code, the compiler may ask for an explicit annotation.
When it does, the error message explains exactly what to write.

---

## Regions (Advanced)

HMAT adds a concept beyond Rust's lifetimes: **regions**. A region is a named scope that
groups allocations together and frees them all at once when the region exits.

```hmat
region request_scope:
    let buffer = alloc_buffer(1024)    # allocated in the region
    process(buffer)
# buffer freed here when region exits
```

Regions are most useful for server-side code where each request should clean up after itself.
They are optional — you won't need them for most programs.

---

## The `Clone` Escape Hatch

If you need to keep multiple owned copies of a value, use `.clone()`:

```hmat
let original = [1, 2, 3]
let copy = original.clone()    # explicit deep copy
# both original and copy are valid
```

Cloning is explicit — HMAT never copies silently. If it compiles without `.clone()`, no copy
happened.

---

## What HMAT Does Not Check

Compared to Rust, HMAT's ownership checker is deliberately lenient in a few areas:

- **Cyclic references** — allowed via `Rc<T>` / `Arc<T>` when needed. The compiler won't
  prevent them, but documentation notes where they can cause issues.
- **Self-referential structs** — supported with a pin mechanism, not required reading for
  most code.
- **Interior mutability** — allowed via `Cell<T>` / `RefCell<T>` when needed.

The philosophy: reject the programs that would obviously cause bugs in production. Accept
programs that are technically unsafe in theory but safe in practice for well-written code.

---

## Practical Rules to Internalize

| Situation                           | What to do                                    |
|-------------------------------------|-----------------------------------------------|
| Passing a value to a function       | Use `&` (borrow) by default                   |
| Function needs to modify the value  | Use `&mut` (mutable borrow)                   |
| Function needs to own the value     | Move it (no `&`)                              |
| Need to use a value after moving it | `.clone()` before the move                    |
| Returning a reference from a fn     | The compiler infers whether it's valid        |

---

## Common Errors and What They Mean

```
error: value moved here, used later
```
You moved ownership into a function or binding, then tried to use the original. Either
borrow with `&` or clone before moving.

```
error: cannot borrow as mutable — already borrowed as shared
```
A `&` borrow is still live when you tried to take `&mut`. Let the shared borrow expire first.

```
error: cannot borrow as mutable — not declared as mutable
```
Add `mut` to the binding declaration.

---

## See Also

- [Syntax Guide](syntax-guide.md) — borrow syntax reference
- [Ownership Spec](../spec/0.2/ownership.md) — formal model
- [Error Handling Guide](error-handling.md) — how errors interact with ownership
