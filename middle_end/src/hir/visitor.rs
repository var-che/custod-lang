//! HIR visitor traits
//!
//! This module defines visitor traits for HIR traversal

use crate::hir::types::*;

/// Trait for HIR visitors
pub trait HirVisitor {
    /// Visit a program
    fn visit_program(&mut self, program: &HirProgram);
    
    /// Visit a statement
    fn visit_statement(&mut self, stmt: &HirStatement);
    
    /// Visit a function
    fn visit_function(&mut self, func: &HirFunction);
    
    /// Visit a variable declaration
    fn visit_variable(&mut self, var: &HirVariable);
    
    /// Visit an expression
    fn visit_expression(&mut self, expr: &HirExpression);
    
    // Default implementations that traverse the HIR structure
    // ...
}

/// Walk through all nodes in a HIR program using a visitor
pub fn walk_program<V: HirVisitor>(visitor: &mut V, program: &HirProgram) {
    visitor.visit_program(program);
    
    for stmt in &program.statements {
        visitor.visit_statement(stmt);
    }
}

// Implement various analysis passes using the visitor pattern
pub mod passes {
    use super::*;
    
    /// Constant folding visitor
    pub struct ConstantFolder {
        // ...
    }
    
    impl HirVisitor for ConstantFolder {
        // ...
    }
    
    /// Dead code elimination visitor
    pub struct DeadCodeEliminator {
        // ...
    }
    
    impl HirVisitor for DeadCodeEliminator {
        // ...
    }
}
