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

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    Mismatch {
        expected: Ty,
        found: Ty,
    },
    UnknownName(String),
    AlreadyUsed(String),
    NotCallable,
    ArityMismatch {
        name: String,
        expected: usize,
        found: usize,
    },
    UnknownField {
        struct_name: String,
        field: String,
    },
    InvalidOperator {
        operator: Token,
        left: Ty,
        right: Ty,
    },
    NoFields(Ty),
    Unsupported(String),
    DuplicateDefinition(String),
    ConflictingEntryPoint,
    AlreadyDefined(String),
    AssignmentToImmutable {
        name: String,
        kind: BindingKind,
    },
}
