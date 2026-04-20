# HMAT Syntax Guide

A complete reference for HMAT syntax. Every construct in the language, in one document.

---

## Source Files

- Extension: `.hm`
- Encoding: UTF-8
- Line endings: LF or CRLF (normalized to LF)

---

## Comments

```hmat
# This is a comment — everything from # to end of line is ignored
let x = 1    # inline comments work too
```

HMAT has only line comments. Block comments (`/* */`) do not exist.

---

## Significant Whitespace

HMAT uses indentation to define blocks — like Python, but stricter.

- **4 spaces** is the canonical indent unit. Tabs are accepted (counted as 4 columns) but `hfmt` normalizes them.
- Blank lines and comment-only lines inside a block are ignored for indentation purposes.
- Indentation level must match a previously opened level when decreasing.

```hmat
fn example():
    let x = 1        # indented 4 spaces — inside fn body
    if x > 0:
        print("yes") # indented 8 spaces — inside if body
    print("done")    # back at 4 — if block closed
# back at 0 — fn block closed
```

---

## Statements and Expressions

Most things in HMAT are expressions — they produce a value. Statements are expressions used for their side effects.

The last expression in a block is its value (implicit return):

```hmat
fn sign(n: int) -> str:
    if n > 0:
        "positive"    # this is the return value of the if
    elif n < 0:
        "negative"
    else:
        "zero"
```

---

## Variables

```hmat
let x = 42             # immutable binding, type inferred (int)
let y: float = 3.14    # explicit type annotation
let mut z = 0          # mutable — can be reassigned

z = z + 1              # assignment to mutable
z += 1                 # compound assignment
```

**Bindings are immutable by default.** You must opt into mutation with `mut`.
This is a feature, not a restriction — it makes code easier to reason about.

### Supported Compound Assignments

`+=`  `-=`  `*=`  `/=`  `%=`

---

## Literals

### Integers

```hmat
let a = 42          # decimal
let b = 0xFF        # hexadecimal
let c = 0b1101      # binary
let d = 0o77        # octal
let e = 1_000_000   # underscores for readability
```

Default type: `int` (i64). Use type annotations for other sizes: `let x: i32 = 42`.

### Floats

```hmat
let pi = 3.14159
let tiny = 1.5e-10       # scientific notation
let big = 6.022e23
```

Floats require digits on both sides of the decimal: `3.14` is valid, `3.` is not.

### Strings

```hmat
let greeting = "Hello, HMAT!"
let escaped = "line one\nline two"
let hex_char = "\x41"          # 'A'
```

Escape sequences: `\n` `\t` `\r` `\\` `\"` `\'` `\0` `\xHH`

### F-Strings (Formatted Strings)

```hmat
let name = "HMAT"
let version = 2
let msg = f"Hello from {name} v{version}!"
```

Expressions inside `{}` are evaluated at runtime and converted to strings.

### Booleans and Nil

```hmat
let t = true
let f = false
let nothing = nil    # the absence of a value
```

---

## Types

### Primitive Types

| Type      | Description                         |
|-----------|-------------------------------------|
| `int`     | 64-bit signed integer (default)     |
| `float`   | 64-bit float (default)              |
| `bool`    | `true` or `false`                   |
| `str`     | UTF-8 string                        |
| `byte`    | alias for `u8`                      |
| `()`      | unit type — "returns nothing"       |

Fixed-size variants: `i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64`

### Compound Types

```hmat
let pair: (int, str) = (1, "one")     # tuple
let nums: [int] = [1, 2, 3]           # list
let map: Map<str, int> = {}           # map (stdlib)
```

### Generic Types

```hmat
let items: [T]           # list of T
let maybe: Option<T>     # Some(T) or None
let result: Result<T, E> # Ok(T) or Err(E)
```

---

## Functions

```hmat
fn name(param1: Type1, param2: Type2) -> ReturnType:
    body
```

```hmat
fn add(a: int, b: int) -> int:
    a + b

fn greet(name: str):          # returns () — omit -> ()
    print(f"Hello, {name}!")

fn always_one() -> int:
    return 1                  # explicit return is also valid
```

### Default Parameters

```hmat
fn connect(host: str, port: int = 8080):
    # ...
```

### Named Arguments

```hmat
fn rect(width: float, height: float) -> float:
    width * height

let area = rect(width: 4.0, height: 5.0)
```

### Variadic (Spread)

```hmat
fn sum(nums: ...int) -> int:
    nums.reduce(0, |acc, x| acc + x)
```

---

## Closures

```hmat
let double = |x| x * 2
let add = |x, y| x + y
let verbose = |x: int| -> int:
    let result = x * 2
    result + 1
```

Closures capture their environment by borrow:

```hmat
let factor = 3
let scale = |x| x * factor    # borrows `factor`
```

---

## Control Flow

### If / Elif / Else

```hmat
if condition:
    # ...
elif other_condition:
    # ...
else:
    # ...
```

`if` is an expression:

```hmat
let label = if score >= 90: "pass" else: "fail"
```

### Match

```hmat
match value:
    pattern1 -> expression
    pattern2 -> expression
    _        -> default_expression    # _ is wildcard
```

Match is exhaustive — the compiler rejects non-exhaustive patterns.

#### Pattern Types

```hmat
match n:
    0       -> "zero"           # literal
    1..=9   -> "single digit"   # inclusive range
    x if x > 100 -> "big"      # guard
    _       -> "other"          # wildcard

match shape:
    Shape::Circle(r)           -> 3.14 * r^2      # enum variant with data
    Shape::Rectangle(w, h)     -> w * h
    Shape::Triangle { base, height } -> 0.5 * base * height    # struct variant

match maybe_val:
    Some(v) -> v
    None    -> 0
```

### For Loops

```hmat
for i in 0..10:           # 0 to 9 (exclusive end)
    print(i)

for i in 0..=10:          # 0 to 10 (inclusive end)
    print(i)

for item in collection:   # iterate any iterable
    print(item)

for (i, item) in collection.enumerate():    # with index
    print(f"{i}: {item}")
```

### While Loops

```hmat
while condition:
    # ...

while let Some(v) = iterator.next():    # destructuring while
    process(v)
```

### Break and Continue

```hmat
for i in 0..100:
    if i == 10: break
    if i % 2 == 0: continue
    print(i)
```

---

## Structs

```hmat
struct Name:
    field1: Type1
    field2: Type2

    fn method(self) -> ReturnType:
        # self is the instance
        self.field1

    fn static_method() -> Name:
        Name { field1: value1, field2: value2 }
```

```hmat
struct Point:
    x: float
    y: float

    fn new(x: float, y: float) -> Point:
        Point { x, y }

    fn distance(self, other: &Point) -> float:
        ((self.x - other.x)^2 + (self.y - other.y)^2)

let p = Point::new(1.0, 2.0)    # static method call
let d = p.distance(&origin)      # instance method call
```

Struct update syntax:

```hmat
let p2 = Point { x: 5.0, ..p }    # same as p but x = 5.0
```

---

## Enums

```hmat
enum Direction:
    North
    South
    East
    West

enum Shape:
    Circle(float)                     # tuple variant
    Rectangle(float, float)
    Triangle { base: float, height: float }    # struct variant
```

Enums are used with `match`:

```hmat
match direction:
    Direction::North -> "up"
    Direction::South -> "down"
    Direction::East  -> "right"
    Direction::West  -> "left"
```

---

## Traits

```hmat
trait Displayable:
    fn display(self) -> str

impl Displayable for Point:
    fn display(self) -> str:
        f"({self.x}, {self.y})"
```

### Trait Bounds

```hmat
fn print_all<T: Displayable>(items: [T]):
    for item in items:
        print(item.display())

fn max<T: Comparable>(a: T, b: T) -> T:
    if a > b: a else: b

fn describe<T: Displayable + Comparable>(a: T, b: T):
    # T must implement both
```

---

## Generics

```hmat
fn identity<T>(x: T) -> T:
    x

struct Stack<T>:
    items: [T]

    fn push(mut self, item: T):
        self.items.push(item)

    fn pop(mut self) -> Option<T>:
        self.items.pop()
```

---

## Error Handling

```hmat
fn might_fail() -> Result<int, str>:
    if bad_condition:
        return Err("something went wrong")
    return Ok(42)

match might_fail():
    Ok(v)  -> print(f"got: {v}")
    Err(e) -> print(f"error: {e}")
```

The `?` operator: propagates `Err` from the enclosing function.

```hmat
fn chained() -> Result<int, str>:
    let a = might_fail()?     # returns Err immediately if Err
    let b = another()?
    return Ok(a + b)
```

`Option<T>` — a value that may or may not exist:

```hmat
fn first<T>(list: [T]) -> Option<T>:
    if list.is_empty(): None else: Some(list[0])

let val = first(nums).unwrap_or(0)    # default if None
```

---

## Ownership and Borrowing

```hmat
let s = "hello"     # owned
let r = &s          # shared borrow — read-only
let m = &mut s      # mutable borrow — one at a time
```

The compiler tracks lifetimes automatically in most cases. No explicit lifetime annotations required (unlike Rust) for common patterns.

See the [Ownership Guide](ownership-guide.md) for the full model.

---

## Imports

```hmat
import hmat::io
from hmat::collections import Vec, Map
import hmat::net as net
```

---

## Visibility

```hmat
pub struct Config:          # public
    pub host: str           # public field
    port: int               # private field (default)

pub fn connect():           # public function
    # ...
```

Everything is private by default. Add `pub` to expose it.

---

## Type Aliases

```hmat
type UserId = int
type Result<T> = Result<T, AppError>    # specialized alias
```

---

## Decorators / Attributes

```hmat
@test
fn test_add():
    assert(add(1, 2) == 3)

@derive(Debug, Clone)
struct Config:
    host: str
    port: int

@inline
fn hot_path(x: int) -> int:
    x * 2
```

---

## Unsafe

```hmat
unsafe:
    let ptr = raw_ptr(data)
    ptr.write(42)
```

`unsafe` blocks unlock raw pointer access and FFI. Everything outside an `unsafe` block is memory-safe.

---

## See Also

- [Getting Started](getting-started.md) — quickstart tutorial
- [Type System](../spec/0.2/types.md) — full type specification
- [Ownership Guide](ownership-guide.md) — memory model details
- [AI Guide](ai-guide.md) — `ai model`, `await`, `pipeline`
- [Error Handling Guide](error-handling.md) — `Result`, `Option`, `?`
- [Examples](../examples/README.md) — working programs
