#![allow(dead_code)]
use crate::lexer::span::Span;
use crate::lexer::token::Token;
use crate::parser::expression::{RangeKind, StructLiteralField};
use crate::parser::statement::{ElseIf, FuncParam, MatchArm};
use crate::parser::{Expression, Statement};
use crate::typechecker::types::Ty;

pub struct TypedExpr {
    expr: Expression,
}

pub enum TypedExpression {
    Identifier {
        name: String,
        span: Span,
    },

    IntLiteral {
        val: isize,
        ty: Ty,
        span: Span,
    },

    FloatLiteral {
        val: f64,
        ty: Ty,
        span: Span,
    },

    BoolLiteral {
        val: bool,
        ty: Ty,
        span: Span,
    },

    CharLiteral {
        val: char,
        ty: Ty,
        span: Span,
    },

    StringLiteral {
        val: String,
        ty: Ty,
        span: Span,
    },

    StructLiteral {
        name: String,
        ty: Ty,
        fields: Vec<StructLiteralField>,
        fields_ty: Vec<Ty>,
        span: Span,
    },

    Prefix {
        operator: Token,
        right: Box<Expression>,
        ty: Ty,
        span: Span,
    },

    Infix {
        left: Box<Expression>,
        left_ty: Ty,
        operator: Token,
        right: Box<Expression>,
        right_ty: Ty,
        span: Span,
    },

    Postfix {
        left: Box<Expression>,
        ty: Ty,
        operator: Token,
        span: Span,
    },

    Index {
        left: Box<Expression>,
        ty: Ty,
        index: Box<Expression>,
        span: Span,
    },

    Call {
        function: Box<Expression>,
        ty: Ty,
        args: Vec<Expression>,
        args_ty: Vec<Ty>,
        span: Span,
    },

    Block {
        body: Vec<Statement>,
        span: Span,
    },

    Match {
        target: Box<Expression>,
        target_ty: Ty,
        arms: Vec<MatchArm>,
        arms_ty: Vec<Ty>,
        span: Span,
    },

    Field {
        object: Box<Expression>,
        obj_ty: Ty,
        name: String,
        field_ty: Ty,
        span: Span,
    },

    MethodCall {
        object: Box<Expression>,
        obj_ty: Ty,
        name: String,
        method_ty: Ty,
        args: Vec<Expression>,
        args_ty: Ty,
        span: Span,
    },

    Lambda {
        params: Vec<FuncParam>,
        ty: Ty,
        return_ty: Option<Ty>,
        body: Vec<Statement>,
        span: Span,
    },

    Range {
        start: Option<Box<Expression>>,
        end: Option<Box<Expression>>,
        range_kind: RangeKind,
        span: Span,
    },

    If {
        condition: Box<Expression>,
        then_block: Vec<Statement>,
        else_if: Vec<ElseIf>,
        else_block: Vec<Statement>,
        span: Span,
    },
}
