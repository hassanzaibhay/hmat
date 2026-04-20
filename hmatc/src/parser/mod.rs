//! HMAT Parser.
//!
//! Hand-rolled recursive descent parser consuming the spanned token stream
//! produced by [`crate::lexer`]. Produces an [`ast::Program`] or a
//! [`ParseError`] with an error code in the `E010`–`E099` range reserved
//! for syntax-level diagnostics (codes `E001`–`E003` belong to the lexer;
//! see `spec/0.2/grammar.md` §9).
//!
//! # Phase 0 scope
//!
//! The surface this parser recognises is deliberately narrow — just what
//! is needed to get the `hello_world.hm` milestone across the finish line:
//!
//! - `fn` declarations at top level (params, optional return type, body)
//! - `let` / `return` / expression statements inside blocks
//! - Expressions: literals, identifiers, calls, field access, unary,
//!   binary arithmetic / comparison / logical operators, grouping
//! - Types: named, generic, `&T`, `&mut T`
//!
//! Structs, enums, traits, impl blocks, imports, ai-model declarations,
//! pipelines, match, for, while, closures, pattern matching, and `?` error
//! propagation all land in later phases when the spec pieces around them
//! (type checker, ownership checker) catch up.
//!
//! # Design notes
//!
//! - Precedence is baked into the recursive descent itself: one function
//!   per precedence level, climbing from `parse_or` (lowest) down to
//!   `parse_primary` (highest).
//! - Newlines inside `(...)` are skipped so multi-line arg/param lists
//!   parse cleanly; everywhere else `Newline` is a statement terminator.
//! - Every error carries a byte [`Span`] and a help string so the
//!   diagnostic renderer (future work) can build the spec §9 caret format.

use crate::ast::*;
use crate::lexer::{Span, Token};
use thiserror::Error;

/// A `(Token, Span)` pair as produced by the lexer.
pub type SpannedToken = (Token, Span);

/// Parses the given spanned token stream into an AST.
///
/// # Errors
/// Returns the first [`ParseError`] encountered. Phase 0 has no error
/// recovery — the parser bails on the first invalid construct.
///
/// # Example
/// ```
/// let tokens = hmatc::lexer::tokenize("fn main():\n    let x = 1\n").unwrap();
/// let program = hmatc::parser::parse(tokens).unwrap();
/// assert_eq!(program.items.len(), 1);
/// ```
pub fn parse(tokens: Vec<SpannedToken>) -> Result<Program, ParseError> {
    let mut p = Parser::new(tokens);
    p.parse_program()
}

// ======================================================================
// Parser state
// ======================================================================

struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
    /// Zero-width span at EOF, reused for unexpected-end-of-input errors.
    eof_span: Span,
}

impl Parser {
    fn new(tokens: Vec<SpannedToken>) -> Self {
        let eof_span = tokens.last().map(|(_, s)| s.end..s.end).unwrap_or(0..0);
        Self {
            tokens,
            pos: 0,
            eof_span,
        }
    }

    // ------------------------------------------------------------------
    // Stream helpers
    // ------------------------------------------------------------------

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|(t, _)| t)
    }

    fn peek_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|(_, s)| s.clone())
            .unwrap_or_else(|| self.eof_span.clone())
    }

    fn advance(&mut self) -> Option<SpannedToken> {
        if self.pos < self.tokens.len() {
            let item = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(item)
        } else {
            None
        }
    }

    /// Tokens compare by discriminant only — the payload of `Identifier`,
    /// `IntLiteral`, etc. is not meaningful when matching "is the next
    /// token an identifier?".
    fn check(&self, expected: &Token) -> bool {
        self.peek()
            .map(|t| std::mem::discriminant(t) == std::mem::discriminant(expected))
            .unwrap_or(false)
    }

    /// Consumes exactly the expected token variant or produces a diagnostic.
    /// `description` is surfaced to the user — prefer something actionable
    /// ("`:` before function body") over the token label itself.
    fn expect(&mut self, expected: &Token, description: &str) -> Result<SpannedToken, ParseError> {
        if self.check(expected) {
            Ok(self.advance().expect("check() guarantees a next token"))
        } else if self.is_at_end() {
            Err(ParseError::UnexpectedEof {
                span: self.eof_span.clone(),
                expected: description.to_string(),
            })
        } else {
            let found = self
                .peek()
                .map(|t| t.label().to_string())
                .unwrap_or_else(|| "end of input".to_string());
            Err(ParseError::UnexpectedToken {
                span: self.peek_span(),
                found,
                expected: description.to_string(),
            })
        }
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Some(Token::Newline)) {
            self.advance();
        }
    }

    /// Skips newlines, indents, and dedents inside `(...)` lists where
    /// significant whitespace is irrelevant. The lexer always injects
    /// Indent/Dedent around indented content, even inside parentheses, so
    /// argument and parameter lists must swallow those tokens to support
    /// multi-line call and signature layouts.
    fn skip_paren_whitespace(&mut self) {
        while matches!(
            self.peek(),
            Some(Token::Newline | Token::Indent | Token::Dedent)
        ) {
            self.advance();
        }
    }

    // ------------------------------------------------------------------
    // Top level
    // ------------------------------------------------------------------

    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let start = self.tokens.first().map(|(_, s)| s.start).unwrap_or(0);
        let end = self.tokens.last().map(|(_, s)| s.end).unwrap_or(0);

        let mut items = Vec::new();
        self.skip_newlines();
        while !self.is_at_end() {
            items.push(self.parse_item()?);
            self.skip_newlines();
        }
        Ok(Program {
            items,
            span: start..end,
        })
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        match self.peek() {
            Some(Token::Fn) => Ok(Item::Function(self.parse_function()?)),
            Some(other) => Err(ParseError::UnexpectedToken {
                span: self.peek_span(),
                found: other.label().to_string(),
                expected: "a top-level declaration (`fn`)".to_string(),
            }),
            None => Err(ParseError::UnexpectedEof {
                span: self.eof_span.clone(),
                expected: "a top-level declaration".to_string(),
            }),
        }
    }

    // ------------------------------------------------------------------
    // Functions
    // ------------------------------------------------------------------

    fn parse_function(&mut self) -> Result<FunctionDecl, ParseError> {
        let fn_tok = self.expect(&Token::Fn, "`fn`")?;
        let start = fn_tok.1.start;

        let name = self.parse_identifier()?;
        self.expect(&Token::LParen, "`(` to open parameter list")?;
        let params = self.parse_param_list()?;
        self.expect(&Token::RParen, "`)` to close parameter list")?;

        let return_type = if matches!(self.peek(), Some(Token::Arrow)) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(&Token::Colon, "`:` before function body")?;
        self.expect(&Token::Newline, "newline after function header")?;
        let body = self.parse_block()?;
        let end = body.span.end;

        Ok(FunctionDecl {
            name,
            params,
            return_type,
            body,
            span: start..end,
        })
    }

    fn parse_param_list(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        // Tolerate newlines/indents/dedents inside the parameter list so
        // users can break long signatures across lines.
        self.skip_paren_whitespace();
        if matches!(self.peek(), Some(Token::RParen)) {
            return Ok(params);
        }
        loop {
            params.push(self.parse_param()?);
            self.skip_paren_whitespace();
            if matches!(self.peek(), Some(Token::Comma)) {
                self.advance();
                self.skip_paren_whitespace();
            } else {
                break;
            }
        }
        Ok(params)
    }

    fn parse_param(&mut self) -> Result<Param, ParseError> {
        let mutable = if matches!(self.peek(), Some(Token::Mut)) {
            self.advance();
            true
        } else {
            false
        };
        let name = self.parse_identifier()?;
        self.expect(&Token::Colon, "`:` before parameter type")?;
        let ty = self.parse_type()?;
        let span = name.span.start..ty.span.end;
        Ok(Param {
            name,
            mutable,
            ty,
            span,
        })
    }

    // ------------------------------------------------------------------
    // Types
    // ------------------------------------------------------------------

    fn parse_type(&mut self) -> Result<TypeRef, ParseError> {
        let mut is_ref = false;
        let mut is_mut_ref = false;
        let start: usize;

        if matches!(self.peek(), Some(Token::Ampersand)) {
            let amp = self.advance().expect("peeked Ampersand");
            start = amp.1.start;
            is_ref = true;
            if matches!(self.peek(), Some(Token::Mut)) {
                self.advance();
                is_mut_ref = true;
            }
        } else {
            start = self.peek_span().start;
        }

        // Name must be an identifier — keywords like `int` are lexed as
        // identifiers per spec §2.3 (primitives are not reserved words).
        let (tok, name_span) = self.advance().ok_or_else(|| ParseError::UnexpectedEof {
            span: self.eof_span.clone(),
            expected: "a type".to_string(),
        })?;
        let name = match tok {
            Token::Identifier(n) => n,
            other => {
                return Err(ParseError::ExpectedType {
                    span: name_span,
                    found: other.label().to_string(),
                });
            }
        };

        let mut end = name_span.end;
        let mut args = Vec::new();
        if matches!(self.peek(), Some(Token::Lt)) {
            self.advance();
            loop {
                args.push(self.parse_type()?);
                if matches!(self.peek(), Some(Token::Comma)) {
                    self.advance();
                } else {
                    break;
                }
            }
            let close = self.expect(&Token::Gt, "`>` to close generic arguments")?;
            end = close.1.end;
        }

        Ok(TypeRef {
            name,
            args,
            is_ref,
            is_mut_ref,
            span: start..end,
        })
    }

    // ------------------------------------------------------------------
    // Blocks and statements
    // ------------------------------------------------------------------

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let indent_tok = self.expect(&Token::Indent, "indented block body")?;
        let start = indent_tok.1.start;

        let mut statements = Vec::new();
        self.skip_newlines();
        while !matches!(self.peek(), Some(Token::Dedent) | None) {
            statements.push(self.parse_statement()?);
            self.skip_newlines();
        }

        if statements.is_empty() {
            return Err(ParseError::EmptyBlock {
                span: start..self.peek_span().end,
            });
        }

        let dedent_tok = self.expect(&Token::Dedent, "end of block (dedent)")?;
        Ok(Block {
            statements,
            span: start..dedent_tok.1.end,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek() {
            Some(Token::Let) => Ok(Statement::Let(self.parse_let()?)),
            Some(Token::Return) => Ok(Statement::Return(self.parse_return()?)),
            Some(_) => {
                let expr = self.parse_expression()?;
                self.expect(&Token::Newline, "newline to end statement")?;
                Ok(Statement::Expr(expr))
            }
            None => Err(ParseError::UnexpectedEof {
                span: self.eof_span.clone(),
                expected: "a statement".to_string(),
            }),
        }
    }

    fn parse_let(&mut self) -> Result<LetStmt, ParseError> {
        let let_tok = self.expect(&Token::Let, "`let`")?;
        let start = let_tok.1.start;

        let mutable = if matches!(self.peek(), Some(Token::Mut)) {
            self.advance();
            true
        } else {
            false
        };
        let name = self.parse_identifier()?;

        let ty = if matches!(self.peek(), Some(Token::Colon)) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(&Token::Eq, "`=` in let statement")?;
        let value = self.parse_expression()?;
        let nl = self.expect(&Token::Newline, "newline after let statement")?;
        Ok(LetStmt {
            name,
            mutable,
            ty,
            value,
            span: start..nl.1.end,
        })
    }

    fn parse_return(&mut self) -> Result<ReturnStmt, ParseError> {
        let ret = self.expect(&Token::Return, "`return`")?;
        let start = ret.1.start;

        let value = if matches!(self.peek(), Some(Token::Newline)) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        let nl = self.expect(&Token::Newline, "newline after return")?;
        Ok(ReturnStmt {
            value,
            span: start..nl.1.end,
        })
    }

    // ------------------------------------------------------------------
    // Expressions — climbing precedence
    //
    // Order, lowest → highest, matches spec §7:
    //   or, and, equality, comparison, additive, multiplicative,
    //   unary, power, postfix, primary
    // ------------------------------------------------------------------

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expression, ParseError> {
        let mut lhs = self.parse_and()?;
        while matches!(self.peek(), Some(Token::Or)) {
            self.advance();
            let rhs = self.parse_and()?;
            lhs = binary(BinOp::Or, lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> Result<Expression, ParseError> {
        let mut lhs = self.parse_equality()?;
        while matches!(self.peek(), Some(Token::And)) {
            self.advance();
            let rhs = self.parse_equality()?;
            lhs = binary(BinOp::And, lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_equality(&mut self) -> Result<Expression, ParseError> {
        let mut lhs = self.parse_comparison()?;
        loop {
            let op = match self.peek() {
                Some(Token::EqEq) => BinOp::Eq,
                Some(Token::NotEq) => BinOp::NotEq,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_comparison()?;
            lhs = binary(op, lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let mut lhs = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                Some(Token::Lt) => BinOp::Lt,
                Some(Token::LtEq) => BinOp::LtEq,
                Some(Token::Gt) => BinOp::Gt,
                Some(Token::GtEq) => BinOp::GtEq,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_additive()?;
            lhs = binary(op, lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_additive(&mut self) -> Result<Expression, ParseError> {
        let mut lhs = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_multiplicative()?;
            lhs = binary(op, lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, ParseError> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Some(Token::Star) => BinOp::Mul,
                Some(Token::Slash) => BinOp::Div,
                Some(Token::Percent) => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_unary()?;
            lhs = binary(op, lhs, rhs);
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        let op = match self.peek() {
            Some(Token::Minus) => Some(UnOp::Neg),
            Some(Token::Not) => Some(UnOp::Not),
            _ => None,
        };
        if let Some(op) = op {
            let tok = self.advance().expect("peeked unary operator");
            let operand = self.parse_unary()?;
            let span = tok.1.start..operand.span().end;
            return Ok(Expression::Unary(Unary {
                op,
                operand: Box::new(operand),
                span,
            }));
        }
        self.parse_power()
    }

    /// `^` is right-associative per spec §7.
    fn parse_power(&mut self) -> Result<Expression, ParseError> {
        let base = self.parse_postfix()?;
        if matches!(self.peek(), Some(Token::Caret)) {
            self.advance();
            let rhs = self.parse_unary()?;
            Ok(binary(BinOp::Pow, base, rhs))
        } else {
            Ok(base)
        }
    }

    fn parse_postfix(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                Some(Token::Dot) => {
                    self.advance();
                    let field = self.parse_identifier()?;
                    let span = expr.span().start..field.span.end;
                    expr = Expression::FieldAccess(FieldAccess {
                        base: Box::new(expr),
                        field,
                        span,
                    });
                }
                Some(Token::LParen) => {
                    self.advance();
                    let args = self.parse_arg_list()?;
                    let rparen = self.expect(&Token::RParen, "`)` to close call")?;
                    let span = expr.span().start..rparen.1.end;
                    expr = Expression::Call(Call {
                        callee: Box::new(expr),
                        args,
                        span,
                    });
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expression>, ParseError> {
        let mut args = Vec::new();
        self.skip_paren_whitespace();
        if matches!(self.peek(), Some(Token::RParen)) {
            return Ok(args);
        }
        loop {
            args.push(self.parse_expression()?);
            self.skip_paren_whitespace();
            if matches!(self.peek(), Some(Token::Comma)) {
                self.advance();
                self.skip_paren_whitespace();
                // Trailing comma — stop before attempting another expression.
                if matches!(self.peek(), Some(Token::RParen)) {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        // Clone the next token so we can move its payload (String) into the
        // AST. Cloning happens once per primary and is dwarfed by later
        // compiler passes.
        let peeked = match self.peek() {
            Some(t) => t.clone(),
            None => {
                return Err(ParseError::UnexpectedEof {
                    span: self.eof_span.clone(),
                    expected: "an expression".to_string(),
                });
            }
        };

        match peeked {
            Token::IntLiteral(value) => {
                let (_, span) = self.advance().expect("peeked IntLiteral");
                Ok(Expression::Literal(Literal::Int { value, span }))
            }
            Token::FloatLiteral(value) => {
                let (_, span) = self.advance().expect("peeked FloatLiteral");
                Ok(Expression::Literal(Literal::Float { value, span }))
            }
            Token::StringLiteral(value) => {
                let (_, span) = self.advance().expect("peeked StringLiteral");
                Ok(Expression::Literal(Literal::Str { value, span }))
            }
            Token::FString(value) => {
                let (_, span) = self.advance().expect("peeked FString");
                Ok(Expression::Literal(Literal::FStr { value, span }))
            }
            Token::True => {
                let (_, span) = self.advance().expect("peeked True");
                Ok(Expression::Literal(Literal::Bool { value: true, span }))
            }
            Token::False => {
                let (_, span) = self.advance().expect("peeked False");
                Ok(Expression::Literal(Literal::Bool { value: false, span }))
            }
            Token::Nil => {
                let (_, span) = self.advance().expect("peeked Nil");
                Ok(Expression::Literal(Literal::Nil { span }))
            }
            Token::Identifier(name) => {
                let (_, span) = self.advance().expect("peeked Identifier");
                Ok(Expression::Identifier(Identifier { name, span }))
            }
            Token::LParen => {
                let lparen = self.advance().expect("peeked LParen");
                self.skip_newlines();
                let inner = self.parse_expression()?;
                self.skip_newlines();
                let rparen = self.expect(&Token::RParen, "`)` to close grouped expression")?;
                let span = lparen.1.start..rparen.1.end;
                Ok(Expression::Grouped(Grouped {
                    inner: Box::new(inner),
                    span,
                }))
            }
            other => {
                let span = self.peek_span();
                Err(ParseError::InvalidExpression {
                    span,
                    found: other.label().to_string(),
                })
            }
        }
    }

    fn parse_identifier(&mut self) -> Result<Identifier, ParseError> {
        match self.peek().cloned() {
            Some(Token::Identifier(name)) => {
                let (_, span) = self.advance().expect("peeked Identifier");
                Ok(Identifier { name, span })
            }
            Some(other) => {
                let span = self.peek_span();
                Err(ParseError::ExpectedIdentifier {
                    span,
                    found: other.label().to_string(),
                })
            }
            None => Err(ParseError::UnexpectedEof {
                span: self.eof_span.clone(),
                expected: "an identifier".to_string(),
            }),
        }
    }
}

/// Convenience constructor — builds a `Binary` expression and computes
/// the enclosing span from the two children.
fn binary(op: BinOp, lhs: Expression, rhs: Expression) -> Expression {
    let span = lhs.span().start..rhs.span().end;
    Expression::Binary(Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        span,
    })
}

// ======================================================================
// Errors
// ======================================================================

/// Errors produced by the HMAT parser.
///
/// Codes `E010`–`E099` per spec §9. Every variant carries a span and
/// a human-readable `help()` hint so the diagnostic renderer can produce
/// the "what went wrong + how to fix it" format the compiler promises.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ParseError {
    /// Saw a token that is valid HMAT but not the one this position expected.
    /// Emitted with error code `E010`.
    #[error("E010: expected {expected}, found {found}")]
    UnexpectedToken {
        span: Span,
        found: String,
        expected: String,
    },

    /// Source ended before a parser rule was satisfied.
    /// Emitted with error code `E011`.
    #[error("E011: unexpected end of input — expected {expected}")]
    UnexpectedEof { span: Span, expected: String },

    /// A position required an identifier (function name, parameter name,
    /// field name, etc.) but got something else.
    /// Emitted with error code `E012`.
    #[error("E012: expected an identifier, found {found}")]
    ExpectedIdentifier { span: Span, found: String },

    /// A position required a type reference but got something else.
    /// Emitted with error code `E013`.
    #[error("E013: expected a type, found {found}")]
    ExpectedType { span: Span, found: String },

    /// A block `INDENT ... DEDENT` opened but contained no statements.
    /// Emitted with error code `E014`.
    #[error("E014: empty block — expected at least one statement")]
    EmptyBlock { span: Span },

    /// Parser was trying to build an expression but hit a token that
    /// cannot start one.
    /// Emitted with error code `E015`.
    #[error("E015: invalid expression — unexpected {found}")]
    InvalidExpression { span: Span, found: String },
}

impl ParseError {
    /// Returns the span this error points to, for diagnostic rendering.
    pub fn span(&self) -> &Span {
        match self {
            ParseError::UnexpectedToken { span, .. } => span,
            ParseError::UnexpectedEof { span, .. } => span,
            ParseError::ExpectedIdentifier { span, .. } => span,
            ParseError::ExpectedType { span, .. } => span,
            ParseError::EmptyBlock { span } => span,
            ParseError::InvalidExpression { span, .. } => span,
        }
    }

    /// Returns the four-character diagnostic code (e.g. `E010`).
    pub fn code(&self) -> &'static str {
        match self {
            ParseError::UnexpectedToken { .. } => "E010",
            ParseError::UnexpectedEof { .. } => "E011",
            ParseError::ExpectedIdentifier { .. } => "E012",
            ParseError::ExpectedType { .. } => "E013",
            ParseError::EmptyBlock { .. } => "E014",
            ParseError::InvalidExpression { .. } => "E015",
        }
    }

    /// Returns an actionable fix hint. Every parser error has one — the Zen
    /// of HMAT #2: errors are teachers, not walls.
    pub fn help(&self) -> &'static str {
        match self {
            ParseError::UnexpectedToken { .. } => {
                "check the surrounding syntax against the expected form — \
                 a misplaced punctuation or keyword is the usual culprit"
            }
            ParseError::UnexpectedEof { .. } => {
                "the file ended mid-declaration — add the missing piece \
                 (often a closing `)`, `:`, or indented body)"
            }
            ParseError::ExpectedIdentifier { .. } => {
                "this position needs a name — identifiers must start with a \
                 letter or `_` and contain only letters, digits, and `_`"
            }
            ParseError::ExpectedType { .. } => {
                "this position needs a type annotation — e.g. `int`, `str`, \
                 `Point`, `Result<int, str>`, `&Point`"
            }
            ParseError::EmptyBlock { .. } => {
                "a block cannot be empty — add at least one statement, or use \
                 `pass` once Phase 2 introduces it"
            }
            ParseError::InvalidExpression { .. } => {
                "expected a value here — a literal, identifier, function call, \
                 or a `(grouped expression)`"
            }
        }
    }
}
