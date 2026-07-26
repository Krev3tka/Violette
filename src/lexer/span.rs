use crate::lexer::token::Token;

#[derive(Debug, Eq, Clone, Copy)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

impl PartialEq for Span {
    fn eq(&self, _: &Self) -> bool { true }
}

impl Span {
    pub fn new(line: usize, col: usize) -> Self {
        Span { line, col }
    }
}



#[derive(Debug, PartialEq, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}
