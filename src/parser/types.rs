use crate::lexer::span::Span;
use crate::lexer::token::{PrimitiveType, Token};

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum Precedence {
    Lowest, // Base syntax
    Assign,
    LogicOr,     // ||
    LogicAnd,    // &&
    BitOr,       // #
    BitXor,      // ^
    BitAnd,      // &
    Equals,      // == !=
    LessGreater, // < > <= >=
    Sprout,      // ~>
    Shift,       // >> <<
    Sum,         // + -
    Product,     // * / %
    Prefix,      // -X !X ++X --X
    Power,       // **
    Postfix,     // x++ x-- () [] .
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
    UnexpectedToken {
        token: Token,
        span: Span,
    },
    UnexpectedEof,
    Expected {
        expected: Token,
        found: Token,
        span: Span,
    },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken { token, span } => write!(
                f,
                "unexpected token: {:?} at {}:{}",
                token, span.line, span.col
            ),
            ParseError::UnexpectedEof => write!(f, "unexpected end of file"),
            ParseError::Expected {
                expected,
                found,
                span,
            } => write!(
                f,
                "expected {:?}, found {:?} at {}:{}",
                expected, found, span.line, span.col
            ),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Primitive(PrimitiveType),

    Named(TypePath),

    Generic { name: String, param: Box<Type> },

    Union(Vec<Type>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypePath {
    pub segments: Vec<String>,
}
