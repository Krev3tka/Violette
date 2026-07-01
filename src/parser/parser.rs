use crate::lexer::lexer::Lexer;
use crate::lexer::token;
use crate::lexer::token::{Token, PrimitiveType};
use crate::parser::{Expression, Precedence, Statement};
use crate::parser::expression::token_precedence;
use crate::parser::Precedence::{Equals, LessGreater, Lowest, Prefix, Product, Sum};
use crate::parser::statement::{ElseIf, FunParam, IfStatement};
use crate::parser::Statement::{ForCondition, ForCounter};
use crate::parser::types::{ParseError, Type, TypePath};

macro_rules! primitive {
    ($self:expr, $variant:expr) => {{
        $self.next_token();
        Ok(Type::Primitive($variant))
    }};
}

pub struct Parser {
    lexer: Lexer,
    pub current_token: Token,
    pub peek_token: Token,
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

    pub fn parse_type(&mut self) -> Result<Type, ParseError> {
        if matches!(self.current_token, Token::LSB) {
            self.next_token();

            let mut variants = Vec::new();

            variants.push(self.parse_single_type()?);

            while matches!(self.current_token, Token::Pipe) {
                self.next_token();
                variants.push(self.parse_single_type()?);
            }

            self.expect(Token::RSB)?;

            return Ok(Type::Union(variants));
        }

        let first = self.parse_single_type()?;

        if !matches!(self.current_token, Token::Pipe) {
            return Ok(first)
        }

        let mut variants = vec![first];

        while matches!(self.current_token, Token::Pipe) {
            self.next_token();
            variants.push(self.parse_single_type()?);
        }

        Ok(Type::Union(variants))
    }

    pub fn parse_single_type(&mut self) -> Result<Type, ParseError> {
        match self.current_token.clone() {
            Token::PrimitiveType(PrimitiveType::Int) => primitive!(self, PrimitiveType::Int),
            Token::PrimitiveType(PrimitiveType::Int8) => primitive!(self, PrimitiveType::Int8),
            Token::PrimitiveType(PrimitiveType::Int16) => primitive!(self, PrimitiveType::Int16),
            Token::PrimitiveType(PrimitiveType::Int32) => primitive!(self, PrimitiveType::Int32),
            Token::PrimitiveType(PrimitiveType::Int64) => primitive!(self, PrimitiveType::Int64),

            Token::PrimitiveType(PrimitiveType::Uint) => primitive!(self, PrimitiveType::Uint),
            Token::PrimitiveType(PrimitiveType::Uint8) => primitive!(self, PrimitiveType::Uint8),
            Token::PrimitiveType(PrimitiveType::Uint16) => primitive!(self, PrimitiveType::Uint16),
            Token::PrimitiveType(PrimitiveType::Uint32) => primitive!(self, PrimitiveType::Uint32),
            Token::PrimitiveType(PrimitiveType::Uint64) => primitive!(self, PrimitiveType::Uint64),

            Token::PrimitiveType(PrimitiveType::Float32) => primitive!(self, PrimitiveType::Float32),
            Token::PrimitiveType(PrimitiveType::Float64) => primitive!(self, PrimitiveType::Float64),

            Token::PrimitiveType(PrimitiveType::Bool) => primitive!(self, PrimitiveType::Bool),
            Token::PrimitiveType(PrimitiveType::String) => primitive!(self, PrimitiveType::String),

            Token::Identifier(name) => {
                let mut segments = vec![name.clone()];
                self.next_token();

                while self.current_token == Token::Dot {
                    self.next_token();

                    match self.current_token.clone() {
                        Token::Identifier(subname) => segments.push(subname),
                        _ => return Err(ParseError::UnexpectedToken(self.current_token.clone()))
                    }

                    self.next_token()
                }

                if matches!(self.current_token, Token::LeftParen) {
                    self.expect(Token::LeftParen)?;
                    let param = self.parse_single_type()?;
                    self.expect(Token::RightParen)?;

                    Ok(Type::Generic { name, param: Box::new(param) } )
                } else {
                    Ok(Type::Named(TypePath {
                        segments
                    }))
                }
            }

            _ => Err(ParseError::UnexpectedToken(self.current_token.clone()))
        }
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

    pub fn current_precedence(&self) -> Precedence {
        token_precedence(&self.current_token)
    }

    pub fn peek_precedence(&self) -> Precedence {
        token_precedence(&self.peek_token)
    }

    pub fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {
            self.next_token();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(self.current_token.clone()))
        }
    }
}