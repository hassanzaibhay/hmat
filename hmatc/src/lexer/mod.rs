//! HMAT Lexer.
//!
//! Tokenizes HMAT source code into a flat stream of [`Token`]s with byte
//! spans. Significant whitespace is handled by a post-processing pass that
//! injects [`Token::Indent`] and [`Token::Dedent`] tokens driven by the
//! indentation of each logical (non-blank, non-comment) line.
//!
//! # Two-phase design
//!
//! 1. **Raw tokenization** — `logos` produces a flat stream of tokens with
//!    inter-token whitespace and `#`-to-end-of-line comments stripped.
//!    Physical `\n` bytes are preserved as [`Token::Newline`] tokens.
//! 2. **Indent injection** — the raw stream is walked once. At the start of
//!    each logical line the indentation column of the first content token is
//!    compared against an indent stack; `Indent`/`Dedent` tokens are emitted
//!    accordingly. Blank and comment-only lines are skipped without affecting
//!    indentation, per spec §2.2.
//!
//! # Indentation
//!
//! - 4 spaces is the canonical step, but any consistent width works.
//! - A tab counts as 4 columns for indent purposes. `hfmt` will normalize
//!   later; the lexer accepts the mixed form.
//! - An indent level that decreases to a value that was never opened is an
//!   [`LexError::IndentMismatch`] error.
//! - Outstanding indents are closed with `Dedent` tokens at end-of-file so
//!   every `Indent` has a matching `Dedent`.

mod error;
mod token;

pub use error::{LexError, Span};
pub use token::Token;

use logos::Logos;

/// A token paired with its byte span in the original source.
pub type SpannedToken = (Token, Span);

/// Tokenizes HMAT source into a stream of tokens with spans, including
/// synthesised `Indent`/`Dedent` tokens for significant whitespace.
///
/// # Errors
/// Returns [`LexError`] on the first invalid token, malformed literal, or
/// indentation inconsistency encountered.
///
/// # Example
/// ```ignore
/// let tokens = hmatc::lexer::tokenize("fn main():\n    print(\"hi\")\n")?;
/// assert!(matches!(tokens[0].0, hmatc::lexer::Token::Fn));
/// ```
pub fn tokenize(source: &str) -> Result<Vec<SpannedToken>, LexError> {
    // Phase 1: run the logos lexer to produce a flat raw stream.
    let raw = raw_tokenize(source)?;

    // Phase 2: walk the stream injecting Indent/Dedent and collapsing
    // newlines that terminate blank or comment-only lines.
    inject_indentation(source, raw)
}

/// Runs the `logos` state machine across `source`, returning all tokens in
/// order. Stops and returns the first invalid-token error.
fn raw_tokenize(source: &str) -> Result<Vec<SpannedToken>, LexError> {
    let mut out = Vec::new();
    let mut lex = Token::lexer(source);

    while let Some(result) = lex.next() {
        let span = lex.span();
        match result {
            Ok(tok) => out.push((tok, span)),
            Err(()) => {
                let snippet = lex.slice().to_string();
                // Distinguish "obviously a literal that failed to parse" from
                // "unknown character" for better diagnostics.
                let kind = literal_kind_from_snippet(&snippet);
                return Err(match kind {
                    Some(k) => LexError::MalformedLiteral {
                        span,
                        kind: k,
                        snippet,
                    },
                    None => LexError::InvalidToken { span, snippet },
                });
            }
        }
    }
    Ok(out)
}

/// Heuristic: if the unrecognised slice looks like it was trying to be a
/// literal, classify it so the diagnostic can say so.
fn literal_kind_from_snippet(snippet: &str) -> Option<&'static str> {
    let first = snippet.chars().next()?;
    if first == '"' {
        Some("string")
    } else if first == 'f' && snippet.starts_with("f\"") {
        Some("string")
    } else if first.is_ascii_digit() {
        if snippet.contains('.') {
            Some("float")
        } else {
            Some("integer")
        }
    } else {
        None
    }
}

/// Post-processes the raw token stream to emit `Indent`/`Dedent` tokens and
/// collapse newlines spanning blank or comment-only lines.
fn inject_indentation(
    source: &str,
    raw: Vec<SpannedToken>,
) -> Result<Vec<SpannedToken>, LexError> {
    let mut out: Vec<SpannedToken> = Vec::with_capacity(raw.len());
    let mut indent_stack: Vec<usize> = vec![0];
    let mut at_line_start = true;

    for (tok, span) in raw {
        if matches!(tok, Token::Newline) {
            // Suppress newlines for blank or comment-only lines (we only
            // reach a Newline while at_line_start when nothing else was on
            // this logical line).
            if !at_line_start {
                out.push((Token::Newline, span));
                at_line_start = true;
            }
            continue;
        }

        if at_line_start {
            let col = indent_column_of(source, span.start);
            let current = *indent_stack.last().expect("indent stack never empty");

            if col > current {
                indent_stack.push(col);
                out.push((Token::Indent, span.start..span.start));
            } else if col < current {
                while *indent_stack.last().expect("indent stack never empty") > col {
                    indent_stack.pop();
                    out.push((Token::Dedent, span.start..span.start));
                }
                let now = *indent_stack.last().expect("indent stack never empty");
                if now != col {
                    return Err(LexError::IndentMismatch {
                        span: span.start..span.start,
                        expected: now,
                        found: col,
                    });
                }
            }
            at_line_start = false;
        }

        out.push((tok, span));
    }

    // End-of-file: emit a trailing newline if the last line carried content
    // without a terminating `\n`, then close any still-open indent levels.
    let eof = source.len()..source.len();
    if !at_line_start {
        out.push((Token::Newline, eof.clone()));
    }
    while indent_stack.len() > 1 {
        indent_stack.pop();
        out.push((Token::Dedent, eof.clone()));
    }

    Ok(out)
}

/// Computes the effective indentation column of the content token starting
/// at `byte_offset` by scanning the characters between the preceding `\n`
/// (or start of file) and `byte_offset`. Each space counts as 1 column and
/// each tab as 4, per spec §2.2.
fn indent_column_of(source: &str, byte_offset: usize) -> usize {
    let prefix = &source[..byte_offset];
    let line_start = prefix.rfind('\n').map(|p| p + 1).unwrap_or(0);
    let indent_slice = &source[line_start..byte_offset];
    let mut col = 0;
    for c in indent_slice.chars() {
        match c {
            ' ' => col += 1,
            '\t' => col += 4,
            // Non-whitespace shouldn't appear here because logos has already
            // skipped inter-token whitespace. Break defensively.
            _ => break,
        }
    }
    col
}
