#![allow(dead_code)]
use crate::lexer::span::Span;
use crate::lexer::token::Token;
use crate::parser::expression::RangeKind;
use crate::parser::statement::{FuncParam, StructParam, VariantCase};
use crate::typechecker::error::TypeError;
use crate::typechecker::types::Ty;

#[derive(Clone, Debug, PartialEq)]
pub struct TypedProgram {
    pub package: String,
    pub declarations: Vec<TypedStmt>,
    pub main: Vec<TypedStmt>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypedExpr {
    pub kind: TypedExprKind,
    pub ty: Ty,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypedStmt {
    pub kind: TypedStmtKind,
    pub ty: Ty,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypedExprKind {
    Identifier {
        name: String,
        is_ref: bool,
    },

    IntLiteral {
        val: isize,
    },

    FloatLiteral {
        val: f64,
    },

    BoolLiteral {
        val: bool,
    },

    CharLiteral {
        val: char,
    },

    StringLiteral {
        val: String,
    },

    StructLiteral {
        name: String,
        fields: Vec<TypedStructLiteralField>,
    },

    ArrayLiteral {
        values: Vec<TypedExpr>,
    },

    Prefix {
        operator: Token,
        right: Box<TypedExpr>,
    },

    Infix {
        left: Box<TypedExpr>,
        operator: Token,
        right: Box<TypedExpr>,
    },

    Postfix {
        left: Box<TypedExpr>,
        operator: Token,
    },

    Index {
        left: Box<TypedExpr>,
        index: Box<TypedExpr>,
    },

    Call {
        function: Box<TypedExpr>,
        args: Vec<TypedExpr>,
    },

    Block {
        body: Vec<TypedStmt>,
    },

    Match {
        target: Box<TypedExpr>,
        arms: Vec<TypedMatchArm>,
    },

    Field {
        object: Box<TypedExpr>,
        name: String,
    },

    MethodCall {
        object: Box<TypedExpr>,
        name: String,
        args: Vec<TypedExpr>,
    },

    Lambda {
        params: Vec<FuncParam>,
        return_ty: Option<Ty>,
        body: Vec<TypedStmt>,
    },

    Range {
        start: Option<Box<TypedExpr>>,
        end: Option<Box<TypedExpr>>,
        range_kind: RangeKind,
    },

    If {
        condition: Box<TypedExpr>,
        then_block: Vec<TypedStmt>,
        else_if: Vec<TypedElseIf>,
        else_block: Vec<TypedStmt>,
        closing_span: Option<Span>,
    },

    Error {
        error: TypeError,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypedStmtKind {
    Expression(TypedExpr),

    Var {
        name: String,
        ty: Ty,
        value: TypedExpr,
    },

    Let {
        name: String,
        ty: Ty,
        value: TypedExpr,
    },

    Const {
        name: String,
        ty: Ty,
        value: TypedExpr,
    },

    While {
        condition: TypedExpr,
        body: Vec<TypedStmt>,
    },

    ForRange {
        variable: String,
        iterable: TypedExpr,
        body: Vec<TypedStmt>,
    },

    Break,

    Continue,

    Return {
        value: Option<TypedExpr>,
    },

    ExternFunc {
        name: String,
        params: Vec<FuncParam>,
        return_type: Ty,
        span: Span,
    },

    Defer {
        callee: Box<TypedStmt>,
        span: Span,
    },

    Func {
        name: String,
        params: Vec<FuncParam>,
        return_type: Ty,
        body: Vec<TypedStmt>,
    },

    Struct {
        name: String,
        fields: Vec<StructParam>,
    },

    Extend {
        target: Ty,
        methods: Vec<TypedStmt>,
    },

    Variant {
        name: String,
        cases: Vec<VariantCase>,
    },

    Error {
        error: TypeError,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypedMatchArm {
    pub pattern: TypedExpr,
    pub body: TypedExpr,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypedElseIf {
    pub condition: TypedExpr,
    pub block: Vec<TypedStmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypedStructLiteralField {
    pub field_name: String,
    pub field_val: Box<TypedExpr>,
}

impl TypedExpr {
    pub fn error(span: Span, error: TypeError) -> Self {
        Self {
            kind: TypedExprKind::Error { error },
            ty: Ty::Error,
            span,
        }
    }
}

impl TypedStmt {
    pub fn error(span: Span, error: TypeError) -> Self {
        Self {
            kind: TypedStmtKind::Error { error },
            ty: Ty::Error,
            span,
        }
    }
}
