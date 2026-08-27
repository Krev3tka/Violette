use crate::lexer::token::Token;
use crate::parser::statement::{FunParam, MatchArm};
use crate::parser::{Expression, Statement};

#[derive(Debug, Eq, PartialEq, Clone, Copy, Ord, PartialOrd, Default)]
pub struct Position {
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number.
    pub col: usize,
}

impl Position {
    pub fn new(line: usize, col: usize) -> Self {
        Position { line, col }
    }
}

/// Source code location represented by line and column numbers.
///
/// Used for diagnostic reporting and error tracking.
#[derive(Debug, Eq, PartialEq, Clone, Copy, Default)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Span { start, end }
    }

    #[allow(dead_code)]
    pub fn merge(&self, with: &Span) -> Span {
        Self {
            start: self.start.min(with.start),
            end: self.end.max(with.end),
        }
    }
}

/// Token paired with its source code location.
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

#[allow(dead_code)]
pub trait ClearSpan {
    fn clear_span(&mut self);
}

impl ClearSpan for Expression {
    fn clear_span(&mut self) {
        match self {
            Expression::Identifier { span, .. }
            | Expression::IntLiteral { span, .. }
            | Expression::FloatLiteral { span, .. }
            | Expression::BoolLiteral { span, .. }
            | Expression::StringLiteral { span, .. } => *span = Span::default(),

            Expression::Infix {
                left, right, span, ..
            } => {
                left.clear_span();
                right.clear_span();
                *span = Span::default();
            }

            Expression::Prefix { right, span, .. } => {
                right.clear_span();
                *span = Span::default();
            }

            Expression::Postfix { left, span, .. } => {
                left.clear_span();
                *span = Span::default();
            }

            Expression::Index {
                left, index, span, ..
            } => {
                left.clear_span();
                index.clear_span();
                *span = Span::default();
            }

            Expression::Call {
                function,
                args,
                span,
                ..
            } => {
                function.clear_span();
                for arg in args {
                    arg.clear_span();
                }
                *span = Span::default();
            }

            Expression::Match {
                target, arms, span, ..
            } => {
                target.clear_span();
                for arm in arms {
                    arm.clear_span();
                }
                *span = Span::default();
            }

            Expression::Block { body, span, .. } => {
                for stmt in body {
                    stmt.clear_span();
                }
                *span = Span::default();
            }

            Expression::StructLiteral { fields, span, .. } => {
                for f in fields {
                    f.field_val.clear_span();
                }
                *span = Span::default();
            }

            Expression::Field { object, span, .. } => {
                object.clear_span();
                *span = Span::default();
            }

            Expression::MethodCall {
                object, args, span, ..
            } => {
                object.clear_span();
                for arg in args {
                    arg.clear_span();
                }
                *span = Span::default();
            }

            Expression::Lambda { body, span, .. } => {
                for stmt in body {
                    stmt.clear_span();
                }
                *span = Span::default();
            }

            Expression::Range {
                start, end, span, ..
            } => {
                if let Some(s) = start {
                    s.clear_span();
                }
                if let Some(e) = end {
                    e.clear_span();
                }
                *span = Span::default();
            }
        }
    }
}

impl ClearSpan for Statement {
    fn clear_span(&mut self) {
        match self {
            Statement::Expression { expression, span } => {
                expression.clear_span();
                *span = Span::default();
            }
            Statement::Var { value, span, .. }
            | Statement::Let { value, span, .. }
            | Statement::Const { value, span, .. } => {
                value.clear_span();
                *span = Span::default();
            }
            Statement::If(if_stmt) => {
                if_stmt.condition.clear_span();
                for stmt in &mut if_stmt.then_block {
                    stmt.clear_span();
                }
                for else_if in &mut if_stmt.else_if {
                    else_if.condition.clear_span();
                    for stmt in &mut else_if.block {
                        stmt.clear_span();
                    }
                    else_if.span = Span::default();
                }

                for stmt in &mut if_stmt.else_block {
                    stmt.clear_span();
                }
                if_stmt.span = Span::default();
            }
            Statement::ForCondition {
                condition,
                body,
                span,
                ..
            } => {
                condition.clear_span();
                for stmt in body {
                    stmt.clear_span();
                }
                *span = Span::default();
            }
            Statement::ForCounter {
                init,
                condition,
                post,
                body,
                span,
                ..
            } => {
                init.clear_span();
                condition.clear_span();
                post.clear_span();
                for stmt in body {
                    stmt.clear_span();
                }
                *span = Span::default();
            }
            Statement::ForRange {
                iterable,
                body,
                span,
                ..
            } => {
                iterable.clear_span();
                for stmt in body {
                    stmt.clear_span();
                }
                *span = Span::default();
            }
            Statement::Break {
                span
            } => {
                *span = Span::default()
            },
            Statement::Continue {
                span
            } => {
                *span = Span::default()
            },
            Statement::Return { value, span } => {
                if let Some(val) = value {
                    val.clear_span();
                }
                *span = Span::default();
            }
            Statement::ExternFun { params, span, ..} => {
                for param in params {
                    param.clear_span();
                }

                *span = Span::default();
            }
            Statement::Fun {
                params, body, span, ..
            } => {
                for param in params {
                    param.clear_span();
                }
                for stmt in body {
                    stmt.clear_span();
                }
                *span = Span::default();
            }
            Statement::Struct { fields, span, .. } => {
                for field in fields {
                    field.clear_span();
                }
                *span = Span::default();
            }
        }
    }
}

impl ClearSpan for MatchArm {
    fn clear_span(&mut self) {
        self.pattern.clear_span();
        self.body.clear_span();
        self.span = Span::default();
    }
}

impl ClearSpan for FunParam {
    fn clear_span(&mut self) {
        self.span = Span::default();
    }
}
