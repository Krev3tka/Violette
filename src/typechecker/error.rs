use crate::typechecker::types::Ty;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    Mismatch {
        expected: Ty,
        found: Ty,
    },
    UnknownName(String),
    NotCallable,
    ArityMismatch {
        name: String,
        expected: usize,
        found: usize,
    },
    Unsupported(String),
    DuplicateDefinition(String),
    ConflictingEntryPoint
}
