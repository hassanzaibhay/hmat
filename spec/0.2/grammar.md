# HMAT Language Specification v0.2
# Grammar — Lexical Rules and Syntax
# Author: Hassan Zaib Hayat <hassanzaibhayatske@gmail.com>
#
# This is the authoritative grammar reference.
# The compiler (hmatc) must implement exactly what is written here.
# Any deviation is a compiler bug, not a language feature.
#
# Notation:
#   ::=         definition
#   |           alternation
#   [ ]         optional (zero or one)
#   { }         zero or more
#   ( )         grouping
#   ' '         literal terminal
#   UPPER       lexer token (defined in Lexical section)
#   lower       grammar rule (defined in Grammar section)

---

## 1. Source File

An HMAT source file is a UTF-8 encoded text file with the `.hm` extension.
Line endings: LF (`\n`) or CRLF (`\r\n`) — normalized to LF.
Encoding: UTF-8. Only ASCII identifiers in v0.2 (Unicode planned for v0.3).

---

## 2. Lexical Rules

### 2.1 Whitespace and Comments

```
WHITESPACE  ::= ' ' | '\t' | '\r'
NEWLINE     ::= '\n'
COMMENT     ::= '#' { any_char_except_newline }
```

Whitespace (excluding newlines) is ignored between tokens.
Newlines are **significant** — they terminate statements and drive indentation.
Comments run from `#` to end of line. They are ignored by the parser.

### 2.2 Indentation

HMAT uses **significant indentation** (Python-style) with **4-space** standard.
Tabs are allowed but normalized to 4 spaces each (with a warning from `hfmt`).

The lexer post-processes newlines into three logical tokens:
- `NEWLINE` — end of statement
- `INDENT` — indentation level increased (emitted once per increase)
- `DEDENT` — indentation level decreased (emitted once per decrease, possibly multiple)

```
indent_rule:
  After NEWLINE, count leading spaces on next non-empty line.
  If count > current_indent: emit INDENT, push count to stack
  If count == current_indent: no indent token
  If count < current_indent: emit DEDENT for each level popped
  Blank lines and comment-only lines do not affect indentation.
```

### 2.3 Keywords

The following are reserved and cannot be used as identifiers:

```
fn       let      mut      return   if       else     elif
match    for      in       while    break    continue
struct   enum     trait    impl     type     pub      self
async    await    ai       model    load     pipeline
unsafe   import   from     as       true     false    nil
and      or       not      is       in
```

### 2.4 Identifiers

```
IDENTIFIER  ::= (ALPHA | '_') { ALPHA | DIGIT | '_' }
ALPHA       ::= 'a'..'z' | 'A'..'Z'
DIGIT       ::= '0'..'9'
```

Identifiers are case-sensitive.
Convention (not enforced by compiler, enforced by hfmt):
- Variables, functions, parameters: `snake_case`
- Types, structs, enums, traits: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

### 2.5 Literals

```
INT_LITERAL     ::= DIGIT { DIGIT | '_' }
                  | '0x' HEX_DIGIT { HEX_DIGIT | '_' }
                  | '0b' BIN_DIGIT { BIN_DIGIT | '_' }
                  | '0o' OCT_DIGIT { OCT_DIGIT | '_' }

FLOAT_LITERAL   ::= DIGIT { DIGIT } '.' { DIGIT }
                    [ ('e' | 'E') ['+' | '-'] DIGIT { DIGIT } ]

STRING_LITERAL  ::= '"' { string_char } '"'
string_char     ::= any_char_except('"', '\', '\n')
                  | ESCAPE_SEQ

F_STRING        ::= 'f"' { fstring_part } '"'
fstring_part    ::= string_char
                  | '{' expression '}'

ESCAPE_SEQ      ::= '\n' | '\t' | '\r' | '\\' | '\"' | '\0'
                  | '\x' HEX_DIGIT HEX_DIGIT

BOOL_LITERAL    ::= 'true' | 'false'
NIL_LITERAL     ::= 'nil'

HEX_DIGIT       ::= DIGIT | 'a'..'f' | 'A'..'F'
BIN_DIGIT       ::= '0' | '1'
OCT_DIGIT       ::= '0'..'7'
```

Numeric separators: `1_000_000` is valid, `_1000` is not.

### 2.6 Operators

```
Arithmetic:   +   -   *   /   %   ^
Comparison:   ==  !=  <   <=  >   >=
Logical:      and or  not
Bitwise:      &   |   ~   <<  >>
Assignment:   =   +=  -=  *=  /=  %=
Arrow:        ->  =>
Borrow:       &   &mut
Range:        ..  ..=
Propagate:    ?
Access:       .   ::
Conditional:  if (ternary)
```

### 2.7 Delimiters

```
(  )  — parentheses
[  ]  — brackets
{  }  — braces (used in f-strings only; blocks use indentation)
:     — type annotation, block start
,     — separator
```

---

## 3. Grammar Rules

### 3.1 Top Level

```
program         ::= { top_level_item }
top_level_item  ::= function_decl
                  | struct_decl
                  | enum_decl
                  | trait_decl
                  | impl_block
                  | type_alias
                  | ai_model_decl
                  | pipeline_decl
                  | import_stmt
                  | constant_decl
                  | NEWLINE
```

### 3.2 Import

```
import_stmt     ::= 'import' module_path NEWLINE
                  | 'from' module_path 'import' import_list NEWLINE

module_path     ::= IDENTIFIER { '::' IDENTIFIER }
import_list     ::= import_item { ',' import_item }
import_item     ::= IDENTIFIER [ 'as' IDENTIFIER ]
                  | '*'
```

Examples:
```hmat
import hmat::io
from hmat::collections import Vec, HashMap
from hmat::ai import Model as AiModel
```

### 3.3 Constants

```
constant_decl   ::= 'let' IDENTIFIER ':' type '=' expression NEWLINE
```
(Top-level `let` without `mut` is a constant. Must be a compile-time expression.)

### 3.4 Functions

```
function_decl   ::= [ 'pub' ] [ 'async' ] 'fn' IDENTIFIER
                    [ generic_params ]
                    '(' [ param_list ] ')'
                    [ '->' type ]
                    ':' NEWLINE
                    INDENT
                    { statement }
                    DEDENT

param_list      ::= param { ',' param }
param           ::= [ 'mut' ] IDENTIFIER ':' type
                  | 'self'
                  | '&' 'self'
                  | '&' 'mut' 'self'

generic_params  ::= '<' generic_param { ',' generic_param } '>'
generic_param   ::= IDENTIFIER [ ':' trait_bound ]
trait_bound     ::= IDENTIFIER { '+' IDENTIFIER }
```

Examples:
```hmat
fn greet(name: str) -> str:
    return f"Hello, {name}!"

pub async fn fetch(url: str) -> Result<str, HttpError>:
    let response = await http.get(url)?
    return response.text()

fn max<T: Comparable>(a: T, b: T) -> T:
    return a if a > b else b
```

### 3.5 Structs

```
struct_decl     ::= [ 'pub' ] 'struct' IDENTIFIER [ generic_params ] ':'
                    NEWLINE INDENT
                    { struct_field | function_decl | NEWLINE }
                    DEDENT

struct_field    ::= [ 'pub' ] IDENTIFIER ':' type NEWLINE
```

Examples:
```hmat
pub struct Point:
    x: float
    y: float

    fn distance(self, other: Point) -> float:
        return sqrt((self.x - other.x)^2 + (self.y - other.y)^2)

pub struct Stack<T>:
    data: Vec<T>
    size: int

    pub fn push(mut self, value: T):
        self.data.push(value)
        self.size += 1

    pub fn pop(mut self) -> Option<T>:
        return self.data.pop()
```

### 3.6 Enums

```
enum_decl       ::= [ 'pub' ] 'enum' IDENTIFIER [ generic_params ] ':'
                    NEWLINE INDENT
                    { enum_variant | NEWLINE }
                    DEDENT

enum_variant    ::= IDENTIFIER [ enum_payload ] NEWLINE
enum_payload    ::= '(' type_list ')'
                  | '{' struct_field_list '}'

type_list       ::= type { ',' type }
struct_field_list ::= struct_field { struct_field }
```

Examples:
```hmat
pub enum Direction:
    North
    South
    East
    West

pub enum Result<T, E>:
    Ok(T)
    Err(E)

pub enum Shape:
    Circle(float)
    Rectangle(float, float)
    Triangle { base: float, height: float }
```

### 3.7 Traits

```
trait_decl      ::= [ 'pub' ] 'trait' IDENTIFIER [ generic_params ] ':'
                    NEWLINE INDENT
                    { trait_method | NEWLINE }
                    DEDENT

trait_method    ::= function_signature NEWLINE
                  | function_decl      # with default implementation

function_signature ::= [ 'async' ] 'fn' IDENTIFIER [ generic_params ]
                       '(' [ param_list ] ')' [ '->' type ]
```

Examples:
```hmat
pub trait Printable:
    fn print(self)

pub trait Comparable:
    fn compare(self, other: Self) -> int
    fn equals(self, other: Self) -> bool:
        return self.compare(other) == 0

pub trait Serialize:
    fn to_json(self) -> str
    fn from_json(data: str) -> Result<Self, ParseError>
```

### 3.8 Impl Blocks

```
impl_block      ::= 'impl' [ generic_params ] [ IDENTIFIER 'for' ] IDENTIFIER ':'
                    NEWLINE INDENT
                    { function_decl | NEWLINE }
                    DEDENT
```

Examples:
```hmat
impl Point:
    fn new(x: float, y: float) -> Point:
        return Point { x: x, y: y }

impl Printable for Point:
    fn print(self):
        print(f"Point({self.x}, {self.y})")
```

### 3.9 Type Aliases

```
type_alias      ::= 'type' IDENTIFIER [ generic_params ] '=' type NEWLINE
```

Examples:
```hmat
type Callback = fn(str) -> bool
type Matrix<T> = Vec<Vec<T>>
type IoResult<T> = Result<T, IoError>
```

### 3.10 AI Model Declarations

```
ai_model_decl   ::= 'ai' 'model' IDENTIFIER '=' 'load' '(' STRING_LITERAL ')'
                    [ ai_config_block ] NEWLINE

ai_config_block ::= ':' NEWLINE INDENT { ai_config_field } DEDENT
ai_config_field ::= IDENTIFIER ':' expression NEWLINE
```

Examples:
```hmat
ai model assistant = load("anthropic/claude-3-5-sonnet")

ai model gpt = load("openai/gpt-4o"):
    temperature: 0.7
    max_tokens: 2048
    timeout: 30
```

### 3.11 Pipeline Declarations

```
pipeline_decl   ::= 'pipeline' IDENTIFIER ':'
                    NEWLINE INDENT
                    pipeline_body
                    DEDENT

pipeline_body   ::= pipeline_stage { '->' pipeline_stage } NEWLINE

pipeline_stage  ::= IDENTIFIER
                  | function_call
```

Examples:
```hmat
pipeline analyze_text:
    input -> tokenize -> embed -> classify -> output

pipeline process_image:
    load_image(path) -> resize(224, 224) -> normalize -> model.infer -> output
```

---

## 4. Statements

```
statement       ::= let_stmt
                  | assignment_stmt
                  | return_stmt
                  | if_stmt
                  | match_stmt
                  | for_stmt
                  | while_stmt
                  | break_stmt
                  | continue_stmt
                  | unsafe_block
                  | expression_stmt
                  | NEWLINE

expression_stmt ::= expression NEWLINE
```

### 4.1 Let Statement

```
let_stmt        ::= 'let' [ 'mut' ] IDENTIFIER [ ':' type ] '=' expression NEWLINE
```

Examples:
```hmat
let name = "HMAT"
let mut counter: int = 0
let point: Point = Point { x: 1.0, y: 2.0 }
```

### 4.2 Assignment

```
assignment_stmt ::= lvalue assign_op expression NEWLINE
lvalue          ::= IDENTIFIER { '.' IDENTIFIER | '[' expression ']' }
assign_op       ::= '=' | '+=' | '-=' | '*=' | '/=' | '%='
```

### 4.3 Return

```
return_stmt     ::= 'return' [ expression ] NEWLINE
```

Last expression in a block is implicitly returned (expression-oriented):
```hmat
fn add(a: int, b: int) -> int:
    a + b    # implicit return — no 'return' keyword needed
```

### 4.4 If Statement

```
if_stmt         ::= 'if' expression ':' NEWLINE INDENT { statement } DEDENT
                    { 'elif' expression ':' NEWLINE INDENT { statement } DEDENT }
                    [ 'else' ':' NEWLINE INDENT { statement } DEDENT ]
```

Ternary (inline):
```
ternary_expr    ::= expression 'if' expression 'else' expression
```

Examples:
```hmat
if x > 0:
    print("positive")
elif x < 0:
    print("negative")
else:
    print("zero")

let label = "even" if x % 2 == 0 else "odd"
```

### 4.5 Match Statement

```
match_stmt      ::= 'match' expression ':' NEWLINE INDENT
                    { match_arm }
                    DEDENT

match_arm       ::= pattern '->' ( expression | block ) NEWLINE

pattern         ::= literal_pattern
                  | identifier_pattern
                  | enum_pattern
                  | tuple_pattern
                  | wildcard_pattern
                  | guard_pattern

literal_pattern     ::= INT_LITERAL | FLOAT_LITERAL | STRING_LITERAL | BOOL_LITERAL | NIL_LITERAL
identifier_pattern  ::= IDENTIFIER
enum_pattern        ::= IDENTIFIER '::' IDENTIFIER [ '(' pattern_list ')' ]
tuple_pattern       ::= '(' pattern_list ')'
wildcard_pattern    ::= '_'
guard_pattern       ::= pattern 'if' expression
pattern_list        ::= pattern { ',' pattern }
```

Examples:
```hmat
match status:
    200 -> print("OK")
    404 -> print("Not Found")
    500 -> print("Server Error")
    _   -> print("Unknown")

match shape:
    Shape::Circle(r) -> print(f"Circle with radius {r}")
    Shape::Rectangle(w, h) -> print(f"{w} x {h}")
    Shape::Triangle { base, height } -> print(f"Triangle {base} {height}")

match value:
    n if n > 100 -> print("big")
    n if n > 0   -> print("positive")
    _            -> print("non-positive")
```

### 4.6 For Loop

```
for_stmt        ::= 'for' IDENTIFIER 'in' expression ':' NEWLINE
                    INDENT { statement } DEDENT
```

Examples:
```hmat
for item in collection:
    print(item)

for i in 0..10:
    print(i)

for i in 0..=10:    # inclusive range
    print(i)
```

### 4.7 While Loop

```
while_stmt      ::= 'while' expression ':' NEWLINE INDENT { statement } DEDENT
```

### 4.8 Break and Continue

```
break_stmt      ::= 'break' NEWLINE
continue_stmt   ::= 'continue' NEWLINE
```

### 4.9 Unsafe Block

```
unsafe_block    ::= 'unsafe' ':' NEWLINE INDENT { statement } DEDENT
```

Unsafe blocks must have a preceding comment explaining why they are safe:
```hmat
# SAFETY: ptr is guaranteed non-null at this point (checked 3 lines above)
# INVARIANT: alignment is always 8 bytes for this type
unsafe:
    let val = *ptr
```

---

## 5. Expressions

```
expression      ::= ternary_expr

ternary_expr    ::= logical_or_expr [ 'if' expression 'else' expression ]

logical_or_expr ::= logical_and_expr { 'or' logical_and_expr }
logical_and_expr::= equality_expr { 'and' equality_expr }
equality_expr   ::= comparison_expr { ('==' | '!=') comparison_expr }
comparison_expr ::= range_expr { ('<' | '<=' | '>' | '>=') range_expr }
range_expr      ::= additive_expr [ ('..' | '..=') additive_expr ]
additive_expr   ::= multiplicative_expr { ('+' | '-') multiplicative_expr }
multiplicative_expr ::= unary_expr { ('*' | '/' | '%') unary_expr }
unary_expr      ::= [ 'not' | '-' | '~' | '&' | '&mut' ] power_expr
power_expr      ::= await_expr [ '^' unary_expr ]
await_expr      ::= postfix_expr [ 'await' ]      # postfix await... OR
                  | 'await' postfix_expr           # prefix await (both valid)
postfix_expr    ::= primary_expr { postfix_op }
postfix_op      ::= '.' IDENTIFIER
                  | '.' function_call_args
                  | '[' expression ']'
                  | '?'
                  | '(' [ arg_list ] ')'

primary_expr    ::= IDENTIFIER
                  | literal
                  | f_string
                  | struct_literal
                  | tuple_expr
                  | list_expr
                  | map_expr
                  | closure_expr
                  | '(' expression ')'
                  | function_call
```

### 5.1 Literals

```
literal         ::= INT_LITERAL
                  | FLOAT_LITERAL
                  | STRING_LITERAL
                  | BOOL_LITERAL
                  | NIL_LITERAL
```

### 5.2 Struct Literal

```
struct_literal  ::= IDENTIFIER '{' [ field_init_list ] '}'
field_init_list ::= field_init { ',' field_init }
field_init      ::= IDENTIFIER ':' expression
                  | IDENTIFIER    # shorthand: field name == variable name
```

Examples:
```hmat
let p = Point { x: 1.0, y: 2.0 }

let x = 1.0
let y = 2.0
let p = Point { x, y }   # shorthand
```

### 5.3 Collections

```
list_expr       ::= '[' [ expression_list ] ']'
                  | '[' expression 'for' IDENTIFIER 'in' expression [ 'if' expression ] ']'

map_expr        ::= '{' [ map_entry_list ] '}'
map_entry_list  ::= map_entry { ',' map_entry }
map_entry       ::= expression ':' expression

tuple_expr      ::= '(' expression ',' { expression ',' } ')'

expression_list ::= expression { ',' expression }
```

Examples:
```hmat
let nums = [1, 2, 3, 4, 5]
let squares = [x^2 for x in 1..=5]
let evens = [x for x in nums if x % 2 == 0]

let scores = { "alice": 100, "bob": 95 }
let pair = (1, "hello")
```

### 5.4 Closures

```
closure_expr    ::= '|' [ param_list ] '|' '->' type ':' expression
                  | '|' [ param_list ] '|' expression
                  | '||' expression
```

Examples:
```hmat
let double = |x| x * 2
let add = |a, b| a + b
let greet = |name: str| -> str: f"Hello, {name}!"
let no_args = || 42
```

### 5.5 Function Calls

```
function_call   ::= IDENTIFIER [ '::' IDENTIFIER ] [ generic_args ] '(' [ arg_list ] ')'
arg_list        ::= arg { ',' arg }
arg             ::= [ IDENTIFIER ':' ] expression    # named or positional
generic_args    ::= '<' type_list '>'
```

Examples:
```hmat
print("hello")
Vec::new()
max::<int>(a, b)
sort(list, key: |x| x.age)
```

### 5.6 Await Expression

```hmat
# Both are valid — compiler normalizes them
let result = await some_async_fn()
let result = some_async_fn().await    # postfix style
```

### 5.7 Error Propagation Operator

```
?    # on a Result<T,E> or Option<T>: unwraps Ok/Some or early-returns Err/None
```

```hmat
fn read_file(path: str) -> Result<str, IoError>:
    let content = fs.read(path)?    # returns Err if read fails
    return Ok(content)
```

---

## 6. Types

```
type            ::= primitive_type
                  | named_type
                  | generic_type
                  | function_type
                  | reference_type
                  | optional_type
                  | result_type
                  | tuple_type
                  | array_type
                  | ai_type

primitive_type  ::= 'int' | 'i8' | 'i16' | 'i32' | 'i64' | 'i128'
                  | 'uint' | 'u8' | 'u16' | 'u32' | 'u64' | 'u128'
                  | 'float' | 'f32' | 'f64'
                  | 'bool'
                  | 'str'
                  | 'char'
                  | 'byte'
                  | '()'          # unit type (void equivalent)

named_type      ::= IDENTIFIER
generic_type    ::= IDENTIFIER '<' type_list '>'
function_type   ::= 'fn' '(' [ type_list ] ')' [ '->' type ]
reference_type  ::= '&' type | '&mut' type
optional_type   ::= 'Option' '<' type '>'
result_type     ::= 'Result' '<' type ',' type '>'
tuple_type      ::= '(' type ',' { type ',' } ')'
array_type      ::= '[' type ']'         # dynamic array (Vec equivalent)
                  | '[' type ';' INT_LITERAL ']'  # fixed-size array

ai_type         ::= 'Model'              # AI model handle
                  | 'Prompt'             # typed prompt string
                  | 'HmatCode'           # generated HMAT code (safe eval)
```

### Default Integer/Float Types

- `int` → `i64` on 64-bit platforms
- `uint` → `u64` on 64-bit platforms
- `float` → `f64`

---

## 7. Operator Precedence (High to Low)

| Level | Operators              | Associativity |
|-------|------------------------|---------------|
| 1     | `()` `[]` `.` `?`     | Left          |
| 2     | `^` (power)           | Right         |
| 3     | `not` `-` `~` `&` `&mut` | Prefix     |
| 4     | `*` `/` `%`           | Left          |
| 5     | `+` `-`               | Left          |
| 6     | `..` `..=`            | Non-assoc     |
| 7     | `<` `<=` `>` `>=`    | Non-assoc     |
| 8     | `==` `!=`             | Left          |
| 9     | `&` (bitwise)         | Left          |
| 10    | `|` (bitwise)         | Left          |
| 11    | `and`                 | Left          |
| 12    | `or`                  | Left          |
| 13    | `if` `else` (ternary) | Right         |
| 14    | `=` `+=` `-=` etc.   | Right         |

---

## 8. Error Handling

HMAT has no exceptions. Errors are values.

```hmat
# Result<T, E> — operation that can fail
fn parse_int(s: str) -> Result<int, ParseError>:
    ...

# Option<T> — value that may be absent
fn find(list: [int], target: int) -> Option<int>:
    ...

# ? operator — propagate errors
fn process(input: str) -> Result<Output, Error>:
    let parsed = parse_int(input)?      # early return if Err
    let found = find_in_db(parsed)?     # early return if Err
    return Ok(transform(found))

# Pattern matching on results
match divide(10.0, 0.0):
    Ok(result) -> print(f"Result: {result}")
    Err(e)     -> print(f"Error: {e}")

# Chaining
let result = parse_int(s)
    .map(|n| n * 2)
    .and_then(|n| validate(n))
    .unwrap_or(0)
```

---

## 9. Compiler Error Format

Every diagnostic must follow this exact format:

```
error[EXXXX]: <human readable message>
  --> <file>:<line>:<column>
   |
<line_num> | <source line>
   |         <^^^^^^^^^^^^> <what this points to>
   |
   = help: <actionable suggestion>
   = note: <optional additional context>
```

Error code ranges:
- `E001–E099`: Syntax errors (parser)
- `E100–E199`: Type errors (type checker)
- `E200–E299`: Ownership errors (borrow checker)
- `E300–E399`: AI construct errors (ai validator)
- `E400–E499`: Import/module errors
- `E500–E599`: Codegen errors (internal — user should never see these)
- `W001–W099`: Warnings (unused variables, deprecated features)

---

## 10. Grammar Summary (Quick Reference)

```
program         = { top_level_item }
top_level_item  = fn_decl | struct_decl | enum_decl | trait_decl
                | impl_block | type_alias | ai_model_decl
                | pipeline_decl | import_stmt | constant_decl

fn_decl         = ['pub'] ['async'] 'fn' IDENT [generics] '(' [params] ')' ['-> type'] ':' block
struct_decl     = ['pub'] 'struct' IDENT [generics] ':' INDENT {field | fn_decl} DEDENT
enum_decl       = ['pub'] 'enum' IDENT [generics] ':' INDENT {variant} DEDENT
trait_decl      = ['pub'] 'trait' IDENT [generics] ':' INDENT {fn_sig | fn_decl} DEDENT
impl_block      = 'impl' [generics] [IDENT 'for'] IDENT ':' INDENT {fn_decl} DEDENT

statement       = let_stmt | assign_stmt | return_stmt | if_stmt | match_stmt
                | for_stmt | while_stmt | break_stmt | continue_stmt
                | unsafe_block | expr_stmt

expression      = ternary | logical | comparison | range | arithmetic
                | unary | power | await_expr | postfix | primary

type            = primitive | named | generic | fn_type | &type | &mut type
                | Option<T> | Result<T,E> | [T] | [T;N] | (T, ...)
```
