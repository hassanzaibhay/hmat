# HMAT Language Specification v0.2
# Type System
# Author: Hassan Zaib Hayat <hassanzaibhayatske@gmail.com>

---

## 1. Overview

HMAT is a **statically typed** language with **full type inference**.
You never have to write a type annotation, but you may always add one for clarity.
The type system is **sound** — if it compiles, it is type-safe.

Design goals:
- Inference is the default; annotations are optional
- Generics are expressive but don't require `PhantomData` or lifetime annotations in common use
- Errors are values — no implicit exceptions
- The compiler always tells you *what* is wrong and *how* to fix it

---

## 2. Primitive Types

### 2.1 Integer Types

| Type   | Size    | Range                                    | Notes            |
|--------|---------|------------------------------------------|------------------|
| `i8`   | 8-bit   | -128 to 127                              |                  |
| `i16`  | 16-bit  | -32,768 to 32,767                        |                  |
| `i32`  | 32-bit  | -2,147,483,648 to 2,147,483,647          |                  |
| `i64`  | 64-bit  | -9.2e18 to 9.2e18                        |                  |
| `i128` | 128-bit | very large                               |                  |
| `int`  | 64-bit  | alias for `i64`                          | **default**      |
| `u8`   | 8-bit   | 0 to 255                                 |                  |
| `u16`  | 16-bit  | 0 to 65,535                              |                  |
| `u32`  | 32-bit  | 0 to 4,294,967,295                       |                  |
| `u64`  | 64-bit  | 0 to 1.8e19                              |                  |
| `u128` | 128-bit | very large                               |                  |
| `uint` | 64-bit  | alias for `u64`                          |                  |
| `byte` | 8-bit   | alias for `u8`                           | for raw bytes    |

Integer literals default to `int` unless context requires a different type.

Integer overflow: **panic in debug mode, wrapping in release mode** (configurable).

### 2.2 Float Types

| Type    | Size    | Precision        | Notes       |
|---------|---------|------------------|-------------|
| `f32`   | 32-bit  | ~7 decimal digits|             |
| `f64`   | 64-bit  | ~15 decimal digits|            |
| `float` | 64-bit  | alias for `f64`  | **default** |

Float literals default to `float` unless annotated.

### 2.3 Boolean

```hmat
let t: bool = true
let f: bool = false
```

### 2.4 Character and String

| Type   | Description                                      |
|--------|--------------------------------------------------|
| `char` | A single Unicode scalar value (4 bytes)          |
| `str`  | UTF-8 encoded, heap-allocated, owned string      |

String operations:
```hmat
let s: str = "hello"
let len = s.length()          # 5
let upper = s.upper()         # "HELLO"
let concat = s + " world"     # "hello world"
let ch: char = s[0]           # 'h'
let sub = s[1..3]             # "el"
let interpolated = f"{s} world"
```

### 2.5 Unit Type

`()` — the unit type. Returned by functions that produce no value.

```hmat
fn print_greeting(name: str):   # implicitly returns ()
    print(f"Hello, {name}!")
```

---

## 3. Compound Types

### 3.1 Tuples

Fixed-size, heterogeneous. Indexed by position.

```hmat
let pair: (int, str) = (42, "hello")
let first = pair.0
let second = pair.1

# Destructuring
let (x, y) = (1, 2)
let (name, age, active) = ("Alice", 30, true)
```

### 3.2 Arrays

**Fixed-size arrays:** stack-allocated, size known at compile time.
```hmat
let arr: [int; 5] = [1, 2, 3, 4, 5]
let first = arr[0]
let len = arr.length()    # 5, compile-time known
```

**Dynamic arrays:** heap-allocated, resizable.
```hmat
let list: [int] = [1, 2, 3]
list.push(4)
list.pop()
let len = list.length()

# List comprehension
let squares: [int] = [x^2 for x in 1..=10]
let evens = [x for x in list if x % 2 == 0]
```

### 3.3 Maps (Hash Maps)

```hmat
let scores: {str: int} = { "alice": 100, "bob": 95 }
scores["charlie"] = 88
let alice_score = scores["alice"]          # int — panics if missing
let bob_score = scores.get("bob")          # Option<int> — safe
let unknown = scores.get("dave").unwrap_or(0)
```

### 3.4 Sets

```hmat
let s: {int} = {1, 2, 3, 4, 5}
s.add(6)
let has_3 = s.contains(3)    # true
```

---

## 4. Option and Result

These are built into the language — not just library types.

### 4.1 Option<T>

Represents a value that may or may not be present. Replaces null.

```hmat
enum Option<T>:
    Some(T)
    None

# Creating
let maybe: Option<int> = Some(42)
let nothing: Option<int> = None

# Using
match maybe:
    Some(n) -> print(f"Got {n}")
    None    -> print("Nothing")

# Shortcuts
let val = maybe.unwrap()           # panics if None
let val = maybe.unwrap_or(0)       # default if None
let val = maybe.unwrap_or_else(|| compute_default())
let doubled = maybe.map(|n| n * 2)     # Option<int>
let chained = maybe.and_then(|n| if n > 0: Some(n) else: None)
```

### 4.2 Result<T, E>

Represents an operation that can succeed or fail.

```hmat
enum Result<T, E>:
    Ok(T)
    Err(E)

# Creating
fn divide(a: float, b: float) -> Result<float, DivisionError>:
    if b == 0.0:
        return Err(DivisionError.ZeroDivision)
    return Ok(a / b)

# Using
match divide(10.0, 2.0):
    Ok(result) -> print(f"Result: {result}")
    Err(e)     -> print(f"Error: {e}")

# ? operator — propagate error upward
fn compute(input: str) -> Result<float, AppError>:
    let num = parse_float(input)?    # returns Err(AppError) if parse fails
    let result = divide(num, 2.0)?
    return Ok(result)

# Combinators
let doubled = result.map(|n| n * 2)
let fallback = result.unwrap_or(0.0)
let mapped_err = result.map_err(|e| AppError::from(e))
```

---

## 5. Type Inference

The compiler infers types from context. Inference is bidirectional.

```hmat
let x = 42              # inferred: int
let y = 3.14            # inferred: float
let s = "hello"         # inferred: str
let b = true            # inferred: bool

let nums = [1, 2, 3]    # inferred: [int]
let pairs = [(1, "a"), (2, "b")]  # inferred: [(int, str)]

fn identity<T>(x: T) -> T:
    return x

let result = identity(42)    # T inferred as int
let result = identity("hi")  # T inferred as str
```

When inference fails, the compiler gives a specific error:
```
error[E101]: cannot infer type for `x`
  --> main.hm:3:5
   |
3  |     let x = []
   |         ^ type annotation needed
   |
   = help: add a type annotation: `let x: [int] = []`
```

---

## 6. Generics

### 6.1 Generic Functions

```hmat
fn max<T: Comparable>(a: T, b: T) -> T:
    return a if a > b else b

fn first<T>(list: [T]) -> Option<T>:
    if list.length() == 0:
        return None
    return Some(list[0])
```

### 6.2 Generic Structs

```hmat
struct Pair<A, B>:
    first: A
    second: B

    fn swap(self) -> Pair<B, A>:
        return Pair { first: self.second, second: self.first }

let p = Pair { first: 1, second: "hello" }
let swapped = p.swap()  # Pair<str, int>
```

### 6.3 Trait Bounds

Single bound:
```hmat
fn print_all<T: Printable>(items: [T]):
    for item in items:
        item.print()
```

Multiple bounds:
```hmat
fn process<T: Serializable + Comparable + Clone>(item: T) -> str:
    return item.to_json()
```

Where clause (for complex bounds):
```hmat
fn transform<T, U>(items: [T]) -> [U]
    where T: Into<U>, U: Default:
    return [item.into() for item in items]
```

### 6.4 Monomorphization

Generics are compiled via monomorphization — the compiler generates a specialized version for each concrete type used. No runtime overhead.

```hmat
max(1, 2)         # generates max_int(a: int, b: int) -> int
max(1.0, 2.0)     # generates max_float(a: float, b: float) -> float
```

---

## 7. Built-in Traits

These traits are implemented automatically or have special compiler support:

| Trait        | Methods                          | Auto-derived? | Notes                        |
|--------------|----------------------------------|---------------|------------------------------|
| `Clone`      | `clone(self) -> Self`            | `@derive`     | Deep copy                    |
| `Copy`       | (marker)                         | `@derive`     | Bitwise copy, no move        |
| `Debug`      | `debug(self) -> str`             | `@derive`     | Debug representation         |
| `Display`    | `display(self) -> str`           | manual        | Human-readable representation|
| `Comparable` | `compare(self, other: Self)->int`| manual        | Enables `<`, `>`, etc.       |
| `Equals`     | `equals(self, other: Self)->bool`| `@derive`     | Enables `==`, `!=`           |
| `Hash`       | `hash(self) -> uint`             | `@derive`     | For use in maps/sets         |
| `Into<T>`    | `into(self) -> T`                | manual        | Type conversion              |
| `From<T>`    | `from(val: T) -> Self`           | manual        | Type conversion (other dir)  |
| `Default`    | `default() -> Self`              | `@derive`     | Zero-like value              |
| `Serialize`  | `to_json(self) -> str`           | `@derive`     | JSON serialization           |

### Derive Attribute

```hmat
@derive(Clone, Debug, Equals, Hash)
struct Point:
    x: float
    y: float
```

---

## 8. Type Coercion

HMAT does **not** do implicit numeric coercion. All conversions are explicit.

```hmat
let x: i32 = 42
let y: i64 = x as i64      # explicit cast
let z: float = x as float

# String conversion
let s = x.to_str()
let n = "42".parse::<int>()    # Result<int, ParseError>
```

The `as` keyword is for lossless or safe casts.  
For lossy casts (e.g., `i64 → i32`), use `.truncate()` to make the loss explicit.

---

## 9. AI Types

These are first-class types for the AI-native features:

```hmat
# Model — a handle to an AI model
let m: Model = load("anthropic/claude-3-5-sonnet")

# Calling the model produces typed outputs
let reply: str = await m.chat("Hello")
let code: HmatCode = await m.generate_code("sort a list")
let summary: str = await m.summarize(long_text)

# HmatCode — a safe, typed representation of generated HMAT code
# Cannot be executed directly without an explicit eval() in unsafe context
let code: HmatCode = await model.generate_code("compute fibonacci")
# code.source is a str — readable
# eval(code) is only allowed in unsafe blocks
```

---

## 10. Type Error Examples

```
error[E100]: type mismatch
  --> main.hm:5:18
   |
5  |     let x: int = "hello"
   |                  ^^^^^^^ expected `int`, found `str`
   |
   = help: remove the type annotation to let inference work, or change the value

error[E102]: binary operation `+` not defined for types `int` and `str`
  --> main.hm:8:15
   |
8  |     let sum = 1 + "two"
   |                 ^ no implementation of `Add<str>` for `int`
   |
   = help: convert the string to an int: `"two".parse::<int>()?`

error[E103]: cannot use `None` where `Option<int>` is expected without annotation
  --> main.hm:12:15
   |
12 |     let x = None
   |             ^^^^ type of `None` is ambiguous
   |
   = help: add a type annotation: `let x: Option<int> = None`
```
