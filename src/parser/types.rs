use crate::lexer::token::{PrimitiveType, Token};

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum Precedence {
    Lowest, // Base syntax
    Assign,
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

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(Token),
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
