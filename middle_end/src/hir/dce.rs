//! Dead code elimination for HIR
//!
//! This module implements basic dead code elimination optimizations.

use crate::hir::types::*;
use std::collections::HashSet;

/// Eliminate dead code in a HIR program
pub fn eliminate_dead_code(program: &mut HirProgram) {
    // First, identify used variables
    let used_variables = find_used_variables(program);
    
    // Then remove unused variable declarations
    program.statements.retain(|stmt| {
        match stmt {
            HirStatement::Declaration(var) => {
                used_variables.contains(&var.name)
            },
            // Keep all other statements
            _ => true,
        }
    });
    
    // Process nested blocks and function bodies
    for i in 0..program.statements.len() {
        if let Some(stmt) = program.statements.get_mut(i) {
            eliminate_dead_code_in_statement(stmt, &used_variables);
        }
    }
}

/// Find all variables that are actually used in the program
fn find_used_variables(program: &HirProgram) -> HashSet<String> {
    let mut used = HashSet::new();
    
    // First pass: collect all uses
    for stmt in &program.statements {
        collect_used_variables(stmt, &mut used);
    }
    
    used
}

/// Collect variable uses from a statement
fn collect_used_variables(stmt: &HirStatement, used: &mut HashSet<String>) {
    match stmt {
        HirStatement::Declaration(var) => {
            // Process initializer if present
            if let Some(init) = &var.initializer {
                collect_used_variables_expr(init, used);
            }
        },
        
        HirStatement::Assignment(assign) => {
            // Assignment target is considered used
            used.insert(assign.target.clone());
            collect_used_variables_expr(&assign.value, used);
        },
        
        HirStatement::Expression(expr) => {
            collect_used_variables_expr(expr, used);
        },
        
        HirStatement::Return(expr_opt) => {
            if let Some(expr) = expr_opt {
                collect_used_variables_expr(expr, used);
            }
        },
        
        HirStatement::Print(expr) => {
            collect_used_variables_expr(expr, used);
        },
        
        HirStatement::Block(statements) => {
            for stmt in statements {
                collect_used_variables(stmt, used);
            }
        },
        
        HirStatement::Function(func) => {
            // Function parameters are considered used within the function
            for param in &func.parameters {
                used.insert(param.name.clone());
            }
            
            // Process function body
            for stmt in &func.body {
                collect_used_variables(stmt, used);
            }
        },
        
        HirStatement::If { condition, then_branch, else_branch } => {
            collect_used_variables_expr(condition, used);
            collect_used_variables(then_branch, used);
            if let Some(else_stmt) = else_branch {
                collect_used_variables(else_stmt, used);
            }
        },
        
        HirStatement::While { condition, body } => {
            collect_used_variables_expr(condition, used);
            collect_used_variables(body, used);
        },
    }
}

/// Collect variable uses from an expression
fn collect_used_variables_expr(expr: &HirExpression, used: &mut HashSet<String>) {
    match expr {
        HirExpression::Variable(name, _, _) => {
            used.insert(name.clone());
        },
        
        HirExpression::Binary { left, right, .. } => {
            collect_used_variables_expr(left, used);
            collect_used_variables_expr(right, used);
        },
        
        HirExpression::Call { function: _, arguments, .. } => {
            // Function name itself is not tracked as a variable use here
            for arg in arguments {
                collect_used_variables_expr(arg, used);
            }
        },
        
        HirExpression::Conditional { condition, then_expr, else_expr, .. } => {
            collect_used_variables_expr(condition, used);
            collect_used_variables_expr(then_expr, used);
            collect_used_variables_expr(else_expr, used);
        },
        
        HirExpression::Cast { expr, .. } => {
            collect_used_variables_expr(expr, used);
        },
        
        HirExpression::Peak(expr) => {
            collect_used_variables_expr(expr, used);
        },
        
        HirExpression::Clone(expr) => {
            collect_used_variables_expr(expr, used);
        },
        
        // Literals don't use variables
        _ => {},
    }
}

/// Recursively eliminate dead code in statement blocks
fn eliminate_dead_code_in_statement(stmt: &mut HirStatement, used_variables: &HashSet<String>) {
    match stmt {
        HirStatement::Block(statements) => {
            // Remove unused variable declarations
            statements.retain(|stmt| {
                match stmt {
                    HirStatement::Declaration(var) => used_variables.contains(&var.name),
                    _ => true,
                }
            });
            
            // Recursively process the remaining statements
            for sub_stmt in statements.iter_mut() {
                eliminate_dead_code_in_statement(sub_stmt, used_variables);
            }
        },
        
        HirStatement::Function(func) => {
            // Process function body
            for sub_stmt in func.body.iter_mut() {
                eliminate_dead_code_in_statement(sub_stmt, used_variables);
            }
        },
        
        HirStatement::If { then_branch, else_branch, .. } => {
            eliminate_dead_code_in_statement(then_branch, used_variables);
            if let Some(else_stmt) = else_branch {
                eliminate_dead_code_in_statement(else_stmt, used_variables);
            }
        },
        
        HirStatement::While { body, .. } => {
            eliminate_dead_code_in_statement(body, used_variables);
        },
        
        _ => {},
    }
}
