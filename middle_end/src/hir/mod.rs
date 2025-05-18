//! High-level Intermediate Representation (HIR)
//!
//! The HIR is a slightly lower-level representation than the AST,
//! with resolved names, explicit types, and simplified constructs.

mod types;
mod converter;
pub mod validation;
mod scope;
mod name_resolver;
mod desugar;

// Re-export key types and functions
pub use types::*;
pub use converter::{convert_ast_to_hir, convert_statements_to_hir};
pub use validation::validate_hir;
pub use name_resolver::resolve_names;
pub use desugar::desugar_program;

#[cfg(test)]
mod tests;

