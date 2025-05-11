//! High-level Intermediate Representation (HIR)
//!
//! This module manages the transformation from AST to a higher-level
//! representation optimized for analysis and optimization.

pub mod types;
pub mod converters;
pub mod permissions;

// Re-export key types for convenient access
pub use types::{
    HirValue, HirStatement, HirVariable, HirProgram, 
    HirActor, HirMethod, HirBehavior, MethodKind,
    HirAssignment, TypeEnvironment
};
pub use converters::{convert_to_hir, HirConverter, convert_expression};
pub use permissions::{PermissionInfo, PermissionChecker};

mod tests;