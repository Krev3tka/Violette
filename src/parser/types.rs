use crate::lexer::Token;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum Precedence {
    Lowest,      // Base syntax
    Assign,
    Equals,      // == !=
    LessGreater, // < > <= >=
    Sum,         // + -
    Product,     // * / %
    Prefix,      // -X, !X ++X --X
    Power,       // **
    Postfix,     // x++ x-- () [] .
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(Token),
}
