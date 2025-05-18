//! HIR validation
//!
//! This module provides functions to validate the HIR for correctness.

use crate::hir::types::*;
use std::collections::HashSet;

/// Error type for HIR validation
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Undefined variable
    UndefinedVariable {
        /// Variable name
        name: String,
        /// Usage context
        context: String,
    },
    
    /// Type mismatch
    TypeMismatch {
        /// Expected type
        expected: front_end::types::Type,
        /// Actual type
        actual: front_end::types::Type,
        /// Context for the mismatch
        context: String,
    },
    
    /// Permission error
    PermissionError {
        /// Error message
        message: String,
    },
    
    /// Other errors
    Other(String),
}

/// Validate an HIR program
pub fn validate_hir(program: &HirProgram) -> Result<(), Vec<ValidationError>> {
    // Collect all validations to perform
    let mut errors = Vec::new();
    
    // Run variable declaration check
    if let Err(var_errors) = check_undeclared_variables(program) {
        errors.extend(var_errors);
    }
    
    // Run type compatibility check
    if let Err(type_errors) = check_type_compatibility(program) {
        errors.extend(type_errors);
    }
    
    // Return all errors or success
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Check for undeclared variables in a program
pub fn check_undeclared_variables(program: &HirProgram) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    let mut declared_vars = HashSet::new();
    
    // First collect all variable declarations
    for stmt in &program.statements {
        match stmt {
            HirStatement::Declaration(var) => {
                declared_vars.insert(var.name.clone());
            },
            HirStatement::Function(func) => {
                // Add function parameters as declared variables
                for param in &func.parameters {
                    declared_vars.insert(param.name.clone());
                }
            },
            _ => {}
        }
    }
    
    // Then check all variable usages
    for stmt in &program.statements {
        match stmt {
            HirStatement::Assignment(assign) => {
                if !declared_vars.contains(&assign.target) {
                    errors.push(ValidationError::UndefinedVariable {
                        name: assign.target.clone(),
                        context: "assignment target".to_string(),
                    });
                }
                
                check_expr_for_undeclared(&assign.value, &declared_vars, &mut errors);
            },
            HirStatement::Expression(expr) => {
                check_expr_for_undeclared(expr, &declared_vars, &mut errors);
            },
            HirStatement::Return(expr) => {
                check_expr_for_undeclared(expr, &declared_vars, &mut errors);
            },
            HirStatement::Print(expr) => {
                check_expr_for_undeclared(expr, &declared_vars, &mut errors);
            },
            _ => {}
        }
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Check type compatibility in all expressions
fn check_type_compatibility(program: &HirProgram) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    // Check each statement for type compatibility
    for stmt in &program.statements {
        check_statement_types(stmt, program, &mut errors);
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Check type compatibility for a statement
fn check_statement_types(stmt: &HirStatement, program: &HirProgram, errors: &mut Vec<ValidationError>) {
    match stmt {
        HirStatement::Declaration(var) => {
            // Check initializer type if present
            if let Some(init) = &var.initializer {
                let init_type = infer_expr_type(init, program);
                
                if init_type != var.typ {
                    errors.push(ValidationError::TypeMismatch {
                        expected: var.typ.clone(),
                        actual: init_type,
                        context: format!("initialization of variable '{}'", var.name),
                    });
                }
            }
        },
        HirStatement::Assignment(assign) => {
            // Get target variable type
            if let Some(target_type) = program.type_info.variables.get(&assign.target) {
                let value_type = infer_expr_type(&assign.value, program);
                
                if value_type != *target_type {
                    errors.push(ValidationError::TypeMismatch {
                        expected: target_type.clone(),
                        actual: value_type,
                        context: format!("assignment to variable '{}'", assign.target),
                    });
                }
            }
        },
        HirStatement::Return(expr) => {
            // Find the enclosing function (simplified - in a real compiler we'd track scope)
            // For now, just use the first function we find with a matching return type
            for stmt in &program.statements {
                if let HirStatement::Function(func) = stmt {
                    if let Some(return_type) = &func.return_type {
                        let expr_type = infer_expr_type(expr, program);
                        if expr_type != *return_type {
                            errors.push(ValidationError::TypeMismatch {
                                expected: return_type.clone(),
                                actual: expr_type,
                                context: format!("return value in function '{}'", func.name),
                            });
                        }
                    }
                    break;
                }
            }
        },
        HirStatement::Function(func) => {
            // Check function body
            for stmt in &func.body {
                check_statement_types(stmt, program, errors);
            }
        },
        HirStatement::Block(statements) => {
            // Check each statement in the block
            for stmt in statements {
                check_statement_types(stmt, program, errors);
            }
        },
        // Other statement types could be added here
        _ => {},
    }
}

/// Check an expression for undeclared variables
fn check_expr_for_undeclared(
    expr: &HirExpression, 
    declared: &HashSet<String>,
    errors: &mut Vec<ValidationError>
) {
    match expr {
        HirExpression::Variable(name, _) => {
            if !declared.contains(name) {
                errors.push(ValidationError::UndefinedVariable {
                    name: name.clone(),
                    context: "variable reference".to_string(),
                });
            }
        },
        HirExpression::Binary { left, right, .. } => {
            check_expr_for_undeclared(left, declared, errors);
            check_expr_for_undeclared(right, declared, errors);
        },
        HirExpression::Call { arguments, .. } => {
            for arg in arguments {
                check_expr_for_undeclared(arg, declared, errors);
            }
        },
        HirExpression::Peak(expr) => {
            check_expr_for_undeclared(expr, declared, errors);
        },
        HirExpression::Clone(expr) => {
            check_expr_for_undeclared(expr, declared, errors);
        },
        _ => {}
    }
}

/// Infer the type of an expression
fn infer_expr_type(expr: &HirExpression, program: &HirProgram) -> front_end::types::Type {
    match expr {
        HirExpression::Integer(_) => front_end::types::Type::Int,
        
        HirExpression::Variable(name, typ) => {
            // Look up in the type info first, fall back to the annotated type
            program.type_info.variables.get(name).cloned().unwrap_or_else(|| typ.clone())
        },
        
        HirExpression::Binary { result_type, .. } => result_type.clone(),
        
        HirExpression::Call { function, result_type, .. } => {
            // First check if we have the function's return type
            if let Some(func_type) = program.type_info.functions.get(function) {
                func_type.clone().unwrap_or_else(|| result_type.clone())
            } else {
                // Fall back to the annotated result type
                result_type.clone()
            }
        },
        
        HirExpression::Peak(inner) => infer_expr_type(inner, program),
        
        HirExpression::Clone(inner) => infer_expr_type(inner, program),
    }
}
