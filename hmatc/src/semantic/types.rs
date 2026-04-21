//! HMAT Type Checker — Phase 0.
//!
//! Walks an [`crate::ast::Program`] and verifies that:
//!
//! - Every `let` binding's declared type (if any) matches its value.
//! - Every function call's arguments match the declared parameter types.
//! - Every `return` produces the function's declared return type.
//! - Every binary/unary operator is applied to types it is defined for.
//! - Every identifier is in scope (variable or function).
//!
//! # Phase 0 scope
//!
//! Primitives only (`int`, `float`, `str`, `bool`, `()`). No generics, no
//! shapes, no sum types, no ownership, no `T or Fail`. Those land in later
//! phases when the spec around them settles.
//!
//! The checker walks the tree once per function. It is permissive about
//! unknown identifiers in the sense that after reporting an error it keeps
//! going — one compile run surfaces as many problems as possible.
//!
//! See `spec/0.2/types.md` for the language-level semantics.

use std::collections::HashMap;
use thiserror::Error;

use crate::ast::{
    BinOp, Binary, Block, Call, Expression, FunctionDecl, Item, LetStmt, Literal, Program,
    ReturnStmt, Statement, TypeRef, UnOp, Unary,
};
use crate::lexer::Span;

// ======================================================================
// Public types
// ======================================================================

/// A resolved HMAT type as understood by the Phase 0 checker.
///
/// Unknown is the error-propagation sentinel: after emitting a diagnostic,
/// the checker substitutes `Unknown` and suppresses further errors that
/// would cascade from the original cause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// Default signed integer (`int`, also matches the sized aliases).
    Int,
    /// IEEE-754 64-bit float (`float`, `f32`, `f64`).
    Float,
    /// UTF-8 string (`str`).
    Str,
    /// Boolean (`bool`).
    Bool,
    /// Unit — no value (`()`).
    Unit,
    /// `nil` — only produced by the bare `nil` literal. Must be resolved by
    /// a surrounding annotation before it escapes a let binding.
    Nil,
    /// Placeholder inserted after a diagnostic fires, so cascading errors
    /// can be suppressed.
    Unknown,
}

impl Type {
    /// Human-readable rendering used in diagnostics.
    pub fn display(&self) -> String {
        match self {
            Type::Int => "int".into(),
            Type::Float => "float".into(),
            Type::Str => "str".into(),
            Type::Bool => "bool".into(),
            Type::Unit => "()".into(),
            Type::Nil => "nil".into(),
            Type::Unknown => "<unknown>".into(),
        }
    }

    fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }
}

/// The type-checking entry point.
///
/// Returns `Ok(())` when the program is type-correct, or a list of every
/// [`TypeError`] encountered on a single walk of the tree.
///
/// # Example
/// ```
/// let src = "fn main():\n    let x: int = 1\n";
/// let tokens = hmatc::lexer::tokenize(src).unwrap();
/// let program = hmatc::parser::parse(tokens).unwrap();
/// assert!(hmatc::semantic::check(&program).is_ok());
/// ```
pub fn check(program: &Program) -> Result<(), Vec<TypeError>> {
    let mut checker = Checker::new();
    checker.check_program(program);
    if checker.errors.is_empty() {
        Ok(())
    } else {
        Err(checker.errors)
    }
}

// ======================================================================
// Checker state
// ======================================================================

/// Signature used by the checker to validate call sites.
#[derive(Debug, Clone)]
struct FnSig {
    params: Vec<Type>,
    ret: Type,
}

struct Checker {
    errors: Vec<TypeError>,
    /// All user-declared functions plus a handful of Phase 0 builtins.
    functions: HashMap<String, FnSig>,
    /// Stack of lexical scopes. Phase 0 has exactly one active scope per
    /// function — the stack is ready for nested blocks (if / for / match)
    /// in later phases.
    scopes: Vec<HashMap<String, Type>>,
    /// Return type of the function currently being checked.
    current_return: Option<Type>,
    /// Name of the function currently being checked — used in diagnostics.
    current_fn: Option<String>,
}

impl Checker {
    fn new() -> Self {
        let mut functions = HashMap::new();
        // Phase 0 builtins. `print` is here so `hello_world.hm` type-checks
        // without needing a stdlib loader. Signatures will move to a real
        // prelude module once Phase 2 ships import resolution.
        functions.insert(
            "print".to_string(),
            FnSig {
                params: vec![Type::Str],
                ret: Type::Unit,
            },
        );
        Self {
            errors: Vec::new(),
            functions,
            scopes: Vec::new(),
            current_return: None,
            current_fn: None,
        }
    }

    // ------------------------------------------------------------------
    // Scope helpers
    // ------------------------------------------------------------------

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare_var(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    fn lookup_var(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(t.clone());
            }
        }
        None
    }

    // ------------------------------------------------------------------
    // Top-level walk
    // ------------------------------------------------------------------

    fn check_program(&mut self, program: &Program) {
        // Pass 1 — register every function signature so forward calls work.
        for item in &program.items {
            match item {
                Item::Function(f) => self.register_function(f),
            }
        }

        // Pass 2 — check each body against the collected signatures.
        for item in &program.items {
            match item {
                Item::Function(f) => self.check_function(f),
            }
        }
    }

    fn register_function(&mut self, f: &FunctionDecl) {
        let params = f
            .params
            .iter()
            .map(|p| self.resolve_type_ref(&p.ty))
            .collect();
        let ret = match &f.return_type {
            Some(tr) => self.resolve_type_ref(tr),
            None => Type::Unit,
        };
        // Duplicate fn names overwrite silently in Phase 0; a dedicated
        // E11x "duplicate definition" diagnostic is Phase 2 work.
        self.functions
            .insert(f.name.name.clone(), FnSig { params, ret });
    }

    fn check_function(&mut self, f: &FunctionDecl) {
        let sig = match self.functions.get(&f.name.name).cloned() {
            Some(s) => s,
            None => return,
        };

        self.push_scope();
        for (param, ty) in f.params.iter().zip(sig.params.iter()) {
            self.declare_var(param.name.name.clone(), ty.clone());
        }

        let prev_return = self.current_return.take();
        let prev_fn = self.current_fn.take();
        self.current_return = Some(sig.ret.clone());
        self.current_fn = Some(f.name.name.clone());

        self.check_block(&f.body);

        self.current_return = prev_return;
        self.current_fn = prev_fn;
        self.pop_scope();
    }

    fn check_block(&mut self, block: &Block) {
        for stmt in &block.statements {
            self.check_statement(stmt);
        }
    }

    // ------------------------------------------------------------------
    // Statements
    // ------------------------------------------------------------------

    fn check_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let(l) => self.check_let(l),
            Statement::Return(r) => self.check_return(r),
            Statement::Expr(e) => {
                let _ = self.check_expression(e);
            }
        }
    }

    fn check_let(&mut self, l: &LetStmt) {
        let value_ty = self.check_expression(&l.value);

        let binding_ty = match &l.ty {
            Some(tr) => {
                let declared = self.resolve_type_ref(tr);
                if !self.types_compatible(&declared, &value_ty) {
                    self.errors.push(TypeError::Mismatch {
                        span: l.value.span().clone(),
                        expected: declared.display(),
                        actual: value_ty.display(),
                    });
                }
                declared
            }
            None => {
                if value_ty == Type::Nil {
                    self.errors.push(TypeError::CannotInferType {
                        span: l.span.clone(),
                        name: l.name.name.clone(),
                    });
                    Type::Unknown
                } else {
                    value_ty
                }
            }
        };

        self.declare_var(l.name.name.clone(), binding_ty);
    }

    fn check_return(&mut self, r: &ReturnStmt) {
        let expected = self.current_return.clone().unwrap_or(Type::Unit);
        let actual = match &r.value {
            None => Type::Unit,
            Some(e) => self.check_expression(e),
        };
        if !self.types_compatible(&expected, &actual) {
            self.errors.push(TypeError::ReturnMismatch {
                span: r.span.clone(),
                function: self.current_fn.clone().unwrap_or_default(),
                expected: expected.display(),
                actual: actual.display(),
            });
        }
    }

    // ------------------------------------------------------------------
    // Expressions
    // ------------------------------------------------------------------

    fn check_expression(&mut self, e: &Expression) -> Type {
        match e {
            Expression::Literal(l) => self.check_literal(l),
            Expression::Identifier(id) => {
                if let Some(t) = self.lookup_var(&id.name) {
                    t
                } else {
                    self.errors.push(TypeError::UndefinedVariable {
                        span: id.span.clone(),
                        name: id.name.clone(),
                    });
                    Type::Unknown
                }
            }
            Expression::Call(c) => self.check_call(c),
            Expression::FieldAccess(fa) => {
                // Phase 0 has no shapes / structs yet — walk the base for
                // side-effect errors and return Unknown so downstream
                // checks do not cascade.
                let _ = self.check_expression(&fa.base);
                Type::Unknown
            }
            Expression::Binary(b) => self.check_binary(b),
            Expression::Unary(u) => self.check_unary(u),
            Expression::Grouped(g) => self.check_expression(&g.inner),
        }
    }

    fn check_literal(&mut self, l: &Literal) -> Type {
        match l {
            Literal::Int { .. } => Type::Int,
            Literal::Float { .. } => Type::Float,
            Literal::Str { .. } => Type::Str,
            // f-strings evaluate to str in Phase 0; real interpolation
            // type-checking lands when Phase 2 wires f-string splitting.
            Literal::FStr { .. } => Type::Str,
            Literal::Bool { .. } => Type::Bool,
            Literal::Nil { .. } => Type::Nil,
        }
    }

    fn check_call(&mut self, c: &Call) -> Type {
        // Phase 0 only recognises direct calls — `name(args)`. Calling an
        // arbitrary expression (higher-order functions) is Phase 2.
        let callee_name = match &*c.callee {
            Expression::Identifier(id) => id.name.clone(),
            other => {
                let ty = self.check_expression(other);
                for arg in &c.args {
                    let _ = self.check_expression(arg);
                }
                self.errors.push(TypeError::NotCallable {
                    span: other.span().clone(),
                    ty: ty.display(),
                });
                return Type::Unknown;
            }
        };

        let sig = match self.functions.get(&callee_name).cloned() {
            Some(s) => s,
            None => {
                self.errors.push(TypeError::UndefinedFunction {
                    span: c.callee.span().clone(),
                    name: callee_name,
                });
                for arg in &c.args {
                    let _ = self.check_expression(arg);
                }
                return Type::Unknown;
            }
        };

        if sig.params.len() != c.args.len() {
            self.errors.push(TypeError::WrongArgCount {
                span: c.span.clone(),
                name: callee_name.clone(),
                expected: sig.params.len(),
                actual: c.args.len(),
            });
        }

        for (i, arg) in c.args.iter().enumerate() {
            let actual = self.check_expression(arg);
            if let Some(expected) = sig.params.get(i) {
                if !self.types_compatible(expected, &actual) {
                    self.errors.push(TypeError::ArgTypeMismatch {
                        span: arg.span().clone(),
                        name: callee_name.clone(),
                        index: i + 1,
                        expected: expected.display(),
                        actual: actual.display(),
                    });
                }
            }
        }

        sig.ret
    }

    fn check_binary(&mut self, b: &Binary) -> Type {
        let lhs = self.check_expression(&b.lhs);
        let rhs = self.check_expression(&b.rhs);

        // Suppress cascading op errors when a child was already wrong.
        if lhs == Type::Unknown || rhs == Type::Unknown {
            return Type::Unknown;
        }

        match b.op {
            // Arithmetic — same numeric type on both sides. `+` also works
            // on `str` for concatenation, matching Python-ish ergonomics.
            BinOp::Add => {
                if lhs == rhs && (lhs.is_numeric() || lhs == Type::Str) {
                    return lhs;
                }
            }
            BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::Pow => {
                if lhs == rhs && lhs.is_numeric() {
                    return lhs;
                }
            }
            // Equality — any matching type.
            BinOp::Eq | BinOp::NotEq => {
                if lhs == rhs {
                    return Type::Bool;
                }
            }
            // Ordering — numeric or str, both sides same type.
            BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq => {
                if lhs == rhs && (lhs.is_numeric() || lhs == Type::Str) {
                    return Type::Bool;
                }
            }
            // Logical — strict bool on both sides. The `or` fallback form
            // (`divide(x, y) or 0.0`) arrives with `T or Fail` support.
            BinOp::And | BinOp::Or => {
                if lhs == Type::Bool && rhs == Type::Bool {
                    return Type::Bool;
                }
            }
        }

        self.errors.push(TypeError::UndefinedBinaryOp {
            span: b.span.clone(),
            op: b.op.as_str().to_string(),
            lhs: lhs.display(),
            rhs: rhs.display(),
        });
        Type::Unknown
    }

    fn check_unary(&mut self, u: &Unary) -> Type {
        let operand = self.check_expression(&u.operand);
        if operand == Type::Unknown {
            return Type::Unknown;
        }
        match u.op {
            UnOp::Neg if operand.is_numeric() => return operand,
            UnOp::Not if operand == Type::Bool => return Type::Bool,
            _ => {}
        }
        self.errors.push(TypeError::UndefinedUnaryOp {
            span: u.span.clone(),
            op: u.op.as_str().to_string(),
            ty: operand.display(),
        });
        Type::Unknown
    }

    // ------------------------------------------------------------------
    // Type-ref resolution
    // ------------------------------------------------------------------

    fn resolve_type_ref(&mut self, tr: &TypeRef) -> Type {
        // Phase 0 recognises the five primitive surface names and the
        // standard sized aliases; anything else is flagged as unknown so
        // the user gets a clear "check spelling" hint.
        if !tr.args.is_empty() {
            // Generics arrive in Phase 2 — accept the base name so tests
            // exercising the parser's generic path do not double-fire.
            self.errors.push(TypeError::UnknownType {
                span: tr.span.clone(),
                name: format!("{}<...>", tr.name),
            });
            return Type::Unknown;
        }
        match tr.name.as_str() {
            "int" | "i8" | "i16" | "i32" | "i64" | "i128" | "uint" | "u8" | "u16" | "u32"
            | "u64" | "u128" => Type::Int,
            "float" | "f32" | "f64" => Type::Float,
            "bool" => Type::Bool,
            "str" => Type::Str,
            _ => {
                self.errors.push(TypeError::UnknownType {
                    span: tr.span.clone(),
                    name: tr.name.clone(),
                });
                Type::Unknown
            }
        }
    }

    // ------------------------------------------------------------------
    // Compatibility
    // ------------------------------------------------------------------

    /// Two types are compatible when they are equal, or when one is
    /// `Unknown` (a previous error — suppress cascade).
    fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        expected == actual || *expected == Type::Unknown || *actual == Type::Unknown
    }
}

// ======================================================================
// Errors
// ======================================================================

/// Errors produced by the type checker.
///
/// Codes `E100`–`E199` are reserved for type-level diagnostics per
/// `spec/0.2/grammar.md` §9. Every variant carries a [`Span`] and a
/// [`TypeError::help`] string — Zen #2: errors are teachers, not walls.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum TypeError {
    /// A value's type does not match the surrounding annotation.
    /// Emitted with error code `E100`.
    #[error("E100: type mismatch — expected `{expected}`, found `{actual}`")]
    Mismatch {
        span: Span,
        expected: String,
        actual: String,
    },

    /// A binding's type could not be inferred and no annotation was given.
    /// Emitted with error code `E101`.
    #[error("E101: cannot infer type for `{name}`")]
    CannotInferType { span: Span, name: String },

    /// A binary operator was applied to operand types it is not defined for.
    /// Emitted with error code `E102`.
    #[error("E102: `{op}` is not defined for `{lhs}` and `{rhs}`")]
    UndefinedBinaryOp {
        span: Span,
        op: String,
        lhs: String,
        rhs: String,
    },

    /// A unary operator was applied to an operand type it is not defined for.
    /// Emitted with error code `E103`.
    #[error("E103: `{op}` is not defined for `{ty}`")]
    UndefinedUnaryOp { span: Span, op: String, ty: String },

    /// Use of an identifier that has no binding in the current scope.
    /// Emitted with error code `E104`.
    #[error("E104: undefined variable `{name}`")]
    UndefinedVariable { span: Span, name: String },

    /// Call to a name that is not declared as a function.
    /// Emitted with error code `E105`.
    #[error("E105: undefined function `{name}`")]
    UndefinedFunction { span: Span, name: String },

    /// Call with the wrong number of arguments.
    /// Emitted with error code `E106`.
    #[error("E106: function `{name}` expects {expected} argument(s), got {actual}")]
    WrongArgCount {
        span: Span,
        name: String,
        expected: usize,
        actual: usize,
    },

    /// An argument's type does not match the parameter's type.
    /// Emitted with error code `E107`.
    #[error("E107: argument {index} of `{name}`: expected `{expected}`, found `{actual}`")]
    ArgTypeMismatch {
        span: Span,
        name: String,
        index: usize,
        expected: String,
        actual: String,
    },

    /// Attempted to call a non-function value.
    /// Emitted with error code `E108`.
    #[error("E108: `{ty}` is not callable")]
    NotCallable { span: Span, ty: String },

    /// A `return` expression's type does not match the function's declared
    /// return type.
    /// Emitted with error code `E109`.
    #[error(
        "E109: return type mismatch in `{function}` — expected `{expected}`, found `{actual}`"
    )]
    ReturnMismatch {
        span: Span,
        function: String,
        expected: String,
        actual: String,
    },

    /// A type annotation referenced a name the checker does not recognise.
    /// Emitted with error code `E110`.
    #[error("E110: unknown type `{name}`")]
    UnknownType { span: Span, name: String },
}

impl TypeError {
    /// Returns the span this error points to, for diagnostic rendering.
    pub fn span(&self) -> &Span {
        match self {
            TypeError::Mismatch { span, .. } => span,
            TypeError::CannotInferType { span, .. } => span,
            TypeError::UndefinedBinaryOp { span, .. } => span,
            TypeError::UndefinedUnaryOp { span, .. } => span,
            TypeError::UndefinedVariable { span, .. } => span,
            TypeError::UndefinedFunction { span, .. } => span,
            TypeError::WrongArgCount { span, .. } => span,
            TypeError::ArgTypeMismatch { span, .. } => span,
            TypeError::NotCallable { span, .. } => span,
            TypeError::ReturnMismatch { span, .. } => span,
            TypeError::UnknownType { span, .. } => span,
        }
    }

    /// Returns the four-character diagnostic code (e.g. `E100`).
    pub fn code(&self) -> &'static str {
        match self {
            TypeError::Mismatch { .. } => "E100",
            TypeError::CannotInferType { .. } => "E101",
            TypeError::UndefinedBinaryOp { .. } => "E102",
            TypeError::UndefinedUnaryOp { .. } => "E103",
            TypeError::UndefinedVariable { .. } => "E104",
            TypeError::UndefinedFunction { .. } => "E105",
            TypeError::WrongArgCount { .. } => "E106",
            TypeError::ArgTypeMismatch { .. } => "E107",
            TypeError::NotCallable { .. } => "E108",
            TypeError::ReturnMismatch { .. } => "E109",
            TypeError::UnknownType { .. } => "E110",
        }
    }

    /// Returns an actionable fix hint. Every type error has one — Zen #2.
    pub fn help(&self) -> &'static str {
        match self {
            TypeError::Mismatch { .. } => {
                "remove the annotation to let the compiler infer it, \
                 or change the value to match the declared type"
            }
            TypeError::CannotInferType { .. } => {
                "`nil` alone has no type — add an annotation like \
                 `x: int or nil = nil` once nilable types ship"
            }
            TypeError::UndefinedBinaryOp { .. } => {
                "both sides of an arithmetic operator must be the same \
                 numeric type — convert one side explicitly, e.g. \
                 `\"42\".parse_int() or 0`"
            }
            TypeError::UndefinedUnaryOp { .. } => {
                "`-` works on numeric types (int, float); `not` works on bool"
            }
            TypeError::UndefinedVariable { .. } => {
                "declare it with `let` first, or check for a typo \
                 — names are case-sensitive"
            }
            TypeError::UndefinedFunction { .. } => {
                "check the spelling, or import the module that defines it"
            }
            TypeError::WrongArgCount { .. } => {
                "check the function's signature — add the missing arguments \
                 or remove the extra ones"
            }
            TypeError::ArgTypeMismatch { .. } => {
                "convert the value to the expected type, or check the \
                 function's signature for the right order of parameters"
            }
            TypeError::NotCallable { .. } => {
                "only functions can be called with `()` — Phase 0 does not \
                 support higher-order calls yet"
            }
            TypeError::ReturnMismatch { .. } => {
                "change the returned value, or update the function's \
                 declared return type to match"
            }
            TypeError::UnknownType { .. } => {
                "Phase 0 primitives are `int`, `float`, `bool`, `str`, and \
                 `()` — check spelling or wait for Phase 2 user types"
            }
        }
    }
}
