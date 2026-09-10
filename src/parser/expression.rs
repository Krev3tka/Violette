use crate::lexer::span::Span;
use crate::lexer::token::Token;
use crate::parser::Precedence::Lowest;
use crate::parser::parser::{MAX_DEPTH, Parser};
use crate::parser::statement::{FuncParam, MatchArm};
use crate::parser::types::Type;
use crate::parser::{ParseError, Precedence, Statement};
use crate::typechecker::error::BindingKind;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Identifier {
        name: String,
        span: Span,
    },

    IntLiteral {
        val: isize,
        span: Span,
    },

    FloatLiteral {
        val: f64,
        span: Span,
    },

    BoolLiteral {
        val: bool,
        span: Span,
    },

    StringLiteral {
        val: String,
        span: Span,
    },

    StructLiteral {
        name: String,
        fields: Vec<StructLiteralField>,
        span: Span,
    },

    Prefix {
        operator: Token,
        right: Box<Expression>,
        span: Span,
    },

    Infix {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
        span: Span,
    },

    Postfix {
        left: Box<Expression>,
        operator: Token,
        span: Span,
    },

    Index {
        left: Box<Expression>,
        index: Box<Expression>,
        span: Span,
    },

    Call {
        function: Box<Expression>,
        args: Vec<Expression>,
        span: Span,
    },

    Block {
        body: Vec<Statement>,
        span: Span,
    },

    Match {
        target: Box<Expression>,
        arms: Vec<MatchArm>,
        span: Span,
    },

    Field {
        object: Box<Expression>,
        name: String,
        span: Span,
    },

    MethodCall {
        object: Box<Expression>,
        name: String,
        args: Vec<Expression>,
        span: Span,
    },

    Lambda {
        params: Vec<FuncParam>,
        return_type: Option<Type>,
        body: Vec<Statement>,
        span: Span,
    },

    Range {
        start: Option<Box<Expression>>,
        end: Option<Box<Expression>>,
        range_kind: RangeKind,
        span: Span,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructLiteralField {
    pub field_name: String,
    pub field_val: Box<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RangeKind {
    /// # Exclusive range
    /// **doesn't include right bound of the range**
    ///
    /// # Examples:
    /// # Examples:
    /// ```violette
    /// funс main() {
    ///     for i in 1:10 {
    ///         print(i, ", ")
    ///     }
    /// }
    /// ```
    /// ## Output:
    /// `1, 2, 3, 4, 5, 6, 7, 8, 9,`
    Exclusive,

    /// # Inclusive range
    /// **include right bound of the range**
    ///
    /// # Examples:
    /// ```violette
    /// funс main() {
    ///     for i in 1..10 {
    ///         print(i, ", ")
    ///     }
    /// }
    /// ```
    /// ## Output:
    /// `1, 2, 3, 4, 5, 6, 7, 8, 9, 10,`
    Inclusive,
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
        Token::Colon | Token::DoubleDot => Precedence::Range,
        Token::Add | Token::Subtract => Precedence::Sum,
        Token::Multiply | Token::Divide | Token::Modulus => Precedence::Product,
        Token::Power => Precedence::Power,
        Token::Increment
        | Token::Decrement
        | Token::LeftParen
        | Token::LeftBracket
        | Token::Pipe
        | Token::Dot => Precedence::Postfix,
        _ => Lowest,
    }
}

impl Parser {
    pub fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, ParseError> {
        self.depth += 1;

        if self.depth > MAX_DEPTH {
            self.depth -= 1;
            return Err(ParseError::TooDeep {
                span: self.current_token.span,
            });
        }
        let result = self.parse_expression_inner(precedence);
        self.depth -= 1;
        result
    }

    fn parse_expression_inner(&mut self, precedence: Precedence) -> Result<Expression, ParseError> {
        let mut start_span = self.current_token.span;

        let mut left = match &self.current_token.token {
            Token::Int(v) => Expression::IntLiteral {
                val: *v,
                span: start_span,
            },
            Token::Float32(v) => Expression::FloatLiteral {
                val: *v as f64,
                span: start_span,
            },
            Token::Float64(v) => Expression::FloatLiteral {
                val: *v,
                span: start_span,
            },
            Token::Identifier(s) => {
                if self.allowed_struct_literal && matches!(self.peek_token.token, Token::LeftBrace)
                {
                    self.parse_struct_literal(start_span)?
                } else {
                    Expression::Identifier {
                        name: s.clone(),
                        span: start_span,
                    }
                }
            }
            Token::Bool(b) => Expression::BoolLiteral {
                val: *b,
                span: start_span,
            },
            Token::String(s) => Expression::StringLiteral {
                val: s.clone(),
                span: start_span,
            },
            Token::LeftParen => {
                let open_token = self.current_token.token.clone();
                let open_span = self.current_token.span;

                self.paren_depth += 1;
                self.next_token();
                self.skip_terminators();

                match self.current_token.token.clone() {
                    Token::Equals
                    | Token::NotEquals
                    | Token::Less
                    | Token::Greater
                    | Token::LessOrEquals
                    | Token::GreaterOrEquals
                    | Token::Sprout
                    | Token::Add
                    | Token::Multiply
                    | Token::Divide
                    | Token::Modulus
                    | Token::Power
                    | Token::LogicAnd
                    | Token::LogicOr
                    | Token::BitAnd
                    | Token::BitOr
                    | Token::BitXOR
                    | Token::LeftShift
                    | Token::RightShift => {
                        let operator = self.current_token.token.clone();
                        self.next_token();

                        let right_part = self.parse_expression(Lowest)?;
                        self.skip_terminators();

                        if matches!(self.peek_token.token.clone(), Token::RightParen) {
                            self.next_token();
                            self.paren_depth -= 1;
                        } else {
                            return Err(ParseError::UnclosedDelimiter {
                                open_token: Box::new(open_token),
                                open_span,
                                expected_token: Box::new(Token::RightParen),
                                found_tok: Box::new(self.peek_token.token.clone()),
                                found_span: self.peek_token.span,
                            });
                        }

                        let arg_ident = Expression::Identifier {
                            name: String::from("$0"),
                            span: start_span,
                        };

                        let body_expr = Expression::Infix {
                            left: Box::new(arg_ident),
                            operator,
                            right: Box::new(right_part),
                            span: start_span,
                        };

                        return Ok(Expression::Lambda {
                            params: vec![FuncParam {
                                name: String::from("$0"),
                                param_type: Type::Infer,
                                span: start_span,
                                kind: BindingKind::Param,
                                is_ref: false,
                            }],
                            return_type: None,
                            body: vec![Statement::Return {
                                value: Some(body_expr),
                                span: start_span,
                            }],
                            span: start_span,
                        });
                    }
                    _ => {}
                };

                let expr = self.parse_expression(Lowest)?;
                self.skip_terminators();

                if matches!(self.peek_token.token, Token::RightParen) {
                    self.next_token();
                    self.paren_depth -= 1;
                    expr
                } else {
                    return Err(ParseError::UnclosedDelimiter {
                        open_token: Box::new(open_token),
                        open_span,
                        expected_token: Box::new(Token::RightParen),
                        found_tok: Box::new(self.peek_token.token.clone()),
                        found_span: self.peek_token.span,
                    });
                }
            }
            Token::Subtract
            | Token::LogicNot
            | Token::BitAnd
            | Token::BitNot
            | Token::Increment
            | Token::Decrement => {
                let operator = self.current_token.token.clone();
                self.next_token();
                let right = self.parse_expression(Precedence::Prefix)?;

                Expression::Prefix {
                    operator,
                    right: Box::new(right),
                    span: start_span,
                }
            }
            Token::Match => self.parse_match_expression()?,
            Token::Func => self.parse_lambda()?,
            _ => return Err(self.unexpected(&self.current_token)),
        };

        // self.skip_terminators();

        while {
            if self.paren_depth > 0 {
                while matches!(self.peek_token.token, Token::Newline) {
                    self.next_token();
                }
            }

            precedence < self.peek_precedence()
                || (precedence == self.peek_precedence()
                    && matches!(self.peek_token.token, Token::Assign | Token::Power))
        } {
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
                        left: Box::new(left.clone()),
                        operator,
                        right: Box::new(right.clone()),
                        span: left.span().merge(&right.span()),
                    };
                }
                Token::Decrement | Token::Increment | Token::Pipe => {
                    self.next_token();
                    let operator = self.current_token.token.clone();
                    start_span = self.current_token.span;

                    left = Expression::Postfix {
                        left: Box::new(left),
                        operator,
                        span: start_span,
                    }
                }
                Token::Dot => {
                    self.next_token();
                    left = self.parse_dot(left, start_span)?;
                }
                Token::LeftParen => {
                    self.next_token();
                    let args = self.parse_call_args()?;
                    left = Expression::Call {
                        function: Box::new(left),
                        args,
                        span: start_span,
                    };
                }
                Token::LeftBracket => {
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
                        span: start_span,
                    }
                }
                Token::LeftShift | Token::RightShift => {
                    let peek_prec = self.peek_precedence();
                    self.next_token();
                    let operator = self.current_token.token.clone();
                    self.next_token();
                    let right = self.parse_expression(peek_prec)?;

                    left = Expression::Infix {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                        span: start_span,
                    }
                }
                Token::Colon | Token::DoubleDot => {
                    self.next_token();
                    left = self.parse_infix_range(left)?
                }
                _ => break,
            }

            self.skip_terminators();
        }

        Ok(left)
    }

    fn parse_index_expression(&mut self, left: Expression) -> Result<Expression, ParseError> {
        let start_span = self.current_token.span;
        let open_token = self.current_token.token.clone();
        let open_span = self.current_token.span;

        self.next_token();

        let index = self.parse_expression(Lowest)?;

        if !matches!(self.peek_token.token, Token::RightBracket) {
            return Err(ParseError::UnclosedDelimiter {
                open_token: Box::new(open_token),
                open_span,
                expected_token: Box::new(Token::RightBracket),
                found_tok: Box::new(self.peek_token.token.clone()),
                found_span: self.peek_token.span,
            });
        }
        self.next_token();

        Ok(Expression::Index {
            left: Box::new(left),
            index: Box::new(index),
            span: start_span,
        })
    }

    pub fn parse_match_expression(&mut self) -> Result<Expression, ParseError> {
        let start_span = self.current_token.span;

        self.expect(Token::Match, self.unexpected(&self.current_token))?;

        let saved = self.allowed_struct_literal;
        self.allowed_struct_literal = false;
        let target = self.parse_expression(Lowest)?;
        self.allowed_struct_literal = saved;

        self.next_token();

        let open_brace_tok = self.current_token.token.clone();
        let open_brace_span = self.current_token.span;

        self.expect(Token::LeftBrace, self.unexpected(&self.current_token))?;
        self.skip_arm_separators();

        let mut arms = Vec::new();

        while !matches!(self.current_token.token, Token::RightBrace | Token::Eof) {
            let pattern = self.parse_pattern()?;
            self.next_token();

            self.expect(
                Token::FatArrow,
                ParseError::ExpectedFatArrow {
                    found: self.current_token.token.clone(),
                    span: self.current_token.span,
                },
            )?;

            let body = if matches!(self.current_token.token, Token::LeftBrace) {
                self.next_token();
                let (block_stmts, _) = self.parse_block()?;

                Expression::Block {
                    body: block_stmts,
                    span: start_span,
                }
            } else {
                let saved = self.allowed_struct_literal;
                self.allowed_struct_literal = false;

                let e = self.parse_expression(Lowest)?;

                self.allowed_struct_literal = saved;
                self.next_token();
                e
            };
            arms.push(MatchArm {
                pattern,
                body,
                span: start_span,
            });
            self.skip_arm_separators();
        }

        if !matches!(self.current_token.token, Token::RightBrace) {
            return Err(ParseError::UnclosedDelimiter {
                open_token: Box::new(open_brace_tok),
                open_span: open_brace_span,
                expected_token: Box::new(Token::RightBrace),
                found_tok: Box::new(self.current_token.token.clone()),
                found_span: self.current_token.span,
            });
        }

        Ok(Expression::Match {
            target: Box::new(target),
            arms,
            span: start_span,
        })
    }

    pub fn parse_dot(&mut self, left: Expression, span: Span) -> Result<Expression, ParseError> {
        self.expect(Token::Dot, self.unexpected(&self.current_token))?;
        self.skip_terminators();

        let name = match self.current_token.token.clone() {
            Token::Identifier(name) => name,
            _ => {
                return Err(ParseError::ExpectedIdentifier {
                    context: "after '.' in field access",
                    found: self.current_token.token.clone(),
                    span: self.current_token.span,
                });
            }
        };

        if matches!(self.peek_token.token, Token::LeftParen) {
            let span = self.current_token.span;
            self.next_token();
            let args = self.parse_call_args()?;
            Ok(Expression::MethodCall {
                object: Box::new(left),
                name,
                args,
                span,
            })
        } else {
            Ok(Expression::Field {
                object: Box::new(left),
                name,
                span: span.merge(&self.current_token.span),
            })
        }
    }

    pub fn parse_pattern(&mut self) -> Result<Expression, ParseError> {
        self.parse_expression(Lowest)
    }

    pub fn parse_func_params(&mut self) -> Result<Vec<FuncParam>, ParseError> {
        let mut params = Vec::new();

        while !matches!(self.current_token.token, Token::RightParen) {
            let kind = if matches!(self.current_token.token.clone(), Token::Var) {
                self.next_token();
                BindingKind::Var
            } else {
                BindingKind::Param
            };

            let start_span = self.current_token.span;
            let param_name = match self.current_token.token.clone() {
                Token::Identifier(name) => name,
                _ => {
                    return Err(ParseError::ExpectedIdentifier {
                        context: "in function parameter",
                        found: self.current_token.token.clone(),
                        span: self.current_token.span,
                    });
                }
            };

            self.next_token();
            self.expect(Token::Colon, self.unexpected(&self.current_token))?;

            let is_ref = match self.current_token.token.clone() {
                Token::BitAnd => {
                    self.next_token();
                    true
                }
                _ => false,
            };
            let param_type = self.parse_type()?;

            let param = FuncParam {
                name: param_name,
                param_type,
                span: start_span,
                kind,
                is_ref,
            };

            params.push(param);

            match self.current_token.token.clone() {
                Token::Comma => self.expect(Token::Comma, self.unexpected(&self.current_token)),
                Token::RightParen => break,
                _ => return Err(self.unexpected(&self.current_token)),
            }?;
        }

        Ok(params)
    }

    pub fn parse_call_args(&mut self) -> Result<Vec<Expression>, ParseError> {
        let open_paren_tok = self.current_token.token.clone();
        let open_paren_span = self.current_token.span;

        self.paren_depth += 1;
        self.next_token();
        self.skip_terminators();

        let mut args = Vec::new();

        if matches!(self.current_token.token.clone(), Token::RightParen) {
            self.paren_depth -= 1;
            return Ok(args);
        }

        loop {
            args.push(self.parse_expression(Lowest)?);
            self.skip_terminators();

            while self.paren_depth > 0 && matches!(self.peek_token.token, Token::Newline) {
                self.next_token();
            }

            if matches!(self.peek_token.token, Token::Comma) {
                self.next_token();
                self.next_token();

                if matches!(self.current_token.token, Token::RightParen) {
                    break;
                }
            } else if matches!(self.peek_token.token.clone(), Token::RightParen) {
                self.next_token();
                break;
            } else {
                return Err(ParseError::UnclosedDelimiter {
                    open_token: Box::new(open_paren_tok),
                    open_span: open_paren_span,
                    expected_token: Box::new(Token::RightParen),
                    found_tok: Box::new(self.peek_token.token.clone()),
                    found_span: self.peek_token.span,
                });
            }
        }

        self.paren_depth -= 1;
        Ok(args)
    }

    pub fn parse_lambda(&mut self) -> Result<Expression, ParseError> {
        let start_span = self.current_token.span;

        self.expect(Token::Func, self.unexpected(&self.current_token))?;

        let open_paren_tok = self.current_token.token.clone();
        let open_paren_span = self.current_token.span;

        self.expect(Token::LeftParen, self.unexpected(&self.current_token))?;
        let params = self.parse_func_params()?;
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

        let mut return_type = None;

        if matches!(self.current_token.token, Token::LeftBracket) {
            let open_bracket_tok = self.current_token.token.clone();
            let open_bracket_span = self.current_token.span;
            self.next_token();

            return_type = Some(self.parse_type()?);
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
        }

        self.expect(Token::LeftBrace, self.unexpected(&self.current_token))?;

        let (body, _) = self.parse_block()?;
        Ok(Expression::Lambda {
            params,
            return_type,
            body,
            span: start_span,
        })
    }

    pub fn parse_infix_range(&mut self, left: Expression) -> Result<Expression, ParseError> {
        let start_span = self.current_token.span;

        let range_kind = match self.current_token.token {
            Token::Colon => RangeKind::Exclusive,
            Token::DoubleDot => RangeKind::Inclusive,
            _ => unreachable!(),
        };
        self.next_token();

        let end = if matches!(
            self.peek_token.token,
            Token::RightBracket
                | Token::RightParen
                | Token::Comma
                | Token::Semicolon
                | Token::Newline
        ) {
            None
        } else {
            Some(Box::new(self.parse_expression(Precedence::Range)?))
        };

        Ok(Expression::Range {
            start: Some(Box::new(left.clone())),
            end,
            range_kind,
            span: start_span,
        })
    }

    pub fn parse_struct_literal(&mut self, start_span: Span) -> Result<Expression, ParseError> {
        let name = match &self.current_token.token {
            Token::Identifier(n) => n.clone(),
            _ => {
                return Err(ParseError::ExpectedIdentifier {
                    context: "in struct literal name",
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
                Token::Identifier(n) => n,
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

            let field_val = Box::new(self.parse_expression(Lowest)?);
            fields.push(StructLiteralField {
                field_name,
                field_val,
            });

            self.next_token();
            self.skip_terminators();

            match self.current_token.token.clone() {
                Token::Comma => self.expect(Token::Comma, self.unexpected(&self.current_token)),
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

        // self.skip_terminators();

        Ok(Expression::StructLiteral {
            name,
            fields,
            span: start_span,
        })
    }

    fn skip_arm_separators(&mut self) {
        while matches!(self.current_token.token, Token::Newline | Token::Comma) {
            self.next_token();
        }
    }
}

impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Expression::Identifier { span, .. } => *span,
            Expression::IntLiteral { span, .. } => *span,
            Expression::FloatLiteral { span, .. } => *span,
            Expression::BoolLiteral { span, .. } => *span,
            Expression::StringLiteral { span, .. } => *span,
            Expression::StructLiteral { span, .. } => *span,
            Expression::Prefix { span, .. } => *span,
            Expression::Infix { span, .. } => *span,
            Expression::Postfix { span, .. } => *span,
            Expression::Index { span, .. } => *span,
            Expression::Call { span, .. } => *span,
            Expression::Block { span, .. } => *span,
            Expression::Match { span, .. } => *span,
            Expression::Field { span, .. } => *span,
            Expression::MethodCall { span, .. } => *span,
            Expression::Lambda { span, .. } => *span,
            Expression::Range { span, .. } => *span,
        }
    }
}
