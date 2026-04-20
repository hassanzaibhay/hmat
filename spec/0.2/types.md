# HMAT Language Specification v0.3
# Type System
# Author: Hassan Zaib Hayat <hassanzaibhayatske@gmail.com>

---

## 1. Overview

HMAT is statically typed with full inference.
Type annotations are optional — write them for clarity, not for the compiler.
The type system is sound: if it compiles, it is type-safe.

No null. No unchecked errors. No implicit coercion.
The compiler tracks everything; the user annotates nothing unless they want to.

---

## 2. Primitive Types

| Type    | Size  | Notes                  |
|---------|-------|------------------------|
| `int`   | 64-bit| default integer        |
| `i8`..`i128` | — | explicit sizes        |
| `uint`  | 64-bit| default unsigned       |
| `u8`..`u128` | — | explicit sizes        |
| `float` | 64-bit| default float          |
| `f32`   | 32-bit| explicit               |
| `f64`   | 64-bit| same as float          |
| `bool`  | —     | true / false           |
| `str`   | —     | UTF-8 heap string      |
| `char`  | 4-byte| Unicode scalar         |
| `byte`  | 8-bit | alias for u8           |
| `()`    | —     | unit (no value)        |

Integer overflow: panic in debug, wrapping in release (configurable).

---

## 3. Compound Types

### 3.1 Lists

```hmat
nums: [int] = [1, 2, 3]
nums.push(4)
nums.pop()               # returns int or nil
len = nums.length()
first = nums[0]          # int — panics if empty
safe = nums.get(0)       # int or nil — safe
```

Fixed-size arrays: `[int; 5]` — stack allocated, size known at compile time.

### 3.2 Maps

```hmat
scores: {str: int} = { "alice": 100 }
scores["bob"] = 95
val = scores["alice"]        # int — panics if missing
val = scores.get("charlie")  # int or nil — safe
```

### 3.3 Sets

```hmat
primes: {int} = {2, 3, 5, 7}
primes.add(11)
has = primes.contains(3)
```

### 3.4 Tuples

```hmat
pair: (int, str) = (42, "hello")
x = pair.0
y = pair.1

# Destructuring
(a, b) = (1, 2)
```

---

## 4. Fallible and Nilable Types

These replace `Result<T,E>` and `Option<T>`. No wrapper types.
The type annotation describes the shape of the value, not a container around it.

### 4.1 Fallible: `T or Fail`

A function that might fail returns `T or Fail`.
At the call site, either handle it or provide a default with `or`.

```hmat
divide(a: float, b: float) -> float or Fail:
    fail "division by zero" if b == 0.0
    a / b

# Default on failure
result = divide(10, 0) or 0.0

# Explicit handling
on divide(10, 0):
    float as n => use(n)
    Fail  as e => log(e)

# Automatic propagation — no ? needed
process(input: str) -> float or Fail:
    n = parse_float(input)    # if this fails, process fails too — automatic
    divide(n, 2.0)
```

### 4.2 Nilable: `T or nil`

A value that might not exist. No null. Just `nil`.

```hmat
first[T](list: [T]) -> T or nil:
    nil if list.is_empty() else list[0]

# Default
val = first([]) or -1

# Explicit handling
on first([]):
    int as n => print("found {n}")
    nil      => print("empty")
```

### 4.3 Both: `T or Fail or nil`

Rare. Only when a function can fail AND legitimately return nil.

```hmat
find_user(id: int) -> User or Fail or nil:
    fail "db error" if not db.connected()
    db.users.get(id)    # nil if not found
```

---

## 5. Type Inference

The compiler infers types from context. Bidirectional inference.

```hmat
x = 42          # int
y = 3.14        # float
s = "hello"     # str
b = true        # bool
nums = [1,2,3]  # [int]

result = max(3, 7)      # T inferred as int
result = max(1.0, 2.0)  # T inferred as float
```

When inference fails, the error is specific:
```
error[E101]: cannot infer type for `x`
  --> main.hm:3:1
   |
3  | x = []
   | ^ type annotation needed
   |
   = help: add annotation: `x: [int] = []`
```

---

## 6. Generics — `[T]` Not `<T>`

Square brackets are used for both collections and generics.
This is intentional — generics ARE collections of types.

```hmat
# Generic function
max[T: Comparable](a: T, b: T) -> T:
    a if a > b else b

# Generic shape
shape Pair[A, B]:
    first: A
    second: B

    swap(self) -> Pair[B, A]:
        Pair { first: self.second, second: self.first }

# Multiple bounds
process[T: Serializable + Comparable](item: T) -> str:
    item.to_json()
```

Generics compile via monomorphization — zero runtime overhead.

---

## 7. Shapes and Types

### Shapes (Data Structures)

```hmat
shape Point:
    x: float
    y: float
```

Shapes are value types by default (copied on assignment unless large).
For heap allocation, wrap in a reference — compiler decides (user doesn't annotate).

### Types (Sum Types)

```hmat
type Color = Red | Green | Blue

type Shape:
    Circle(radius: float)
    Rect(width: float, height: float)

type Tree[T]:
    Leaf(T)
    Node(left: Tree[T], right: Tree[T])
```

---

## 8. Built-In Traits

Auto-derived with `@derive`:

```hmat
@derive(Clone, Debug, Eq, Hash)
shape Point:
    x: float
    y: float
```

| Trait        | Methods                       | Auto?    |
|--------------|-------------------------------|----------|
| `Clone`      | `clone(self) -> Self`         | @derive  |
| `Copy`       | (marker)                      | @derive  |
| `Debug`      | `debug(self) -> str`          | @derive  |
| `Display`    | `display(self) -> str`        | manual   |
| `Comparable` | `compare(self, Self) -> int`  | manual   |
| `Eq`         | `eq(self, Self) -> bool`      | @derive  |
| `Hash`       | `hash(self) -> uint`          | @derive  |
| `Default`    | `default() -> Self`           | @derive  |
| `Serialize`  | `to_json(self) -> str`        | @derive  |

---

## 9. Type Coercion

HMAT does NOT do implicit numeric coercion. All conversions are explicit.

```hmat
x: i32 = 42
y = x as i64        # explicit
z = x as float      # explicit
s = x.to_str()      # to string
n = "42".parse_int() or 0   # from string
```

---

## 10. AI Types

```hmat
# Model — handle to an AI model
assistant: Model = model("anthropic/claude-3-5-sonnet")

# GenCode — typed AI-generated HMAT code (safe, not raw string)
code: GenCode = assistant.gen("sort a list by length")
code.source          # the HMAT source as str
code.functions       # list of function names

# Executing GenCode requires unsafe
unsafe:
    result = eval(code, input: data)
```

---

## 11. Type Error Examples

```
error[E100]: type mismatch
  --> main.hm:5:7
   |
5  | x: int = "hello"
   |          ^^^^^^^ expected int, found str
   |
   = help: remove the annotation, or change the value

error[E102]: + not defined for int and str
  --> main.hm:8:12
   |
8  | sum = 1 + "two"
   |         ^ no Add[str] for int
   |
   = help: convert str to int: "two".parse_int() or 0

error[E103]: type of nil binding is ambiguous
  --> main.hm:12:5
   |
12 | x = nil
   |     ^^^ cannot infer type
   |
   = help: add annotation: x: int or nil = nil
```
