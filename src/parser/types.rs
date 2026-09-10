use crate::lexer::token::PrimitiveType;
use crate::parser::error::ParseError;
use crate::parser::parser::MAX_DEPTH;

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
    Range,       // : ..
    Sprout,      // ~>
    Shift,       // >> <<
    Sum,         // + -
    Product,     // * / %
    Prefix,      // -X !X ++X --X
    Power,       // **
    Postfix,     // x++ x-- () [] .
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken { token, span } => write!(
                f,
                "unexpected token: {:?} at {}:{}",
                token, span.start.line, span.start.col
            ),
            ParseError::UnexpectedEof { expected, span } => write!(
                f,
                "unexpected end of file, expected {} at the {}:{}",
                expected, span.start.line, span.start.col
            ),
            ParseError::TooDeep { span } => {
                write!(
                    f,
                    "nesting too deep (limit {}) at {}:{} — check for unbalanced delimiters",
                    MAX_DEPTH, span.start.line, span.start.col
                )
            }
            _ => write!(f, "not ready yet or idk, {:?}", self),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Primitive(PrimitiveType),

    Named(TypePath),

    Fn {
        params: Vec<Type>,
        ret: Option<Box<Type>>,
    },

    Generic {
        name: String,
        param: Box<Type>,
    },

    Union(Vec<Type>),

    Infer,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypePath {
    pub segments: Vec<String>,
}
