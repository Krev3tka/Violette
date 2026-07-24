use crate::lexer::token::Token;
use crate::parser::Precedence::Lowest;
use crate::parser::Statement::{ForCondition, ForCounter};
use crate::parser::parser::Parser;
use crate::parser::types::Type;
use crate::parser::{Expression, ParseError};
use std::cell::Cell;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    ExpressionStatement {
        expression: Expression,
    },

    Let {
        name: String,
        value: Expression,
    },

    Const {
        name: String,
        value: Expression,
    },

    If(IfStatement),

    ForCondition {
        condition: Expression,
        body: Vec<Statement>,
    },

    ForRange {
        variable: String,
        iterable: Expression,
        body: Vec<Statement>,
    },

    ForCounter {
        init: Box<Statement>,
        condition: Expression,
        post: Expression,
        body: Vec<Statement>,
    },

    Return {
        value: Option<Expression>,
    },

    Fun {
        name: String,
        params: Vec<FunParam>,
        return_type: Option<Type>,
        body: Vec<Statement>,
    },

    Struct {
        name: String,
        fields: Vec<FunParam>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_block: Vec<Statement>,
    pub else_if: Vec<ElseIf>,
    pub else_block: Option<Vec<Statement>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ElseIf {
    pub condition: Expression,
    pub block: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunParam {
    pub name: String,
    pub param_type: Type,
}

pub type StructParam = FunParam;

#[derive(Debug, PartialEq, Clone)]
pub struct MatchArm {
    pub pattern: Expression,
    pub body: Expression,
}

impl Parser {
    pub fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        self.recursion_depth.set(self.recursion_depth.get() + 1);

        struct LocalGuard(*const Cell<i32>);
        impl Drop for LocalGuard {
            fn drop(&mut self) {
                unsafe {
                    (*self.0).set((*self.0).get() - 1);
                }
            }
        }
        let _guard = LocalGuard(&self.recursion_depth as *const Cell<i32>);

        match &self.current_token.token {
            Token::Let | Token::Const => {
                let is_const = matches!(self.current_token.token, Token::Const);
                self.next_token();
                let name = match &self.current_token.token {
                    Token::Identifier(var_name) => var_name.clone(),
                    _ => return Err(self.unexpected(&self.current_token)),
                };

                self.next_token();
                if !matches!(self.current_token.token, Token::Assign) {
                    return Err(self.unexpected(&self.current_token));
                }

                self.next_token();
                let value = self.parse_expression(Lowest)?;
                self.next_token();

                if is_const {
                    Ok(Statement::Const { name, value })
                } else {
                    Ok(Statement::Let { name, value })
                }
            }
            Token::If => self.parse_if_statement(),
            Token::For => self.parse_for_statement(),
            Token::Fun if matches!(self.peek_token.token, Token::Identifier(_)) => {
                self.parse_function()
            },
            Token::Return => {
                self.next_token();
                let value = match self.parse_expression(Lowest) {
                    Ok(expr) =>  {
                        self.next_token();
                        expr
                    },
                    Err(e) => {
                        if matches!(self.current_token.token, Token::Newline | Token::EOF | Token::RightBrace) {
                            return Ok(Statement::Return {
                                value: None
                            })
                        }

                        return Err(e)
                    }
                };

                Ok(Statement::Return {
                    value: Some(value)
                })
            }
            Token::Struct => self.parse_struct(),
            _ => {
                let expr = self.parse_expression(Lowest)?;
                self.next_token();
                Ok(Statement::ExpressionStatement { expression: expr })
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

        while matches!(self.current_token.token, Token::Else) {
            self.expect(Token::Else)?;

            if matches!(self.current_token.token, Token::If) {
                let else_if_stmt = self.parse_else_if_statement()?;

                else_if.push(else_if_stmt);
            } else if matches!(self.current_token.token, Token::LeftBrace) {
                self.next_token();
                else_block = Some(self.parse_block()?);

                break;
            } else {
                return Err(self.unexpected(&self.current_token));
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

        Ok(ElseIf { condition, block })
    }

    pub fn parse_for_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::For)?;

        if let Token::Identifier(var) = self.current_token.token.clone() {
            if matches!(self.peek_token.token, Token::In) {
                self.expect(Token::Identifier("".to_string()))?;
                self.expect(Token::In)?;

                self.parse_for_range(&var)
            } else if matches!(self.peek_token.token, Token::Assign) {
                self.parse_for_counter(&var)
            } else {
                self.parse_for_condition()
            }
        } else {
            Err(self.unexpected(&self.current_token))
        }
    }

    pub fn parse_for_range(&mut self, var: &String) -> Result<Statement, ParseError> {
        let variable = var.clone();

        let iterable = self.parse_expression(Lowest)?;

        self.next_token();
        self.skip_newlines();

        if !matches!(self.current_token.token, Token::LeftBrace) {
            return Err(self.unexpected(&self.current_token));
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

        self.next_token();

        if !matches!(self.current_token.token, Token::Semicolon) {
            return Err(self.unexpected(&self.current_token));
        }

        self.expect(Token::Semicolon)?;

        let condition = self.parse_expression(Lowest)?;

        self.next_token();

        if !matches!(self.current_token.token, Token::Semicolon) {
            return Err(self.unexpected(&self.current_token));
        }

        self.expect(Token::Semicolon)?;

        let post = self.parse_expression(Lowest)?;

        self.next_token();
        self.skip_newlines();

        if !matches!(self.current_token.token, Token::LeftBrace) {
            return Err(self.unexpected(&self.current_token));
        }
        self.next_token();

        let body = self.parse_block()?;

        Ok(ForCounter {
            init,
            condition,
            post,
            body,
        })
    }

    pub fn parse_for_condition(&mut self) -> Result<Statement, ParseError> {
        let condition = self.parse_expression(Lowest)?;

        self.next_token();

        if !matches!(self.current_token.token, Token::LeftBrace) {
            return Err(self.unexpected(&self.current_token));
        }

        self.next_token();
        let body = self.parse_block()?;

        Ok(ForCondition { condition, body })
    }

    pub fn parse_function(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::Fun)?;

        let name = match self.current_token.token.clone() {
            Token::Identifier(fun_name) => fun_name.clone(),
            _ => return Err(self.unexpected(&self.current_token)),
        };

        self.next_token();
        self.expect(Token::LeftParen)?;
        let params = self.parse_fun_params()?;

        self.expect(Token::RightParen)?;

        let return_type = match self.current_token.token {
            Token::LeftBrace => None,
            _ => Some(self.parse_type()?),
        };

        self.expect(Token::LeftBrace)?;

        let body = self.parse_block()?;

        Ok(Statement::Fun {
            name,
            params,
            return_type,
            body,
        })
    }

    pub fn parse_struct(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::Struct)?;

        let name = match self.current_token.token.clone() {
            Token::Identifier(struct_name) => struct_name,
            _ => return Err(self.unexpected(&self.current_token)),
        };

        self.next_token();
        self.expect(Token::LeftBrace)?;
        self.skip_newlines();
        let mut fields = Vec::new();

        while !matches!(self.current_token.token, Token::RightBrace) {
            let field_name = match self.current_token.token.clone() {
                Token::Identifier(name) => name,
                _ => return Err(self.unexpected(&self.current_token)),
            };
            self.next_token();
            self.expect(Token::Colon)?;

            let field_type = self.parse_type()?;

            let field = StructParam {
                name: field_name,
                param_type: field_type,
            };

            fields.push(field);

            match self.current_token.token.clone() {
                Token::Comma => self.expect(Token::Comma),
                Token::Newline => {
                    self.skip_newlines();
                    continue;
                }
                Token::RightBrace => break,
                _ => return Err(self.unexpected(&self.current_token)),
            }?;
            self.skip_newlines();
        }

        Ok(Statement::Struct { name, fields })
    }

    pub fn parse_package(&mut self) -> Result<String, ParseError> {
        self.skip_newlines();
        self.expect(Token::Package)?;

        let name = match self.current_token.token.clone() {
            Token::Identifier(n) => n,
            _ => return Err(ParseError::UnexpectedToken {
                token: self.current_token.token.clone(),
                span: self.current_token.span
            })
        };

        Ok(name)
    }

    pub fn parse_imports(&mut self) -> Result<Vec<String>, ParseError> {
        self.expect(Token::Import)?;

        let mut packages = Vec::new();

        if matches!(self.current_token.token, Token::LeftParen) {
            self.expect(Token::LeftParen)?;
            self.skip_newlines();

            while !matches!(self.current_token.token, Token::RightParen) {
                let name = match self.current_token.token.clone() {
                    Token::Identifier(v) => v,
                    _ => return Err(self.unexpected(&self.current_token)),
                };

                self.next_token();

                packages.push(name);

                match self.current_token.token.clone() {
                    Token::Comma => self.expect(Token::Comma),
                    Token::Newline => { self.skip_newlines(); continue; }
                    Token::RightParen => break,
                    _ => return Err(self.unexpected(&self.current_token)),
                }?;

                self.skip_newlines();
            }
        } else {
            packages.push(match self.current_token.token.clone() {
                Token::Identifier(v) => v,
                _ => return Err(ParseError::UnexpectedToken {
                    token: self.current_token.token.clone(),
                    span: self.current_token.span
                })
            });
        }

        Ok(packages)
    }

    pub fn parse_block(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();

        self.skip_newlines();

        while !matches!(self.current_token.token, Token::RightBrace | Token::EOF) {
            let stmt = self.parse_statement()?;

            statements.push(stmt);
            self.skip_newlines();
        }

        if matches!(self.current_token.token, Token::RightBrace) {
            self.next_token();
        }

        Ok(statements)
    }
}
