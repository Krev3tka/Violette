//! Parser and AST builder for Violette.
//!
//! Takes `Vec<Token>` and builds an **A**bstract **S**yntax **T**ree, checking correctness of the written code.
//!
//! **AST** contains different [`Statement`]s and [`Expression`]s.

#[macro_use]
pub mod parser;
pub mod expression;
pub mod program;
pub mod statement;
pub mod types;

pub use expression::Expression;
pub use statement::Statement;
pub use types::{ParseError, Precedence};
