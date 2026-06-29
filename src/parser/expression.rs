use crate::lexer::Token;
use crate::parser::Precedence;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Identifier(String),

    IntLiteral(isize),
    BoolLiteral(bool),
    StringLiteral(String),

    Prefix {
        operator: Token,
        right: Box<Expression>,
    },

    Infix {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
}

pub fn token_precedence(token: &Token) -> Precedence {
    match token {
        Token::Assign => Precedence::Assign,
        Token::Equals | Token::NotEquals => Precedence::Equals,
        Token::Less | Token::Greater | Token::LessOrEquals | Token::GreaterOrEquals => Precedence::LessGreater,
        Token::Add | Token::Subtract => Precedence::Sum,
        Token::Multiply | Token::Divide | Token::Modulus => Precedence::Product,
        Token::Power => Precedence::Power,
        _ => Precedence::Lowest,
    }
}