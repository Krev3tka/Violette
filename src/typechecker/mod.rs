//! Semantics and type checking phase.
//!
//! Traverses built by parser AST and checks types correctness before codegen phase.
//! Also checks visibility scopes and callable entities.

pub mod checker;
pub mod env;
pub mod error;
pub mod types;
