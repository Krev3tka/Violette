use crate::lexer::token::Token;
use crate::parser::Expression::Infix;
use crate::parser::Precedence::{Lowest, Prefix};
use crate::parser::parser::Parser;
use crate::parser::statement::{FunParam, MatchArm};
use crate::parser::{ParseError, Precedence, Statement};

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
        operator: Token,
    },

    Index {
        left: Box<Expression>,
        index: Box<Expression>,
    },

    Call {
        function: Box<Expression>,
        args: Vec<Expression>,
    },

    Block {
        body: Vec<Statement>,
    },

    Match {
        target: Box<Expression>,
        arms: Vec<MatchArm>,
    },

    Field {
        object: Box<Expression>,
        name: String,
    },

    MethodCall {
        object: Box<Expression>,
        name: String,
        args: Vec<Expression>,
    },
}

pub fn token_precedence(token: &Token) -> Precedence {
    match token {
        Token::Assign
        | Token::AddAndAssign
        | Token::SubAndAssign
        | Token::MulAndAssign
        | Token::DivAndAssign
        | Token::ModAndAssign => Precedence::Assign,
        Token::Equals | Token::NotEquals => Precedence::Equals,
        Token::LogicOr => Precedence::LogicOr,
        Token::LogicAnd => Precedence::LogicAnd,
        Token::BitOr => Precedence::BitOr,
        Token::BitXOR => Precedence::BitXor,
        Token::BitAnd => Precedence::BitAnd,
        Token::Less | Token::Greater | Token::LessOrEquals | Token::GreaterOrEquals => {
            Precedence::LessGreater
        }
        Token::Sprout => Precedence::Sprout,
        Token::LeftShift | Token::RightShift => Precedence::Shift,
        Token::Add | Token::Subtract => Precedence::Sum,
        Token::Multiply | Token::Divide | Token::Modulus => Precedence::Product,
        Token::Power => Precedence::Power,
        Token::Increment
        | Token::Decrement
        | Token::LeftParen
        | Token::LSB
        | Token::Pipe
        | Token::Dot => Precedence::Postfix,
        _ => Lowest,
    }
}

impl Parser {
    pub fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, ParseError> {
        let mut left = match &self.current_token.token {
            Token::Int(v) => Expression::IntLiteral(*v),
            Token::Identifier(s) => Expression::Identifier(s.clone()),
            Token::Bool(b) => Expression::BoolLiteral(*b),
            Token::String(s) => Expression::StringLiteral(s.clone()),
            Token::LeftParen => {
                self.next_token();

                let expr = self.parse_expression(Lowest)?;

                if matches!(self.peek_token.token, Token::RightParen) {
                    self.next_token();
                    expr
                } else {
                    return Err(self.unexpected(&self.current_token));
                }
            }
            Token::Subtract
            | Token::LogicNot
            | Token::BitNot
            | Token::Increment
            | Token::Decrement => {
                let operator = self.current_token.token.clone();

                self.next_token();

                let right = self.parse_expression(Prefix)?;

                Expression::Prefix {
                    operator,
                    right: Box::new(right),
                }
            }
            Token::Match => self.parse_match_expression()?,
            _ => return Err(self.unexpected(&self.current_token)),
        };

        while precedence < self.peek_precedence()
            || (precedence == self.peek_precedence()
                && matches!(self.peek_token.token, Token::Assign | Token::Power))
        {
            match &self.peek_token.token {
                Token::Add
                | Token::Subtract
                | Token::Multiply
                | Token::Divide
                | Token::Modulus
                | Token::Equals
                | Token::NotEquals
                | Token::Less
                | Token::Greater
                | Token::LessOrEquals
                | Token::GreaterOrEquals
                | Token::Assign
                | Token::Power
                | Token::AddAndAssign
                | Token::SubAndAssign
                | Token::MulAndAssign
                | Token::DivAndAssign
                | Token::ModAndAssign
                | Token::LogicAnd
                | Token::LogicOr
                | Token::BitAnd
                | Token::BitOr
                | Token::BitXOR => {
                    let peek_prec = self.peek_precedence();
                    self.next_token();
                    let operator = self.current_token.token.clone();
                    self.next_token();

                    let right = self.parse_expression(peek_prec)?;

                    left = Expression::Infix {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    };
                }
                Token::Decrement | Token::Increment | Token::Pipe => {
                    self.next_token();
                    let operator = self.current_token.token.clone();

                    left = Expression::Postfix {
                        left: Box::new(left),
                        operator,
                    }
                }
                Token::Dot => {
                    self.next_token();
                    left = self.parse_dot(left)?;
                }
                Token::LeftParen => {
                    self.next_token();
                    let args = self.parse_call_args()?;
                    left = Expression::Call {
                        function: Box::new(left),
                        args,
                    };
                }
                Token::LSB => {
                    self.next_token();

                    left = self.parse_index_expression(left)?;
                }
                Token::Sprout => {
                    self.next_token();
                    self.next_token();

                    let right = self.parse_expression(Precedence::Sprout)?;

                    left = Expression::Call {
                        function: Box::new(right),
                        args: vec![left],
                    }
                }
                Token::LeftShift | Token::RightShift => {
                    let peek_prec = self.peek_precedence();

                    self.next_token();

                    let operator = self.current_token.token.clone();

                    self.next_token();

                    let right = self.parse_expression(peek_prec)?;

                    left = Infix {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                    }
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_index_expression(&mut self, left: Expression) -> Result<Expression, ParseError> {
        self.next_token();

        let index = self.parse_expression(Lowest)?;

        if !matches!(self.peek_token.token, Token::RSB) {
            return Err(self.unexpected(&self.peek_token));
        }
        self.next_token();

        Ok(Expression::Index {
            left: Box::new(left),
            index: Box::new(index),
        })
    }

    pub fn parse_match_expression(&mut self) -> Result<Expression, ParseError> {
        self.expect(Token::Match)?;

        let target = self.parse_expression(Lowest)?;
        self.next_token();

        self.expect(Token::LeftBrace)?;
        self.skip_arm_separators();

        let mut arms = Vec::new();

        while !matches!(self.current_token.token, Token::RightBrace | Token::EOF) {
            trace_before!(self, "parse_match_expression");
            let pattern = self.parse_pattern()?;
            self.next_token();
            self.expect(Token::FatArrow)?;

            let body = if matches!(self.current_token.token, Token::LeftBrace) {
                self.next_token();
                let block_stmts = self.parse_block()?;

                Expression::Block { body: block_stmts }
            } else {
                let e = self.parse_expression(Lowest)?;
                self.next_token();
                e
            };
            arms.push(MatchArm { pattern, body });
            self.skip_arm_separators();
            trace_before!(self, "parse_match_expression");
        }

        if !matches!(self.current_token.token, Token::RightBrace | Token::EOF) {
            return Err(self.unexpected(&self.current_token));
        }

        Ok(Expression::Match {
            target: Box::new(target),
            arms,
        })
    }

    pub fn parse_dot(&mut self, left: Expression) -> Result<Expression, ParseError> {
        self.expect(Token::Dot)?;
        let name = match self.current_token.token.clone() {
            Token::Identifier(name) => name,
            _ => return Err(self.unexpected(&self.current_token)),
        };
        self.next_token();
        if matches!(self.current_token.token, Token::LeftParen) {
            let args = self.parse_call_args()?;
            Ok(Expression::MethodCall {
                object: Box::new(left),
                name,
                args,
            })
        } else {
            Ok(Expression::Field {
                object: Box::new(left),
                name,
            })
        }
    }

    pub fn parse_pattern(&mut self) -> Result<Expression, ParseError> {
        self.parse_expression(Lowest)
    }

    pub fn parse_fun_params(&mut self) -> Result<Vec<FunParam>, ParseError> {
        let mut params = Vec::new();

        while !matches!(self.current_token.token, Token::RightParen) {
            let param_name = match self.current_token.token.clone() {
                Token::Identifier(name) => name,
                _ => return Err(self.unexpected(&self.current_token)),
            };

            self.next_token();
            self.expect(Token::Colon)?;

            let param_type = self.parse_type()?;

            let param = FunParam {
                name: param_name,
                param_type,
            };

            params.push(param);

            match self.current_token.token.clone() {
                Token::Comma => self.expect(Token::Comma),
                Token::RightParen => break,
                _ => return Err(self.unexpected(&self.current_token)),
            }?;
        }

        Ok(params)
    }

    pub fn parse_call_args(&mut self) -> Result<Vec<Expression>, ParseError> {
        self.next_token();
        let mut args = Vec::new();
        while !matches!(self.current_token.token, Token::RightParen) {
            args.push(self.parse_expression(Lowest)?);
            self.next_token();
            if matches!(self.current_token.token, Token::Comma) {
                self.next_token();
            }
        }
        Ok(args)
    }

    fn skip_arm_separators(&mut self) {
        while matches!(self.current_token.token, Token::Newline | Token::Comma) {
            self.next_token();
        }
    }
}
