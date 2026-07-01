use crate::lexer::token::Token;
use crate::parser::{ParseError, Precedence, Statement};
use crate::parser::parser::Parser;
use crate::parser::Precedence::{Lowest, Prefix};

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

    Postfix {
        left: Box<Expression>,
        operator: Token
    },

    Index {
        left: Box<Expression>,
        index: Box<Expression>
    },

    Call {
        function: Box<Expression>,
        args: Vec<Expression>,
    }
}

pub fn token_precedence(token: &Token) -> Precedence {
    match token {
        Token::Assign => Precedence::Assign,
        Token::Equals | Token::NotEquals |
        Token::AddAndAssign | Token::SubAndAssign |
        Token::MulAndAssign | Token::DivAndAssign |
        Token::ModAndAssign => Precedence::Equals,
        Token::Less | Token::Greater | Token::LessOrEquals | Token::GreaterOrEquals => Precedence::LessGreater,
        Token::Add | Token::Subtract => Precedence::Sum,
        Token::Multiply | Token::Divide | Token::Modulus => Precedence::Product,
        Token::Power => Precedence::Power,
        Token::Increment | Token::Decrement | Token::LeftParen | Token::LSB => Precedence::Postfix,
        _ => Precedence::Lowest,
    }
}

impl Parser {
    pub fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, ParseError> {
        let mut left = match &self.current_token {
            Token::Int(v) => Expression::IntLiteral(*v),
            Token::Identifier(s) => Expression::Identifier(s.clone()),
            Token::Bool(b) => Expression::BoolLiteral(*b),
            Token::String(s) => Expression::StringLiteral(s.clone()),
            Token::LeftParen => {
                self.next_token();
                let expr = self.parse_expression(Lowest)?;

                if matches!(self.peek_token, Token::RightParen) {
                    self.next_token();
                    expr
                } else {
                    return Err(ParseError::UnexpectedToken(self.current_token.clone()));
                }
            }
            Token::Subtract | Token::LogicNot | Token::Increment | Token::Decrement => {
                let operator = self.current_token.clone();

                self.next_token();

                let right = self.parse_expression(Prefix);

                Expression::Prefix {
                    operator,
                    right: Box::new(right?),
                }
            }
            _ => return Err(ParseError::UnexpectedToken(self.current_token.clone())),
        };

        while precedence < self.peek_precedence()
            || (precedence == self.peek_precedence() && matches!(self.peek_token, Token::Assign | Token::Power))
        {
            match &self.peek_token {
                Token::Add | Token::Subtract | Token::Multiply | Token::Divide | Token::Modulus |
                Token::Equals | Token::NotEquals | Token::Less | Token::Greater | Token::LessOrEquals | Token::GreaterOrEquals |
                Token::Assign | Token::Power | Token::AddAndAssign | Token::SubAndAssign | Token::MulAndAssign | Token::DivAndAssign => {
                    let peek_prec = self.peek_precedence();
                    self.next_token();
                    let operator = self.current_token.clone();
                    self.next_token();

                    let right = self.parse_expression(peek_prec)?;

                    left = Expression::Infix {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    };
                }
                Token::Decrement | Token::Increment => {
                    self.next_token();
                    let operator = self.current_token.clone();

                    left = Expression::Postfix {
                        left: Box::new(left),
                        operator,
                    }
                }
                Token::LeftParen => {
                    self.next_token();
                    self.next_token();

                    let mut args = Vec::new();

                    while !matches!(self.current_token, Token::RightParen) {
                        args.push(self.parse_expression(Lowest)?);
                        self.next_token();

                        if matches!(self.current_token, Token::Comma) {
                            self.next_token();
                        }
                    }

                    left = Expression::Call {
                        function: Box::new(left),
                        args
                    };
                },
                Token::LSB => {
                    self.next_token();
                    left = self.parse_index_expression(left)?;
                },
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_index_expression(&mut self, left: Expression) -> Result<Expression, ParseError> {
        self.next_token();


        let index = self.parse_expression(Lowest)?;
        self.next_token();

        Ok(Expression::Index {
            left: Box::new(left),
            index: Box::new(index),
        })
    }
}