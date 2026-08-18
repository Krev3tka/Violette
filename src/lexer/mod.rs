//! Lexical analyzer for Violette.
//!
//! Converts source code into set of [`Token`]s.
//! Categorizes keywords, identifiers, literals and operators.
pub mod lexer;
pub mod span;
pub mod token;
