use crate::lexer::span::Span;
use crate::lexer::token::Token;
use crate::typechecker::types::Ty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Var,
    Let,
    Const,
}

impl BindingKind {
    pub fn is_mutable(self) -> bool {
        matches!(self, BindingKind::Var)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DefinitionKind {
    Var,
    Fun,
    Struct,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoopControlKind {
    Break,
    Continue,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    Mismatch {
        expected: Ty,
        found: Ty,
        span: Span,
    },

    UnknownName {
        name: String,
        span: Span,
    },

    NotCallable {
        ty: Ty,
        span: Span,
    },

    ArityMismatch {
        name: String,
        expected: usize,
        found: usize,
        span: Span,
    },

    UnknownField {
        struct_name: String,
        field: String,
        span: Span,
    },

    InvalidBinaryOperator {
        operator: Token,
        left: Box<Ty>,
        right: Box<Ty>,
        span: Span,
    },

    InvalidUnaryOperator {
        operator: Token,
        operand: Ty,
        span: Span,
    },

    NoFields {
        ty: Ty,
        span: Span,
    },

    Unsupported {
        desc: String,
        span: Span,
    },

    DuplicateDefinition {
        name: String,
        first_span: Span,
        second_span: Span,
        def_kind: DefinitionKind,
    },

    ConflictingEntryPoint {
        first_decl_span: Span,
        second_decl_span: Span,
    },

    AssignmentToImmutable {
        name: String,
        kind: BindingKind,
        decl_span: Span,
        assign_span: Span,
    },

    MissingReturn {
        name: String,
        fun_span: Span,
        close_brace_span: Span,
    },

    OutsideLoop {
        kind: LoopControlKind,
        span: Span,
    },

    NoSuchMethod {
        ty: Ty,
        method: String,
        span: Span,
    },
}
