# HMAT Language Specification v0.3
# Grammar — Lexical Rules and Syntax
# Author: Hassan Zaib Hayat <hassanzaibhayatske@gmail.com>
#
# HMAT has its own unique identity.
# It does not look like Python, Rust, C++, Java, Go, or any existing language.
# The user provides only what the compiler has no way of inferring.
# The compiler does maximum work.

---

## 1. What Makes HMAT Unique

HMAT's identity comes from its overall feel and philosophy, not from removing
every keyword. The language has its own character through its combination of:
clean error handling, first-class AI/DS, expressive pattern matching, and a
stdlib that covers what other languages need five packages to achieve.

| Construct    | Common languages           | HMAT                  |
|--------------|----------------------------|-----------------------|
| Functions    | fn/def/func                | fn (kept, familiar)   |
| Variables    | let/var/val                | let (kept, familiar)  |
| Mutability   | Implicit or var            | mut — explicit        |
| Structs      | struct/class               | shape                 |
| Enums        | enum                       | type                  |
| Pattern match| match/switch/when/case     | on                    |
| Match arms   | -> or :                    | =>                    |
| Fallible type| Result<T,E> / throws       | T or Fail             |
| Optional type| Option<T> / T? / nullable  | T or nil              |
| Raise error  | throw/raise/return Err     | fail                  |
| Default value| unwrap_or / ?: / or_else   | or                    |
| Generics     | <T>                        | [T]                   |
| Closures     | |x| / lambda x / x ->     | x => expr             |
| Pipelines    | (various)                  | flow                  |
| Nil value    | null/None/nil              | nil                   |
| AI model     | library import             | ai — first-class      |
| Tensor       | import numpy               | tensor — first-class  |
| DataFrame    | import pandas              | frame — first-class   |
| Neural net   | import torch               | nn — first-class      |

---

## 2. Lexical Rules

### 2.1 Whitespace and Comments

```
WHITESPACE ::= ' ' | '\t' | '\r'
NEWLINE    ::= '\n'
COMMENT    ::= '#' { any_char_except_newline }
```

Newlines are significant. Comments stripped before parsing.

### 2.2 Indentation (Significant Whitespace)

4-space standard. Tabs normalized to 4 spaces.
Lexer post-processes newlines into INDENT/DEDENT tokens:

```
After NEWLINE, count leading spaces on next non-empty/non-comment line:
  count > current  → INDENT, push count
  count == current → nothing
  count < current  → DEDENT for each level popped
```

### 2.3 Keywords

```
fn       let      mut      async    await    return   if       else     elif
on       for      in       while    break    continue
shape    type     flow     ai       model    fail     or
and      not      is       nil      true     false
unsafe   import   from     as       pub      self
```

### 2.4 Identifiers

```
IDENTIFIER ::= (ALPHA | '_') { ALPHA | DIGIT | '_' }
```

Conventions (formatter-enforced):
- Variables, functions, params: snake_case
- Shapes, Types: PascalCase
- Constants: SCREAMING_SNAKE_CASE

### 2.5 Literals

```
INT_LITERAL   ::= DIGIT { DIGIT | '_' }
                | '0x' HEX { HEX | '_' }
                | '0b' BIN { BIN | '_' }
                | '0o' OCT { OCT | '_' }

FLOAT_LITERAL ::= DIGIT { DIGIT } '.' { DIGIT }
                  [ ('e'|'E') ['+'|'-'] DIGIT { DIGIT } ]

STRING_LITERAL ::= '"' { string_char } '"'
F_STRING       ::= 'f"' { string_char | '{' expression '}' } '"'
BOOL_LITERAL   ::= 'true' | 'false'
NIL_LITERAL    ::= 'nil'

ESCAPE_SEQ     ::= '\n' | '\t' | '\r' | '\\' | '\"' | '\0' | '\x' HH
```

Numeric separators: `1_000_000` valid, `_1000` invalid.

### 2.6 Operators

```
Arithmetic:    +  -  *  /  %  ^
Comparison:    == != <  <= >  >=
Logical:       and  or  not
Bitwise:       &  |  ~  <<  >>
Assignment:    =  +=  -=  *=  /=  %=
Return type:   ->
Match / closure arm:  =>
Range:         ..   ..=
Member access: .
```

### 2.7 Delimiters

```
(  )   parentheses
[  ]   brackets (collections AND generics)
{  }   braces (f-strings and map/set literals only)
:      type annotation, block start
,      separator
```

---

## 3. Grammar Rules

### 3.1 Top Level

```
program        ::= { top_level_item }
top_level_item ::= function_decl
                 | shape_decl
                 | type_decl
                 | flow_decl
                 | ai_decl
                 | import_stmt
                 | constant_decl
                 | NEWLINE
```

### 3.2 Imports

```
import_stmt ::= 'import' module_path NEWLINE
              | 'from' module_path 'import' import_list NEWLINE
module_path ::= IDENTIFIER { '.' IDENTIFIER }
import_list ::= IDENTIFIER ['as' IDENTIFIER] { ',' IDENTIFIER ['as' IDENTIFIER] }
```

```hmat
import hmat.io
from hmat.collections import Map, Set
from hmat.ai import ChatMessage as Msg
```

### 3.3 Constants (Top-Level Bindings)

```
constant_decl ::= IDENTIFIER '=' expression NEWLINE
```

Top-level bindings without `mut` are constants (must be compile-time expressions).

### 3.4 Functions

```
function_decl ::= [ 'pub' ] [ 'async' ] 'fn' IDENTIFIER
                  [ generic_params ]
                  '(' [ param_list ] ')'
                  [ '->' return_type ]
                  ':' NEWLINE
                  INDENT { statement } DEDENT

# Single-line shorthand
function_decl ::= [ 'pub' ] [ 'async' ] 'fn' IDENTIFIER '(' [ param_list ] ')' ':' expression NEWLINE

param_list    ::= param { ',' param }
param         ::= [ 'mut' ] IDENTIFIER [ ':' type ]
                | 'self' | 'mut' 'self'

generic_params ::= '[' generic_param { ',' generic_param } ']'
generic_param  ::= IDENTIFIER [ ':' trait_bound ]
trait_bound    ::= IDENTIFIER { '+' IDENTIFIER }

return_type   ::= type
                | type 'or' 'Fail'
                | type 'or' 'nil'
                | type 'or' 'Fail' 'or' 'nil'
```

Examples:
```hmat
# No types — fully inferred
fn greet(name): "Hello, {name}!"

# With types
fn add(a: int, b: int) -> int: a + b

# Multi-line
fn divide(a: float, b: float) -> float or Fail:
    fail "zero" if b == 0.0
    a / b

# Generic
fn max[T: Comparable](a: T, b: T) -> T:
    a if a > b else b

# Async fallible
async fn fetch(url: str) -> str or Fail:
    resp = await http.get(url)
    resp.text()
```

### 3.5 Shape Declarations (Data Structures)

```
shape_decl  ::= [ 'pub' ] 'shape' IDENTIFIER [ generic_params ] ':'
                NEWLINE INDENT
                { shape_field | function_decl | NEWLINE }
                DEDENT

shape_field ::= [ 'pub' ] IDENTIFIER ':' type NEWLINE
```

Examples:
```hmat
shape Point:
    x: float
    y: float

    fn distance(self, other: Point) -> float:
        dx = self.x - other.x
        dy = self.y - other.y
        (dx^2 + dy^2).sqrt()

shape Stack[T]:
    data: [T]
    mut size: int

    fn push(mut self, value: T):
        self.data.push(value)
        self.size += 1

    fn pop(mut self) -> T or nil:
        self.data.pop()
```

### 3.6 Type Declarations (Sum Types)

```
type_decl   ::= [ 'pub' ] 'type' IDENTIFIER [ generic_params ]
                '=' variant { '|' variant } NEWLINE       # inline
              | [ 'pub' ] 'type' IDENTIFIER [ generic_params ] ':'
                NEWLINE INDENT { type_variant NEWLINE } DEDENT  # block

type_variant ::= IDENTIFIER [ '(' field_or_type_list ')' ]
field_or_type_list ::= named_fields | type_list
named_fields ::= IDENTIFIER ':' type { ',' IDENTIFIER ':' type }
type_list    ::= type { ',' type }
```

Examples:
```hmat
# Inline — simple
type Direction = North | South | East | West

# Block — with payloads
type Shape:
    Circle(radius: float)
    Rect(width: float, height: float)
    Triangle(base: float, height: float)

# Generic
type Tree[T]:
    Leaf(T)
    Node(left: Tree[T], right: Tree[T])
```

### 3.7 Pattern Matching — 'on'

```
on_stmt  ::= 'on' expression ':' NEWLINE INDENT { on_arm } DEDENT

on_arm   ::= pattern '=>' ( expression | block ) NEWLINE

pattern  ::= literal_pattern
           | variant_pattern
           | type_pattern
           | nil_pattern
           | wildcard_pattern
           | guard_pattern

literal_pattern  ::= INT_LITERAL | FLOAT_LITERAL | STRING_LITERAL | BOOL_LITERAL
variant_pattern  ::= IDENTIFIER [ '(' pattern_list ')' ]
type_pattern     ::= type 'as' IDENTIFIER
nil_pattern      ::= 'nil'
wildcard_pattern ::= '_'
guard_pattern    ::= pattern 'if' expression
pattern_list     ::= pattern { ',' pattern }
```

Examples:
```hmat
# Literal
on score:
    100      => "perfect"
    90..=99  => "excellent"
    _        => "ok"

# Type variants — no :: prefix, just the name
on shape:
    Circle(r)      => 3.14 * r^2
    Rect(w, h)     => w * h
    Triangle(b, h) => 0.5 * b * h

# Error and nil handling
on divide(10, 0):
    float as n => print("got {n}")
    Fail  as e => print("failed: {e}")

# Guards
on value:
    n if n > 100 => "big"
    n if n > 0   => "positive"
    _            => "other"
```

### 3.8 AI Declarations

```
ai_decl ::= 'ai' IDENTIFIER '=' 'model' '(' STRING_LITERAL ')'
            [ ':' NEWLINE INDENT { IDENTIFIER ':' expression NEWLINE } DEDENT ] NEWLINE
```

```hmat
ai assistant = model("anthropic/claude-3-5-sonnet")

ai gpt = model("openai/gpt-4o"):
    temperature: 0.7
    max_tokens: 2048
```

### 3.9 Flow Declarations (Pipelines)

```
flow_decl ::= 'flow' IDENTIFIER ':'
              NEWLINE INDENT
              flow_stage { '=>' flow_stage } NEWLINE
              DEDENT

flow_stage ::= IDENTIFIER | function_call
```

```hmat
flow analyze:
    input => clean => tokenize => embed => classify
```

---

## 4. Statements

```
statement ::= binding_stmt
            | mut_decl
            | assignment_stmt
            | return_stmt
            | fail_stmt
            | on_stmt
            | if_stmt
            | for_stmt
            | while_stmt
            | break_stmt
            | continue_stmt
            | unsafe_block
            | expression_stmt
            | NEWLINE

binding_stmt    ::= IDENTIFIER '=' expression NEWLINE
mut_decl        ::= 'mut' IDENTIFIER [ ':' type ] '=' expression NEWLINE
assignment_stmt ::= lvalue compound_op expression NEWLINE
return_stmt     ::= 'return' [ expression ] NEWLINE
fail_stmt       ::= 'fail' expression [ 'if' expression ] NEWLINE
expression_stmt ::= expression NEWLINE
lvalue          ::= IDENTIFIER { '.' IDENTIFIER | '[' expression ']' }
compound_op     ::= '+=' | '-=' | '*=' | '/=' | '%='
```

### 4.1 If Statement

```
if_stmt ::= 'if' expression ':' NEWLINE INDENT { statement } DEDENT
            { 'elif' expression ':' NEWLINE INDENT { statement } DEDENT }
            [ 'else' ':' NEWLINE INDENT { statement } DEDENT ]
```

Inline ternary:
```hmat
label = "even" if x % 2 == 0 else "odd"
fail "zero" if b == 0
```

### 4.2 For / While

```hmat
for item in collection:
    print(item)

for i in 0..10:
    print(i)

while x > 0:
    x -= 1
```

### 4.3 Unsafe Block

```hmat
# SAFETY: pointer guaranteed non-null (checked above)
# INVARIANT: alignment is always 8 bytes
unsafe:
    val = raw_ptr(data)
```

---

## 5. Expressions

```
expression       ::= ternary_expr
ternary_expr     ::= or_expr [ 'if' expression 'else' expression ]
or_expr          ::= and_expr { 'or' and_expr }    # logical OR + fallback
and_expr         ::= not_expr { 'and' not_expr }
not_expr         ::= [ 'not' ] equality_expr
equality_expr    ::= comparison_expr { ('=='|'!=') comparison_expr }
comparison_expr  ::= range_expr { ('<'|'<='|'>'|'>=') range_expr }
range_expr       ::= additive_expr [ ('..'|'..=') additive_expr ]
additive_expr    ::= multiplicative_expr { ('+'|'-') multiplicative_expr }
multiplicative_expr ::= unary_expr { ('*'|'/'|'%') unary_expr }
unary_expr       ::= [ '-' | '~' ] power_expr
power_expr       ::= await_expr [ '^' unary_expr ]
await_expr       ::= 'await' postfix_expr | postfix_expr 'await'
postfix_expr     ::= primary_expr { postfix_op }
postfix_op       ::= '.' IDENTIFIER
                   | '(' [ arg_list ] ')'
                   | '[' expression ']'
primary_expr     ::= IDENTIFIER | literal | f_string
                   | shape_literal | list_expr | map_expr | set_expr
                   | closure_expr | '(' expression ')'
```

### 5.1 The `or` Operator

Context-sensitive — compiler resolves:

```hmat
# Fallback (Fail or nil context)
result = divide(10, 0) or 0.0
name   = get_user()    or "anonymous"
val    = first([])     or -1

# Logical (boolean context)
ok = is_admin or is_owner
```

### 5.2 Shape Literals

```hmat
p = Point { x: 1.0, y: 2.0 }
p = Point { x, y }    # shorthand — variable names match fields
```

### 5.3 Collections

```hmat
nums  = [1, 2, 3, 4, 5]
range = [1..10]
evens = [x for x in nums if x % 2 == 0]
scores = { "alice": 100, "bob": 95 }
primes = {2, 3, 5, 7}
```

### 5.4 Closures

```hmat
double   = x => x * 2
add      = (a, b) => a + b
no_args  = () => 42
```

### 5.5 AI Method Calls

```hmat
reply   = assistant.ask("What is HMAT?")
summary = assistant.think(document)
code    = assistant.gen("binary search")
label   = assistant.classify(text, labels: ["pos", "neg"])
vec     = assistant.embed("hello world")
```

---

## 6. Types

```
type          ::= primitive_type | named_type | generic_type
                | list_type | map_type | set_type | tuple_type
                | function_type | fallible_type | nilable_type
                | ai_type

primitive_type ::= 'int'|'i8'|'i16'|'i32'|'i64'|'i128'
                 | 'uint'|'u8'|'u16'|'u32'|'u64'|'u128'
                 | 'float'|'f32'|'f64'
                 | 'bool'|'str'|'char'|'byte'|'()'

generic_type  ::= IDENTIFIER '[' type_list ']'   # [T] not <T>
list_type     ::= '[' type ']'
map_type      ::= '{' type ':' type '}'
set_type      ::= '{' type '}'
fallible_type ::= type 'or' 'Fail'
nilable_type  ::= type 'or' 'nil'
ai_type       ::= 'Model' | 'Flow' | 'GenCode'
```

Defaults: `int` = i64, `float` = f64.

---

## 7. Operator Precedence (High to Low)

| Level | Operators                   | Assoc   |
|-------|-----------------------------|---------|
| 1     | `()` `[]` `.`               | Left    |
| 2     | `^`                         | Right   |
| 3     | `-` `~` (unary)             | Prefix  |
| 4     | `*` `/` `%`                 | Left    |
| 5     | `+` `-`                     | Left    |
| 6     | `..` `..=`                  | Non     |
| 7     | `<` `<=` `>` `>=`           | Non     |
| 8     | `==` `!=`                   | Left    |
| 9     | `&`                         | Left    |
| 10    | `\|`                        | Left    |
| 11    | `and`                       | Left    |
| 12    | `or`                        | Left    |
| 13    | ternary (`if`/`else`)       | Right   |
| 14    | `=` `+=` etc.               | Right   |

---

## 8. Error Model

No exceptions. No Result<T,E>. Errors are values that propagate automatically.

```hmat
# Declare that a function can fail
divide(a: float, b: float) -> float or Fail:
    fail "zero" if b == 0.0
    a / b

# Errors propagate automatically — no ? needed
process(input: str) -> float or Fail:
    n = parse_float(input)     # auto-propagates if fails
    divide(n, 2.0)             # auto-propagates if fails

# Handle with 'or' (default)
result = divide(10, 0) or 0.0

# Handle with 'on' (explicit)
on divide(10, 0):
    float as n => use(n)
    Fail  as e => log(e)
```

---

## 9. Compiler Error Format

```
error[EXXXX]: <message>
  --> file.hm:line:col
   |
N  | source line
   | ^^^^^^^^^^^ what this is
   |
   = help: what to do
   = note: optional context
```

Ranges: E001-E099 syntax, E100-E199 types, E200-E299 ownership,
        E300-E399 AI, E400-E499 modules, W001-W099 warnings.

---

## 10. Design Decision Log

| ID  | Decision                  | Choice          | Rationale                                      |
|-----|---------------------------|-----------------|------------------------------------------------|
| D001| Function keyword          | fn              | Familiar, readable, no reason to remove        |
| D002| Variable keyword          | let (optional)  | let is clean; bare assignment also works       |
| D003| Struct keyword            | shape           | Unique, describes data shape, not borrowed     |
| D004| Enum keyword              | type            | Sum types, not enumeration — more accurate     |
| D005| Pattern match keyword     | on              | Short, unique, reads naturally                 |
| D006| Match arm separator       | =>              | Distinct from return type ->                   |
| D007| Fallible return type      | T or Fail       | English-readable, no type wrapping noise       |
| D008| Optional return type      | T or nil        | English-readable, no Option<T> noise           |
| D009| Error fallback            | or              | Universal, reads as natural English            |
| D010| Raise error               | fail            | Short, unique, clear intent                    |
| D011| Generic brackets          | [T] not <T>     | No confusion with comparison operators         |
| D012| Closure style             | x => expr       | Universal arrow, known from math               |
| D013| Pipeline keyword          | flow            | Unique, data flows through stages              |
| D014| Nil value                 | nil             | Short, unique to HMAT                          |
| D015| Error propagation         | Automatic       | No ? operator — compiler handles it            |
| D016| Borrows                   | Invisible       | Compiler tracks — user never writes &          |
| D017| Significant indentation   | Yes             | Enforces readability                           |
| D018| No semicolons             | Yes             | Newlines are sufficient                        |
