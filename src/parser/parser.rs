use crate::lexer::{Lexer, Token};
use crate::parser::{Expression, Precedence, Statement};
use crate::parser::expression::token_precedence;
use crate::parser::Precedence::{Equals, LessGreater, Lowest, Prefix, Product, Sum};
use crate::parser::statement::{ElseIf, IfStatement};
use crate::parser::Statement::{ForCondition, ForCounter};
use crate::parser::types::ParseError;

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();

        Parser {
            lexer,
            current_token,
            peek_token,
        }
    }

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
                Token::Assign | Token::Power => {
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
                }
                _ => break,
            }
        }

        Ok(left)
    }

    pub fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match &self.current_token {
            Token::Let | Token::Const => {
                let is_const = matches!(self.current_token, Token::Const);
                self.next_token();
                let name = match &self.current_token {
                    Token::Identifier(var_name) => var_name.clone(),
                    _ => return Err(ParseError::UnexpectedToken(self.current_token.clone())),
                };

                self.next_token();
                if !matches!(self.current_token, Token::Assign) {
                    return Err(ParseError::UnexpectedToken(self.current_token.clone()));
                }

                self.next_token();

                let value = self.parse_expression(Lowest)?;

                if is_const {
                    Ok(Statement::Const { name, value })
                } else {
                    Ok(Statement::Let { name, value })
                }

            }
            Token::If => self.parse_if_statement(),
            Token::For => self.parse_for_statement(),
            _ => {
                let expr = self.parse_expression(Lowest)?;
                Ok(Statement::ExpressionStatement {
                    expression: expr
                })
            }
        }
    }

    pub fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::If)?;

        let condition = self.parse_expression(Lowest)?;
        self.next_token();

        self.expect(Token::LeftBrace)?;

        let then_block = self.parse_block()?;

        let mut else_if = Vec::new();
        let mut else_block = None;

        while matches!(self.current_token, Token::Else) {
            self.expect(Token::Else)?;

            if matches!(self.current_token, Token::If) {
                let else_if_stmt = self.parse_else_if_statement()?;
                else_if.push(else_if_stmt);
            } else if matches!(self.current_token, Token::LeftBrace) {
                self.next_token();
                else_block = Some(self.parse_block()?);
                break;
            } else {
                return Err(ParseError::UnexpectedToken(self.current_token.clone()));
            }
        }

        Ok(Statement::If(IfStatement {
            condition,
            then_block,
            else_if,
            else_block,
        }))
    }

    pub fn parse_else_if_statement(&mut self) -> Result<ElseIf, ParseError> {
        self.expect(Token::If)?;

        let condition = self.parse_expression(Lowest)?;
        self.next_token();

        self.expect(Token::LeftBrace)?;

        let block = self.parse_block()?;

        Ok(ElseIf {
            condition,
            block
        })
    }

    pub fn parse_for_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::For)?;

        if let Token::Identifier(var) = self.current_token.clone() {
            if matches!(self.peek_token, Token::In) {
                self.expect(Token::Identifier("".to_string()))?;
                self.expect(Token::In)?;
                self.parse_for_range(&var)
            } else if matches!(self.peek_token, Token::Assign) {
                self.parse_for_counter(&var)
            } else {
                self.parse_for_condition()
            }
        } else {
            Err(ParseError::UnexpectedToken(self.current_token.clone()))
        }
    }

    pub fn parse_for_range(&mut self, var: &String) -> Result<Statement, ParseError> {
        let variable = var.clone();

        let iterable = self.parse_expression(Lowest)?;
        self.next_token();
        self.skip_newlines();

        if !matches!(self.current_token, Token::LeftBrace) {
            return Err(ParseError::UnexpectedToken(self.current_token.clone()))
        }

        self.expect(Token::LeftBrace)?;
        self.skip_newlines();
        let body = self.parse_block()?;

        Ok(Statement::ForRange {
            variable,
            iterable,
            body,
        })
    }

    pub fn parse_for_counter(&mut self, var: &String) -> Result<Statement, ParseError> {
        let name = var.clone();
        self.expect(Token::Identifier("".to_string()))?;
        self.expect(Token::Assign)?;

        let value = self.parse_expression(Lowest)?;
        let init = Box::new(Statement::Let { name, value });

        self.expect(Token::Int(0))?;

        if !matches!(self.current_token, Token::Semicolon) {
            return Err(ParseError::UnexpectedToken(self.current_token.clone()))
        }

        self.expect(Token::Semicolon)?;

        let condition = self.parse_expression(Lowest)?;
        self.next_token();

        if !matches!(self.current_token, Token::Semicolon) {
            return Err(ParseError::UnexpectedToken(self.current_token.clone()))
        }

        self.expect(Token::Semicolon)?;

        let post = self.parse_expression(Lowest)?;
        self.expect(Token::Increment)?;
        self.expect(Token::LeftBrace)?;
        self.skip_newlines();

        let body = self.parse_block()?;

        Ok(ForCounter {
            init,
            condition,
            post,
            body
        })
    }

    pub fn parse_for_condition(&mut self) -> Result<Statement, ParseError> {
        let condition = self.parse_expression(Lowest)?;
        self.next_token();

        if !matches!(self.current_token, Token::LeftBrace) {
            return Err(ParseError::UnexpectedToken(self.current_token.clone()))
        }

        self.next_token();
        let body = self.parse_block()?;

        Ok(ForCondition {
            condition,
            body
        })
    }

    pub fn parse_block(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();

        self.skip_newlines();

        while !matches!(self.current_token, Token::RightBrace | Token::EOF) {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.next_token();
            self.skip_newlines();
        }

        if matches!(self.current_token, Token::RightBrace) {
            self.next_token();
        }

        Ok(statements)
    }

    pub fn parse_program(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();

        self.skip_newlines();

        while !matches!(self.current_token, Token::EOF) {
            let stmt = self.parse_statement()?;
            statements.push(stmt);

            self.skip_newlines();
        }
        Ok(statements)
    }

    pub fn next_token(&mut self) {
        self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token())
    }

    pub fn skip_newlines(&mut self) {
        while matches!(self.current_token, Token::Newline) {
            self.next_token();
        }
    }

    fn current_precedence(&self) -> Precedence {
        token_precedence(&self.current_token)
    }

    fn peek_precedence(&self) -> Precedence {
        token_precedence(&self.peek_token)
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {
            self.next_token();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(self.current_token.clone()))
        }
    }
}