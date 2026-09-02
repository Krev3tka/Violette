use crate::lexer::span::Span;
use crate::lexer::token::Token;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    ExpectedIdentifier {
        context: &'static str,
        found: Token,
        span: Span
    },

    ExpectedExpression {
        context: &'static str,
        found: Token,
        span: Span
    },

    ExpectedAssign {
        context: &'static str,
        found: Token,
        span: Span,
    },

    UnclosedDelimiter {
        open_token: Token,
        open_span: Span,
        expected_token: Token,
        found_tok: Token,
        found_span: Span,
    },

    ExpectedFatArrow {
        found: Token,
        span: Span,
    },

    ExpectedType {
        context: &'static str,
        found: Token,
        span: Span
    },

    InvalidTypeSyntax {
        desc: &'static str,
        span: Span
    },


    InvalidForLoopSyntax {
        span: Span
    },

    InvalidElseBranch {
        found: Token,
        span: Span,
    },

    InvalidSyntaxInExtend {
        found: Token,
        span: Span,
    },


    UnexpectedEof {
        expected: &'static str,
        span: Span,
    },

    TooDeep {
        span: Span
    },

    UnexpectedToken {
        token: Token,
        span: Span
    },
}