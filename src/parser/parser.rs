use crate::lexer::lexer::Lexer;
use crate::lexer::span::SpannedToken;
use crate::lexer::token::{PrimitiveType, Token};
use crate::parser::expression::token_precedence;
use crate::parser::types::{ParseError, Type, TypePath};
use crate::parser::Precedence;

pub const MAX_DEPTH: u32 = 64;

macro_rules! primitive {
    ($self:expr, $variant:expr) => {{
        $self.next_token();
        Ok(Type::Primitive($variant))
    }};
}

macro_rules! trace_before {
    ($self:expr, $name:expr) => {{
        let indent = " ".repeat($self.recursion_depth.get() as usize);
        println!("{indent}-> {} (current: {:?})", $name, $self.current_token)
    }};
}

pub struct Parser {
    lexer: Lexer,
    pub current_token: SpannedToken,
    pub peek_token: SpannedToken,
    pub depth: u32,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let curr_spanned_token = lexer.next_token();
        let peek_spanned_token = lexer.next_token();

        Parser {
            lexer,
            current_token: curr_spanned_token,
            peek_token: peek_spanned_token,
            depth: 0,
        }
    }

    pub fn unexpected(&self, tok: &SpannedToken) -> ParseError {
        if tok.token == Token::Eof {
            ParseError::UnexpectedEof
        } else {
            ParseError::UnexpectedToken {
                token: tok.token.clone(),
                span: tok.span,
            }
        }
    }

    pub fn parse_type(&mut self) -> Result<Type, ParseError> {
        self.depth += 1;

        if self.depth > MAX_DEPTH {
            self.depth -= 1;
            return Err(ParseError::TooDeep {
                span: self.current_token.span,
            });
        }
        let result = self.parse_type_inner();
        self.depth -= 1;
        result
    }

    fn parse_type_inner(&mut self) -> Result<Type, ParseError> {
        if matches!(self.current_token.token, Token::Fun) {
            self.next_token();
            self.expect(Token::LeftParen)?;

            let mut types = Vec::new();

            if !matches!(self.current_token.token, Token::RightParen) {
                loop {
                    types.push(self.parse_type()?);
                    if matches!(self.current_token.token, Token::Comma) {
                        self.next_token();
                    } else {
                        break;
                    }
                }
            }
            self.expect(Token::RightParen)?;

            let mut ret = None;

            if matches!(self.current_token.token, Token::LeftBracket) {
                self.next_token();

                ret = Some(Box::new(self.parse_type()?));
                self.expect(Token::RightBracket)?;
            }

            return Ok(Type::Fn { params: types, ret });
        }

        if matches!(self.current_token.token, Token::LeftBracket) {
            self.next_token();

            let mut variants = Vec::new();

            variants.push(self.parse_single_type()?);
            self.skip_terminators();

            while matches!(self.current_token.token, Token::Pipe) {
                self.next_token();
                variants.push(self.parse_single_type()?);
                self.skip_terminators();
            }

            self.expect(Token::RightBracket)?;

            if variants.len() == 1 {
                return Ok(variants.remove(0));
            } else {
                return Ok(Type::Union(variants));
            }
        }

        self.skip_terminators();

        let first = self.parse_single_type()?;

        if !matches!(self.current_token.token, Token::Pipe) {
            return Ok(first);
        }

        let mut variants = vec![first];

        while matches!(self.current_token.token, Token::Pipe) {
            self.next_token();
            variants.push(self.parse_single_type()?);
        }

        Ok(Type::Union(variants))
    }

    pub fn parse_single_type(&mut self) -> Result<Type, ParseError> {
        self.depth += 1;

        if self.depth > MAX_DEPTH {
            self.depth -= 1;
            return Err(ParseError::TooDeep {
                span: self.current_token.span,
            });
        }
        let result = self.parse_single_type_inner();
        self.depth -= 1;
        result
    }

    fn parse_single_type_inner(&mut self) -> Result<Type, ParseError> {
        match self.current_token.token.clone() {
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

            Token::PrimitiveType(PrimitiveType::Float32) => {
                primitive!(self, PrimitiveType::Float32)
            }
            Token::PrimitiveType(PrimitiveType::Float64) => {
                primitive!(self, PrimitiveType::Float64)
            }

            Token::PrimitiveType(PrimitiveType::Bool) => primitive!(self, PrimitiveType::Bool),
            Token::PrimitiveType(PrimitiveType::String) => primitive!(self, PrimitiveType::String),

            Token::Identifier(name) => {
                let mut segments = vec![name.clone()];
                self.next_token();

                while self.current_token.token == Token::Dot {
                    self.next_token();

                    match self.current_token.token.clone() {
                        Token::Identifier(subname) => segments.push(subname),
                        _ => return Err(self.unexpected(&self.current_token)),
                    }

                    self.next_token()
                }

                if matches!(self.current_token.token, Token::LeftParen) {
                    self.expect(Token::LeftParen)?;
                    let param = self.parse_single_type()?;
                    self.expect(Token::RightParen)?;

                    Ok(Type::Generic {
                        name: segments.last().unwrap().clone(),
                        param: Box::new(param),
                    })
                } else {
                    Ok(Type::Named(TypePath { segments }))
                }
            }

            _ => Err(self.unexpected(&self.current_token)),
        }
    }

    pub fn next_token(&mut self) {
        self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token())
    }

    pub fn skip_terminators(&mut self) {
        while matches!(self.current_token.token, Token::Newline | Token::Semicolon) {
            self.next_token();
        }
    }

    pub fn current_precedence(&self) -> Precedence {
        token_precedence(&self.current_token.token)
    }

    pub fn peek_precedence(&self) -> Precedence {
        token_precedence(&self.peek_token.token)
    }

    pub fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if std::mem::discriminant(&self.current_token.token) == std::mem::discriminant(&expected) {
            self.next_token();
            Ok(())
        } else {
            Err(ParseError::Expected {
                expected,
                found: self.current_token.token.clone(),
                span: self.current_token.span,
            })
        }
    }
}
