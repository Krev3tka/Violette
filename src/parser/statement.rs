use crate::lexer::span::Span;
use crate::lexer::token::Token;
use crate::parser::Precedence::Lowest;
use crate::parser::parser::{MAX_DEPTH, Parser};
use crate::parser::program::ImportItem;
use crate::parser::types::Type;
use crate::parser::{Expression, ParseError};

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Expression {
        expression: Expression,
        span: Span,
    },

    Var {
        name: String,
        value: Expression,
        span: Span,
    },

    Let {
        name: String,
        value: Expression,
        span: Span,
    },

    Const {
        name: String,
        value: Expression,
        span: Span,
    },

    If(IfStatement),

    While {
        condition: Expression,
        body: Vec<Statement>,
        span: Span,
    },

    ForRange {
        variable: String,
        iterable: Expression,
        body: Vec<Statement>,
        span: Span,
    },

    ForCounter {
        init: Box<Statement>,
        condition: Expression,
        post: Expression,
        body: Vec<Statement>,
        span: Span,
    },

    Break {
        span: Span,
    },

    Continue {
        span: Span,
    },

    Return {
        value: Option<Expression>,
        span: Span,
    },

    ExternFunc {
        name: String,
        params: Vec<FuncParam>,
        return_type: Option<Type>,
        span: Span,
    },

    Func {
        name: String,
        params: Vec<FuncParam>,
        return_type: Option<Type>,
        body: Vec<Statement>,
        span: Span,
        ending_span: Span,
    },

    Struct {
        name: String,
        fields: Vec<FuncParam>,
        span: Span,
    },

    Extend {
        target: Type,
        methods: Vec<Statement>,
        span: Span,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_block: Vec<Statement>,
    pub else_if: Vec<ElseIf>,
    pub else_block: Vec<Statement>,
    pub(crate) span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ElseIf {
    pub condition: Expression,
    pub block: Vec<Statement>,
    pub(crate) span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FuncParam {
    pub name: String,
    pub param_type: Type,
    pub(crate) span: Span,
}

pub type StructParam = FuncParam;

#[derive(Debug, PartialEq, Clone)]
pub struct MatchArm {
    pub pattern: Expression,
    pub body: Expression,
    pub(crate) span: Span,
}

impl Parser {
    pub fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        self.depth += 1;

        if self.depth > MAX_DEPTH {
            self.depth -= 1;
            return Err(ParseError::TooDeep {
                span: self.current_token.span,
            });
        }
        let result = self.parse_statement_inner();
        self.depth -= 1;
        result
    }

    fn parse_statement_inner(&mut self) -> Result<Statement, ParseError> {
        let mut span = self.current_token.span;
        match &self.current_token.token {
            Token::Var | Token::Let | Token::Const => {
                let kw_token = self.current_token.token.clone();
                self.next_token();
                let name = match &self.current_token.token {
                    Token::Identifier(var_name) => var_name.clone(),
                    _ => {
                        return Err(ParseError::ExpectedIdentifier {
                            context: match kw_token {
                                Token::Var => "after 'var'",
                                Token::Let => "after 'let'",
                                Token::Const => "after 'const'",
                                _ => unreachable!(),
                            },
                            found: self.current_token.token.clone(),
                            span: self.current_token.span,
                        });
                    }
                };

                span = self.current_token.span;

                self.next_token();
                self.expect(
                    Token::Assign,
                    ParseError::ExpectedAssign {
                        context: "after variable name",
                        found: self.current_token.token.clone(),
                        span: self.current_token.span,
                    },
                )?;

                let value = self.parse_expression(Lowest)?;
                self.next_token();

                match kw_token {
                    Token::Var => Ok(Statement::Var { name, value, span }),
                    Token::Let => Ok(Statement::Let { name, value, span }),
                    Token::Const => Ok(Statement::Const { name, value, span }),
                    _ => unreachable!(),
                }
            }
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_statement(),
            Token::For => self.parse_for_statement(),
            Token::Func if matches!(self.peek_token.token, Token::Identifier(_)) => {
                self.parse_function()
            }
            Token::Break => {
                self.next_token();
                Ok(Statement::Break { span })
            }
            Token::Continue => {
                self.next_token();
                Ok(Statement::Continue { span })
            }
            Token::Extern => self.parse_extern(),
            Token::Return => {
                self.next_token();
                let value = match self.parse_expression(Lowest) {
                    Ok(expr) => {
                        if !matches!(self.current_token.token, Token::RightBrace | Token::Eof) {
                            self.next_token();
                        }
                        expr
                    }
                    Err(e) => {
                        if matches!(
                            self.current_token.token,
                            Token::Newline | Token::Eof | Token::RightBrace
                        ) {
                            return Ok(Statement::Return { value: None, span });
                        }
                        return Err(e);
                    }
                };

                Ok(Statement::Return {
                    value: Some(value),
                    span,
                })
            }
            Token::Struct => self.parse_struct(),
            Token::Extend => self.parse_extend(),
            _ => {
                let expr = self.parse_expression(Lowest)?;
                self.next_token();
                Ok(Statement::Expression {
                    expression: expr,
                    span,
                })
            }
        }
    }

    pub fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        let span = self.current_token.span;
        self.expect(Token::If, self.unexpected(&self.current_token))?;

        let saved = self.allowed_struct_literal;
        self.allowed_struct_literal = false;
        let condition = self.parse_expression(Lowest)?;
        self.allowed_struct_literal = saved;

        self.next_token();

        self.expect(Token::LeftBrace, self.unexpected(&self.current_token))?;

        let (then_block, _) = self.parse_block()?;

        let mut else_if = Vec::new();
        let mut else_block = vec![];

        while matches!(self.current_token.token, Token::Else) {
            self.expect(Token::Else, self.unexpected(&self.current_token))?;

            if matches!(self.current_token.token, Token::If) {
                let else_if_stmt = self.parse_else_if_statement()?;
                else_if.push(else_if_stmt);
            } else if matches!(self.current_token.token, Token::LeftBrace) {
                self.next_token();
                (else_block, _) = self.parse_block()?;
                break;
            } else {
                return Err(ParseError::InvalidElseBranch {
                    found: self.current_token.token.clone(),
                    span: self.current_token.span,
                });
            }
        }

        Ok(Statement::If(IfStatement {
            condition,
            then_block,
            else_if,
            else_block,
            span,
        }))
    }

    pub fn parse_else_if_statement(&mut self) -> Result<ElseIf, ParseError> {
        let span = self.current_token.span;

        self.expect(Token::If, self.unexpected(&self.current_token))?;

        let saved = self.allowed_struct_literal;
        self.allowed_struct_literal = false;
        let condition = self.parse_expression(Lowest)?;
        self.allowed_struct_literal = saved;

        self.next_token();
        self.expect(Token::LeftBrace, self.unexpected(&self.current_token))?;

        let (block, _) = self.parse_block()?;

        Ok(ElseIf {
            condition,
            block,
            span,
        })
    }

    pub fn parse_for_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::For, self.unexpected(&self.current_token))?;

        if let Token::Identifier(var) = self.current_token.token.clone() {
            if matches!(self.peek_token.token, Token::In) {
                self.next_token();
                self.expect(Token::In, self.unexpected(&self.current_token))?;

                self.parse_for_range(&var)
            } else if matches!(self.peek_token.token, Token::Assign) {
                self.parse_for_counter(&var)
            } else {
                Err(ParseError::InvalidForLoopSyntax {
                    span: self.current_token.span,
                })
            }
        } else {
            Err(ParseError::InvalidForLoopSyntax {
                span: self.current_token.span,
            })
        }
    }

    pub fn parse_for_range(&mut self, var: &str) -> Result<Statement, ParseError> {
        let span = self.current_token.span;
        let variable = var.to_owned();

        let saved = self.allowed_struct_literal;
        self.allowed_struct_literal = false;
        let iterable = self.parse_expression(Lowest)?;
        self.allowed_struct_literal = saved;

        self.next_token();
        self.skip_terminators();

        self.expect(Token::LeftBrace, self.unexpected(&self.current_token))?;
        self.skip_terminators();

        let (body, _) = self.parse_block()?;

        Ok(Statement::ForRange {
            variable,
            iterable,
            body,
            span,
        })
    }

    pub fn parse_for_counter(&mut self, var: &str) -> Result<Statement, ParseError> {
        let span = self.current_token.span;
        let name = var.to_owned();
        self.next_token();

        self.expect(
            Token::Assign,
            ParseError::ExpectedAssign {
                context: "in 'for' counter initializer",
                found: self.current_token.token.clone(),
                span: self.current_token.span,
            },
        )?;

        let saved = self.allowed_struct_literal;
        self.allowed_struct_literal = false;
        let value = self.parse_expression(Lowest)?;
        self.allowed_struct_literal = saved;

        let init = Box::new(Statement::Let { name, value, span });

        self.next_token();
        self.expect(Token::Semicolon, self.unexpected(&self.current_token))?;

        let saved = self.allowed_struct_literal;
        self.allowed_struct_literal = false;
        let condition = self.parse_expression(Lowest)?;
        self.allowed_struct_literal = saved;

        self.next_token();
        self.expect(Token::Semicolon, self.unexpected(&self.current_token))?;

        let saved = self.allowed_struct_literal;
        self.allowed_struct_literal = false;
        let post = self.parse_expression(Lowest)?;
        self.allowed_struct_literal = saved;

        self.next_token();
        self.skip_terminators();

        self.expect(Token::LeftBrace, self.unexpected(&self.current_token))?;

        let (body, _) = self.parse_block()?;

        Ok(Statement::ForCounter {
            init,
            condition,
            post,
            body,
            span,
        })
    }

    pub fn parse_while_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::While, self.unexpected(&self.current_token))?;
        let span = self.current_token.span;

        let saved = self.allowed_struct_literal;
        self.allowed_struct_literal = false;
        let condition = self.parse_expression(Lowest)?;
        self.allowed_struct_literal = saved;

        self.next_token();
        self.expect(Token::LeftBrace, self.unexpected(&self.current_token))?;

        let (body, _) = self.parse_block()?;

        Ok(Statement::While {
            condition,
            body,
            span,
        })
    }

    pub fn parse_function(&mut self) -> Result<Statement, ParseError> {
        let mut span = self.current_token.span;

        self.expect(Token::Func, self.unexpected(&self.current_token))?;

        let name = match self.current_token.token.clone() {
            Token::Identifier(fun_name) => fun_name,
            _ => {
                return Err(ParseError::ExpectedIdentifier {
                    context: "after 'func'",
                    found: self.current_token.token.clone(),
                    span: self.current_token.span,
                });
            }
        };

        self.next_token();

        let open_paren_tok = self.current_token.token.clone();
        let open_paren_span = self.current_token.span;

        self.expect(Token::LeftParen, self.unexpected(&self.current_token))?;
        let params = self.parse_fun_params()?;

        span = span.merge(&self.current_token.span);
        self.expect(
            Token::RightParen,
            ParseError::UnclosedDelimiter {
                open_token: Box::new(open_paren_tok),
                open_span: open_paren_span,
                expected_token: Box::new(Token::RightParen),
                found_tok: Box::new(self.current_token.token.clone()),
                found_span: self.current_token.span,
            },
        )?;

        let return_type = match self.current_token.token {
            Token::LeftBrace => None,
            Token::LeftBracket => {
                let open_bracket_tok = self.current_token.token.clone();
                let open_bracket_span = self.current_token.span;
                self.next_token();
                let ty = self.parse_type()?;
                self.expect(
                    Token::RightBracket,
                    ParseError::UnclosedDelimiter {
                        open_token: Box::new(open_bracket_tok),
                        open_span: open_bracket_span,
                        expected_token: Box::new(Token::RightBracket),
                        found_tok: Box::new(self.current_token.token.clone()),
                        found_span: self.current_token.span,
                    },
                )?;
                Some(ty)
            }
            _ => return Err(self.unexpected(&self.current_token)),
        };

        self.expect(Token::LeftBrace, self.unexpected(&self.current_token))?;

        let (body, ending_span) = self.parse_block()?;
        let ending_span = ending_span.unwrap();

        Ok(Statement::Func {
            name,
            params,
            return_type,
            body,
            span,
            ending_span,
        })
    }

    pub fn parse_extern(&mut self) -> Result<Statement, ParseError> {
        let mut span = self.current_token.span;

        self.expect(Token::Extern, self.unexpected(&self.current_token))?;
        self.expect(Token::Func, self.unexpected(&self.current_token))?;

        let name = match self.current_token.token.clone() {
            Token::Identifier(fun_name) => fun_name,
            _ => {
                return Err(ParseError::ExpectedIdentifier {
                    context: "after 'extern func'",
                    found: self.current_token.token.clone(),
                    span: self.current_token.span,
                });
            }
        };

        self.next_token();

        let open_paren_tok = self.current_token.token.clone();
        let open_paren_span = self.current_token.span;

        self.expect(Token::LeftParen, self.unexpected(&self.current_token))?;
        let params = self.parse_fun_params()?;

        span = span.merge(&self.current_token.span);
        self.expect(
            Token::RightParen,
            ParseError::UnclosedDelimiter {
                open_token: Box::new(open_paren_tok),
                open_span: open_paren_span,
                expected_token: Box::new(Token::RightParen),
                found_tok: Box::new(self.current_token.token.clone()),
                found_span: self.current_token.span,
            },
        )?;

        let return_type = match self.current_token.token {
            Token::LeftBracket => {
                let open_bracket_tok = self.current_token.token.clone();
                let open_bracket_span = self.current_token.span;
                self.next_token();
                let ty = self.parse_type()?;
                self.expect(
                    Token::RightBracket,
                    ParseError::UnclosedDelimiter {
                        open_token: Box::new(open_bracket_tok),
                        open_span: open_bracket_span,
                        expected_token: Box::new(Token::RightBracket),
                        found_tok: Box::new(self.current_token.token.clone()),
                        found_span: self.current_token.span,
                    },
                )?;
                Some(ty)
            }
            _ => None,
        };

        Ok(Statement::ExternFunc {
            name,
            params,
            return_type,
            span,
        })
    }

    pub fn parse_struct(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::Struct, self.unexpected(&self.current_token))?;
        let span = self.current_token.span;

        let name = match self.current_token.token.clone() {
            Token::Identifier(struct_name) => struct_name,
            _ => {
                return Err(ParseError::ExpectedIdentifier {
                    context: "after 'struct'",
                    found: self.current_token.token.clone(),
                    span: self.current_token.span,
                });
            }
        };

        self.next_token();

        let open_brace_tok = self.current_token.token.clone();
        let open_brace_span = self.current_token.span;

        self.expect(Token::LeftBrace, self.unexpected(&self.current_token))?;
        self.skip_terminators();
        let mut fields = Vec::new();

        while !matches!(self.current_token.token, Token::RightBrace) {
            let field_name = match self.current_token.token.clone() {
                Token::Identifier(name) => name,
                _ => {
                    return Err(ParseError::ExpectedIdentifier {
                        context: "in struct field name",
                        found: self.current_token.token.clone(),
                        span: self.current_token.span,
                    });
                }
            };
            self.next_token();
            self.expect(Token::Colon, self.unexpected(&self.current_token))?;

            let field_type = self.parse_type()?;

            let field = StructParam {
                name: field_name,
                param_type: field_type,
                span,
            };

            fields.push(field);

            match self.current_token.token.clone() {
                Token::Comma => self.expect(Token::Comma, self.unexpected(&self.current_token)),
                Token::Newline => {
                    self.skip_terminators();
                    continue;
                }
                Token::RightBrace => break,
                _ => return Err(self.unexpected(&self.current_token)),
            }?;
            self.skip_terminators();
        }

        self.expect(
            Token::RightBrace,
            ParseError::UnclosedDelimiter {
                open_token: Box::new(open_brace_tok),
                open_span: open_brace_span,
                expected_token: Box::new(Token::RightBrace),
                found_tok: Box::new(self.current_token.token.clone()),
                found_span: self.current_token.span,
            },
        )?;

        Ok(Statement::Struct { name, fields, span })
    }

    pub fn parse_package(&mut self) -> Result<String, ParseError> {
        self.skip_terminators();
        self.expect(Token::Package, self.unexpected(&self.current_token))?;

        let name = match self.current_token.token.clone() {
            Token::Identifier(n) => n,
            Token::PrimitiveType(t) => format!("{:?}", t).to_lowercase(),
            _ => {
                return Err(ParseError::ExpectedIdentifier {
                    context: "after 'package'",
                    found: self.current_token.token.clone(),
                    span: self.current_token.span,
                });
            }
        };

        Ok(name)
    }

    pub fn parse_imports(&mut self) -> Result<Vec<ImportItem>, ParseError> {
        self.expect(Token::Import, self.unexpected(&self.current_token))?;
        let mut packages = Vec::new();

        if matches!(self.current_token.token, Token::LeftParen) {
            let open_paren_tok = self.current_token.token.clone();
            let open_paren_span = self.current_token.span;

            self.expect(Token::LeftParen, self.unexpected(&self.current_token))?;
            self.skip_terminators();

            while !matches!(self.current_token.token, Token::RightParen) {
                packages.push(self.parse_single_import_item()?);
                self.skip_terminators();

                match self.current_token.token.clone() {
                    Token::Comma => self.expect(Token::Comma, self.unexpected(&self.current_token)),
                    Token::Newline => {
                        self.skip_terminators();
                        continue;
                    }
                    Token::RightParen => break,
                    _ => return Err(self.unexpected(&self.current_token)),
                }?;

                self.skip_terminators();
            }
            self.expect(
                Token::RightParen,
                ParseError::UnclosedDelimiter {
                    open_token: Box::new(open_paren_tok),
                    open_span: open_paren_span,
                    expected_token: Box::new(Token::RightParen),
                    found_tok: Box::new(self.current_token.token.clone()),
                    found_span: self.current_token.span,
                },
            )?;
        } else {
            packages.push(self.parse_single_import_item()?);
            self.skip_terminators();
        }

        Ok(packages)
    }

    pub fn parse_extend(&mut self) -> Result<Statement, ParseError> {
        let start_span = self.current_token.span;

        self.expect(Token::Extend, self.unexpected(&self.current_token))?;

        let target = self.parse_type()?;
        self.skip_terminators();

        let end_span = self.current_token.span;
        let open_brace_tok = self.current_token.token.clone();
        let open_brace_span = self.current_token.span;

        self.expect(Token::LeftBrace, self.unexpected(&self.current_token))?;
        self.skip_terminators();

        let mut methods = Vec::new();

        while !matches!(self.current_token.token, Token::RightBrace | Token::Eof) {
            let method = self.parse_statement()?;

            match method {
                Statement::Func { .. } => {
                    methods.push(method);
                    self.skip_terminators();
                }
                _ => {
                    return Err(ParseError::InvalidSyntaxInExtend {
                        found: self.current_token.token.clone(),
                        span: self.current_token.span,
                    });
                }
            }
        }

        self.expect(
            Token::RightBrace,
            ParseError::UnclosedDelimiter {
                open_token: Box::new(open_brace_tok),
                open_span: open_brace_span,
                expected_token: Box::new(Token::RightBrace),
                found_tok: Box::new(self.current_token.token.clone()),
                found_span: self.current_token.span,
            },
        )?;

        Ok(Statement::Extend {
            target,
            methods,
            span: start_span.merge(&end_span),
        })
    }

    fn parse_statements(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();
        self.skip_terminators();

        while !matches!(self.current_token.token, Token::RightBrace | Token::Eof) {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.skip_terminators();
        }

        Ok(statements)
    }

    pub fn parse_block(&mut self) -> Result<(Vec<Statement>, Option<Span>), ParseError> {
        let saved = self.allowed_struct_literal;
        self.allowed_struct_literal = true;
        let statements = self.parse_statements()?;
        self.allowed_struct_literal = saved;

        if !matches!(self.current_token.token, Token::RightBrace) {
            return Err(ParseError::UnexpectedEof {
                expected: "closing brace '}'",
                span: self.current_token.span,
            });
        }
        let end_span = self.current_token.span;
        self.next_token();

        Ok((statements, Some(end_span)))
    }

    pub fn parse_top_level(&mut self) -> Result<Vec<Statement>, ParseError> {
        self.parse_statements()
    }

    fn parse_single_import_item(&mut self) -> Result<ImportItem, ParseError> {
        let span = self.current_token.span;

        let first_seg = match self.current_token.token.clone() {
            Token::Identifier(v) => v,
            Token::PrimitiveType(t) => format!("{:?}", t).to_lowercase(),
            _ => {
                return Err(ParseError::ExpectedIdentifier {
                    context: "in import path",
                    found: self.current_token.token.clone(),
                    span: self.current_token.span,
                });
            }
        };
        self.next_token();

        let mut module_path = first_seg;
        let mut symbols = Vec::new();

        while matches!(self.current_token.token, Token::Dot)
            && matches!(
                self.peek_token.token,
                Token::Identifier(_) | Token::PrimitiveType(_)
            )
        {
            self.expect(Token::Dot, self.unexpected(&self.current_token))?;

            let sub_name = match self.current_token.token.clone() {
                Token::Identifier(v) => v,
                Token::PrimitiveType(t) => format!("{:?}", t).to_lowercase(),
                _ => {
                    return Err(ParseError::ExpectedIdentifier {
                        context: "after '.' in import path",
                        found: self.current_token.token.clone(),
                        span: self.current_token.span,
                    });
                }
            };
            self.next_token();

            module_path.push('.');
            module_path.push_str(&sub_name);
        }

        if matches!(self.current_token.token, Token::Dot) {
            self.expect(Token::Dot, self.unexpected(&self.current_token))?;

            let open_brace_tok = self.current_token.token.clone();
            let open_brace_span = self.current_token.span;

            self.expect(Token::LeftBrace, self.unexpected(&self.current_token))?;
            self.skip_terminators();

            while !matches!(self.current_token.token, Token::RightBrace) {
                let sym = match self.current_token.token.clone() {
                    Token::Identifier(v) => v,
                    _ => {
                        return Err(ParseError::ExpectedIdentifier {
                            context: "in import symbol list",
                            found: self.current_token.token.clone(),
                            span: self.current_token.span,
                        });
                    }
                };
                self.next_token();
                symbols.push(sym);
                self.skip_terminators();

                match self.current_token.token.clone() {
                    Token::Comma => {
                        self.expect(Token::Comma, self.unexpected(&self.current_token))?;
                        self.skip_terminators();
                    }
                    Token::RightBrace => break,
                    _ => return Err(self.unexpected(&self.current_token)),
                }
            }
            self.expect(
                Token::RightBrace,
                ParseError::UnclosedDelimiter {
                    open_token: Box::new(open_brace_tok),
                    open_span: open_brace_span,
                    expected_token: Box::new(Token::RightBrace),
                    found_tok: Box::new(self.current_token.token.clone()),
                    found_span: self.current_token.span,
                },
            )?;
        }

        Ok(ImportItem {
            module: module_path,
            symbols,
            span,
        })
    }
}

impl Statement {
    pub fn span(&self) -> Span {
        match self {
            Statement::Expression { span, .. } => *span,
            Statement::Var { span, .. } => *span,
            Statement::Let { span, .. } => *span,
            Statement::Const { span, .. } => *span,
            Statement::If(IfStatement { span, .. }) => *span,
            Statement::While { span, .. } => *span,
            Statement::ForRange { span, .. } => *span,
            Statement::ForCounter { span, .. } => *span,
            Statement::Return { span, .. } => *span,
            Statement::ExternFunc { span, .. } => *span,
            Statement::Func { span, .. } => *span,
            Statement::Struct { span, .. } => *span,
            Statement::Extend { span, .. } => *span,
            Statement::Break { span } => *span,
            Statement::Continue { span } => *span,
        }
    }
}
