use crate::lexer::token::Token;
use crate::parser::{Expression, ParseError};
use crate::parser::parser::Parser;
use crate::parser::Precedence::Lowest;
use crate::parser::Statement::{ForCondition, ForCounter};
use crate::parser::types::Type;

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
        value: Expression
    },

    Fun {
        name: String,
        params: Vec<FunParam>,
        return_type: Type,
        body: Vec<Statement>
    }

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

impl Parser {
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
            Token::Fun => self.parse_function(),
            Token::Return => {
                self.next_token();
                let value = self.parse_expression(Lowest)?;
                Ok(Statement::Return { value })
            }
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

        self.next_token();

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
        self.next_token();
        self.skip_newlines();

        if !matches!(self.current_token, Token::LeftBrace) {
            return Err(ParseError::UnexpectedToken(self.current_token.clone()))
        }
        self.next_token();

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

    pub fn parse_function(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::Fun)?;

        let name = match self.current_token.clone() {
            Token::Identifier(fun_name) => fun_name.clone(),
            _ => return Err(ParseError::UnexpectedToken(self.current_token.clone()))
        };

        self.next_token();
        self.expect(Token::LeftParen)?;

        let mut params = Vec::new();

        while !matches!(self.current_token, Token::RightParen) {
            let param_name = match self.current_token.clone() {
                Token::Identifier(name) => name,
                _ => return Err(ParseError::UnexpectedToken(self.current_token.clone()))
            };

            self.next_token();
            self.expect(Token::Colon)?;

            let param_type = self.parse_type()?;

            let param = FunParam {
                name: param_name,
                param_type
            };

            params.push(param);

            match self.current_token.clone() {
                Token::Comma => self.expect(Token::Comma),
                Token::RightParen => break,
                _ => return Err(ParseError::UnexpectedToken(self.current_token.clone()))
            }?;
        }

        self.expect(Token::RightParen)?;
        let return_type = self.parse_type()?;
        self.expect(Token::LeftBrace)?;
        let body = self.parse_block()?;
        println!("fun, current token: {:?}, return type {:?}", self.current_token.clone(), return_type);


        Ok(Statement::Fun {
            name,
            params,
            return_type,
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
}