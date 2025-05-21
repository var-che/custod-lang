//! High-level Intermediate Representation (HIR)
//!
//! This module provides the HIR representation and related functionality.

pub mod types;
pub mod converter;
pub mod scope;
pub mod validation;
pub mod name_resolver;
pub mod desugar;
pub mod diagnostics;
pub mod permissions;  // Make sure this is public
pub mod const_fold;      // New module for constant folding
pub mod dce;             // New module for dead code elimination
pub mod pretty_print;    // New module for pretty printing
pub mod function_analysis; // Add the new module

// Re-export key functions and types
pub use types::{HirProgram, HirStatement, HirExpression};
pub use converter::{convert_ast_to_hir, convert_statements_to_hir};
pub use name_resolver::{resolve_names, resolve_names_with_source}; // Add the new function
pub use validation::ValidationError;
pub use desugar::desugar_program;
pub use const_fold::fold_constants;
pub use dce::eliminate_dead_code;
pub use pretty_print::pretty_print;
pub use permissions::PermissionChecker;
pub use function_analysis::FunctionPermissionsContext;

/// Analyze a program for permission violations
pub fn check_permissions(program: &HirProgram) -> Vec<permissions::PermissionError> {
    // First perform basic permission checking
    let mut checker = permissions::PermissionChecker::new();
    let basic_errors = checker.check_program(program);
    
    // Then perform function-specific permission analysis
    let mut func_checker = function_analysis::FunctionPermissionsContext::new();
    let func_errors = func_checker.analyze_program(program);
    
    // Combine errors
    let mut all_errors = basic_errors;
    all_errors.extend(func_errors);
    all_errors
}

#[cfg(test)]
pub mod tests;

