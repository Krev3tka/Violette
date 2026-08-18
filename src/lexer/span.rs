use crate::lexer::token::Token;

/// Source code location represented by line and column numbers.
///
/// Used for diagnostic reporting and error tracking.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Span {
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number.
    pub col: usize,
}

impl Span {
    pub fn new(line: usize, col: usize) -> Self {
        Span { line, col }
    }
}

/// Token paired with its source code location.
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}
