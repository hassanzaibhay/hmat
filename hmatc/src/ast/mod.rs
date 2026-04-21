//! HMAT Abstract Syntax Tree.
//!
//! AST node definitions emitted by [`crate::parser`] and consumed by later
//! compiler passes. Phase 0 scope covers the minimal surface required to
//! compile `hello_world.hm`: function declarations, parameters, blocks,
//! let/return/expression statements, and a core expression language
//! (literals, identifiers, binary/unary ops, calls, field access,
//! grouping).
//!
//! Nodes carry byte [`Span`]s into the original source so diagnostics can
//! point back at the offending code.
//!
//! See `spec/0.2/grammar.md` §3–§5.

use crate::lexer::Span;
use std::fmt::Write as _;

// ======================================================================
// Top-level
// ======================================================================

/// A whole parsed HMAT source file.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: Span,
}

/// A top-level item in a program. Phase 0 only parses functions; more
/// variants (struct, enum, trait, impl, import, ai_model, pipeline) will be
/// added in later phases per spec §3.1.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Function(FunctionDecl),
}

/// A `fn` declaration — `fn NAME(params) [-> type]: block`.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: Identifier,
    pub params: Vec<Param>,
    pub return_type: Option<TypeRef>,
    pub body: Block,
    pub span: Span,
}

/// A single formal parameter: `[mut] NAME : TYPE`.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: Identifier,
    pub mutable: bool,
    pub ty: TypeRef,
    pub span: Span,
}

/// A reference to a type in source. Phase 0 supports:
///   - named:    `int`, `Point`
///   - generic:  `List[int]`, `Result[int, str]`  (spec v0.3 §2.7 / §3.1 — `[T]`, not `<T>`)
///   - ref:      `&T`, `&mut T`
///   - fallible: `T or Fail`
///   - optional: `T or nil`
///   - combined: `T or Fail or nil`
///
/// Richer type forms (tuples, arrays, fn types) land when Phase 2 needs them.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeRef {
    pub name: String,
    pub args: Vec<TypeRef>,
    pub is_ref: bool,
    pub is_mut_ref: bool,
    /// `T or Fail` — a fallible type per spec v0.3 §4. Only legal on
    /// return-type position today; the parser enforces that.
    pub is_fallible: bool,
    /// `T or nil` — an optional type per spec v0.3 §3.1.
    pub is_nilable: bool,
    pub span: Span,
}

// ======================================================================
// Statements
// ======================================================================

/// A lexical block: `INDENT stmt* DEDENT`.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

/// A statement inside a block. Phase 0 supports let/return/expression
/// statements only — loops, if, match, break, continue arrive in Phase 2.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let(LetStmt),
    Return(ReturnStmt),
    Expr(Expression),
}

/// `let [mut] NAME [: TYPE] = EXPR`.
#[derive(Debug, Clone, PartialEq)]
pub struct LetStmt {
    pub name: Identifier,
    pub mutable: bool,
    pub ty: Option<TypeRef>,
    pub value: Expression,
    pub span: Span,
}

/// `return [EXPR]`.
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStmt {
    pub value: Option<Expression>,
    pub span: Span,
}

// ======================================================================
// Expressions
// ======================================================================

/// An identifier use or binding.
#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
    pub name: String,
    pub span: Span,
}

/// An expression node. Operator precedence is baked into the parser, not
/// the AST — by the time you see this tree, associativity and grouping
/// are settled.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Identifier(Identifier),
    Call(Call),
    FieldAccess(FieldAccess),
    Binary(Binary),
    Unary(Unary),
    Grouped(Grouped),
}

impl Expression {
    /// The source span covered by this expression.
    pub fn span(&self) -> &Span {
        match self {
            Expression::Literal(l) => l.span(),
            Expression::Identifier(i) => &i.span,
            Expression::Call(c) => &c.span,
            Expression::FieldAccess(f) => &f.span,
            Expression::Binary(b) => &b.span,
            Expression::Unary(u) => &u.span,
            Expression::Grouped(g) => &g.span,
        }
    }
}

/// `callee(args...)`.
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub callee: Box<Expression>,
    pub args: Vec<Expression>,
    pub span: Span,
}

/// `base.field`.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldAccess {
    pub base: Box<Expression>,
    pub field: Identifier,
    pub span: Span,
}

/// `lhs OP rhs` for a binary operator.
#[derive(Debug, Clone, PartialEq)]
pub struct Binary {
    pub op: BinOp,
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
    pub span: Span,
}

/// `OP operand` for a unary prefix operator.
#[derive(Debug, Clone, PartialEq)]
pub struct Unary {
    pub op: UnOp,
    pub operand: Box<Expression>,
    pub span: Span,
}

/// `( inner )` — preserved so `--emit=ast` round-trips parenthesisation.
#[derive(Debug, Clone, PartialEq)]
pub struct Grouped {
    pub inner: Box<Expression>,
    pub span: Span,
}

/// Literal values that appear directly in source.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int {
        value: i64,
        span: Span,
    },
    Float {
        value: f64,
        span: Span,
    },
    Str {
        value: String,
        span: Span,
    },
    /// Body of an `f"..."` literal, with the `{expr}` interpolations left
    /// unparsed until Phase 2 wires real f-string splitting.
    FStr {
        value: String,
        span: Span,
    },
    Bool {
        value: bool,
        span: Span,
    },
    Nil {
        span: Span,
    },
}

impl Literal {
    /// The source span of this literal.
    pub fn span(&self) -> &Span {
        match self {
            Literal::Int { span, .. } => span,
            Literal::Float { span, .. } => span,
            Literal::Str { span, .. } => span,
            Literal::FStr { span, .. } => span,
            Literal::Bool { span, .. } => span,
            Literal::Nil { span } => span,
        }
    }
}

/// Binary operators recognised by the Phase 0 parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
}

impl BinOp {
    /// Source-form rendering (e.g. `+`, `and`). Used by the pretty-printer
    /// and error messages.
    pub fn as_str(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Pow => "^",
            BinOp::Eq => "==",
            BinOp::NotEq => "!=",
            BinOp::Lt => "<",
            BinOp::LtEq => "<=",
            BinOp::Gt => ">",
            BinOp::GtEq => ">=",
            BinOp::And => "and",
            BinOp::Or => "or",
        }
    }
}

/// Unary prefix operators recognised by the Phase 0 parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

impl UnOp {
    /// Source-form rendering (`-`, `not`).
    pub fn as_str(self) -> &'static str {
        match self {
            UnOp::Neg => "-",
            UnOp::Not => "not",
        }
    }
}

// ======================================================================
// Pretty printer — `--emit=ast` output
// ======================================================================

/// Two-space indent per tree level — consistent with Python-ish AST dumps
/// and easy to diff.
const INDENT: &str = "  ";

fn push_indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str(INDENT);
    }
}

impl Program {
    /// Renders the AST as an indented, human-readable tree.
    ///
    /// Each node starts at a fixed column; a child is one level deeper.
    /// Spans and raw source offsets are intentionally omitted — the tree
    /// is for reading, not round-tripping.
    pub fn pretty_print(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "Program");
        for item in &self.items {
            item.pretty(&mut out, 1);
        }
        out
    }
}

impl Item {
    fn pretty(&self, out: &mut String, depth: usize) {
        match self {
            Item::Function(f) => f.pretty(out, depth),
        }
    }
}

impl FunctionDecl {
    fn pretty(&self, out: &mut String, depth: usize) {
        push_indent(out, depth);
        let _ = writeln!(out, "FunctionDecl `{}`", self.name.name);

        push_indent(out, depth + 1);
        if self.params.is_empty() {
            let _ = writeln!(out, "params: (none)");
        } else {
            let _ = writeln!(out, "params:");
            for p in &self.params {
                p.pretty(out, depth + 2);
            }
        }

        push_indent(out, depth + 1);
        match &self.return_type {
            None => {
                let _ = writeln!(out, "return: (none)");
            }
            Some(ty) => {
                let _ = writeln!(out, "return:");
                ty.pretty(out, depth + 2);
            }
        }

        push_indent(out, depth + 1);
        let _ = writeln!(out, "body:");
        self.body.pretty(out, depth + 2);
    }
}

impl Param {
    fn pretty(&self, out: &mut String, depth: usize) {
        push_indent(out, depth);
        let mut_tag = if self.mutable { "mut " } else { "" };
        let _ = writeln!(out, "Param {}`{}`", mut_tag, self.name.name);
        self.ty.pretty(out, depth + 1);
    }
}

impl TypeRef {
    fn pretty(&self, out: &mut String, depth: usize) {
        push_indent(out, depth);
        let prefix = match (self.is_ref, self.is_mut_ref) {
            (true, true) => "&mut ",
            (true, false) => "&",
            _ => "",
        };
        let mut suffix = String::new();
        if self.is_fallible {
            suffix.push_str(" or Fail");
        }
        if self.is_nilable {
            suffix.push_str(" or nil");
        }
        if self.args.is_empty() {
            let _ = writeln!(out, "Type {}{}{}", prefix, self.name, suffix);
        } else {
            let _ = writeln!(out, "Type {}{}[]{}", prefix, self.name, suffix);
            for arg in &self.args {
                arg.pretty(out, depth + 1);
            }
        }
    }
}

impl Block {
    fn pretty(&self, out: &mut String, depth: usize) {
        push_indent(out, depth);
        if self.statements.is_empty() {
            let _ = writeln!(out, "Block (empty)");
            return;
        }
        let _ = writeln!(out, "Block");
        for s in &self.statements {
            s.pretty(out, depth + 1);
        }
    }
}

impl Statement {
    fn pretty(&self, out: &mut String, depth: usize) {
        match self {
            Statement::Let(s) => s.pretty(out, depth),
            Statement::Return(s) => s.pretty(out, depth),
            Statement::Expr(e) => {
                push_indent(out, depth);
                let _ = writeln!(out, "ExprStmt");
                e.pretty(out, depth + 1);
            }
        }
    }
}

impl LetStmt {
    fn pretty(&self, out: &mut String, depth: usize) {
        push_indent(out, depth);
        let mut_tag = if self.mutable { "mut " } else { "" };
        let _ = writeln!(out, "Let {}`{}`", mut_tag, self.name.name);
        if let Some(ty) = &self.ty {
            push_indent(out, depth + 1);
            let _ = writeln!(out, "ty:");
            ty.pretty(out, depth + 2);
        }
        push_indent(out, depth + 1);
        let _ = writeln!(out, "value:");
        self.value.pretty(out, depth + 2);
    }
}

impl ReturnStmt {
    fn pretty(&self, out: &mut String, depth: usize) {
        push_indent(out, depth);
        match &self.value {
            None => {
                let _ = writeln!(out, "Return (no value)");
            }
            Some(v) => {
                let _ = writeln!(out, "Return");
                v.pretty(out, depth + 1);
            }
        }
    }
}

impl Expression {
    fn pretty(&self, out: &mut String, depth: usize) {
        match self {
            Expression::Literal(lit) => lit.pretty(out, depth),
            Expression::Identifier(id) => {
                push_indent(out, depth);
                let _ = writeln!(out, "Ident `{}`", id.name);
            }
            Expression::Call(c) => {
                push_indent(out, depth);
                let _ = writeln!(out, "Call");
                push_indent(out, depth + 1);
                let _ = writeln!(out, "callee:");
                c.callee.pretty(out, depth + 2);
                push_indent(out, depth + 1);
                if c.args.is_empty() {
                    let _ = writeln!(out, "args: (none)");
                } else {
                    let _ = writeln!(out, "args:");
                    for a in &c.args {
                        a.pretty(out, depth + 2);
                    }
                }
            }
            Expression::FieldAccess(fa) => {
                push_indent(out, depth);
                let _ = writeln!(out, "FieldAccess `.{}`", fa.field.name);
                fa.base.pretty(out, depth + 1);
            }
            Expression::Binary(b) => {
                push_indent(out, depth);
                let _ = writeln!(out, "Binary `{}`", b.op.as_str());
                b.lhs.pretty(out, depth + 1);
                b.rhs.pretty(out, depth + 1);
            }
            Expression::Unary(u) => {
                push_indent(out, depth);
                let _ = writeln!(out, "Unary `{}`", u.op.as_str());
                u.operand.pretty(out, depth + 1);
            }
            Expression::Grouped(g) => {
                push_indent(out, depth);
                let _ = writeln!(out, "Grouped");
                g.inner.pretty(out, depth + 1);
            }
        }
    }
}

impl Literal {
    fn pretty(&self, out: &mut String, depth: usize) {
        push_indent(out, depth);
        match self {
            Literal::Int { value, .. } => {
                let _ = writeln!(out, "IntLit {value}");
            }
            Literal::Float { value, .. } => {
                let _ = writeln!(out, "FloatLit {value}");
            }
            Literal::Str { value, .. } => {
                let _ = writeln!(out, "StrLit {value:?}");
            }
            Literal::FStr { value, .. } => {
                let _ = writeln!(out, "FStrLit {value:?}");
            }
            Literal::Bool { value, .. } => {
                let _ = writeln!(out, "BoolLit {value}");
            }
            Literal::Nil { .. } => {
                let _ = writeln!(out, "NilLit");
            }
        }
    }
}
