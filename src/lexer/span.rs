use crate::lexer::token::Token;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Position {
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number.
    pub col: usize,
}

impl Position {
    pub fn new(line: usize, col: usize) -> Self {
        Position { line, col }
    }
}

/// Source code location represented by line and column numbers.
///
/// Used for diagnostic reporting and error tracking.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Span { start, end }
    }
}

/// Token paired with its source code location.
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}
