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
        /// Source location of the error
        location: Option<crate::hir::scope::SourceLocation>,
    },
    
    /// Permission error
    PermissionError {
        /// Error message
        message: String,
    },
    
    /// Other errors
    Other(String),
}

impl ValidationError {
    /// Format a validation error for display
    pub fn format(&self, source_code: Option<&str>) -> String {
        match self {
            ValidationError::TypeMismatch { expected, actual, context, location } => {
                // Use {:?} to format Type values since they don't implement Display
                let mut result = format!("Type mismatch error: expected {:?}, found {:?}\n", expected, actual);
                result.push_str(&format!("In {}\n", context));
                
                // Add source code context if available
                if let (Some(loc), Some(source)) = (location, source_code) {
                    result.push_str(&format!(" --> {}:{}:{}\n", loc.file, loc.line, loc.column));
                    
                    // Extract the line with the error
                    let lines: Vec<&str> = source.lines().collect();
                    if loc.line > 0 && loc.line <= lines.len() {
                        let line_content = lines[loc.line - 1].trim_start();
                        result.push_str(&format!("   |\n{} | {}\n", loc.line, line_content));
                        
                        // Find the token length
                        let token_len = if loc.column <= line_content.len() {
                            // Try to find a word boundary
                            let remainder = &line_content[loc.column.saturating_sub(1)..];
                            let end = remainder.find(|c: char| !c.is_alphanumeric() && c != '_')
                                .unwrap_or(remainder.len());
                            end.max(1) // At least one character
                        } else {
                            1 // Default if column is out of range
                        };
                        
                        // Add the tilde marks underlining the error
                        result.push_str(&format!("   | {}{}\n", 
                            " ".repeat(loc.column.saturating_sub(1)), 
                            "~".repeat(token_len)
                        ));
                    }
                } else {
                    // If we don't have location but do have source, try to find the problematic text
                    if let Some(source) = source_code {
                        // Simple heuristic: find the line containing the relevant variable
                        let var_name = if context.contains("assignment to variable") {
                            // Extract variable name from context message
                            context.split('\'').nth(1).unwrap_or("")
                        } else if context.contains("initialization of variable") {
                            // Extract variable name from context message
                            context.split('\'').nth(1).unwrap_or("")
                        } else {
                            ""
                        };
                        
                        if !var_name.is_empty() {
                            // Find the line containing the variable
                            for (i, line) in source.lines().enumerate() {
                                if line.contains(var_name) {
                                    let line_content = line.trim_start();
                                    let line_num = i + 1;
                                    let col = line.find(var_name).unwrap_or(1) + 1;
                                    
                                    // Add formatted line with error marker
                                    result.push_str(&format!(" --> input:{}:{}\n", line_num, col));
                                    result.push_str(&format!("   |\n{} | {}\n", line_num, line_content));
                                    result.push_str(&format!("   | {}{}\n", 
                                        " ".repeat(col.saturating_sub(1)), 
                                        "~".repeat(var_name.len())
                                    ));
                                    break;
                                }
                            }
                        }
                    }
                }
                
                // Add a helpful suggestion
                result.push_str("\nSuggestion: ");
                match (expected, actual) {
                    (front_end::types::Type::Int, front_end::types::Type::Bool) => {
                        result.push_str("Convert the boolean to an integer with a cast, e.g., 'Int(bool_val)' or use a different variable of integer type.");
                    },
                    (front_end::types::Type::Bool, front_end::types::Type::Int) => {
                        result.push_str("Convert the integer to a boolean with a comparison, e.g., 'int_val != 0' or use a different variable of boolean type.");
                    },
                    (front_end::types::Type::Float, front_end::types::Type::Int) => {
                        result.push_str("Convert the integer to a float with a cast, e.g., 'Float(int_val)'.");
                    },
                    (front_end::types::Type::Int, front_end::types::Type::Float) => {
                        result.push_str("Convert the float to an integer with a cast, e.g., 'Int(float_val)'.");
                    },
                    _ => {
                        // Also use {:?} here for Type formatting
                        result.push_str(&format!("Make sure the types match. You cannot assign a value of type '{:?}' to a variable of type '{:?}'.", actual, expected));
                    }
                }
                
                result
            },
            // Handle other validation error types...
            _ => String::new(),
        }
    }
}

/// Validate an HIR program
pub fn validate_hir_with_source(program: &HirProgram, source: &str) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    // Run variable declaration check
    if let Err(var_errors) = check_undeclared_variables(program) {
        errors.extend(var_errors);
    }
    
    // Run type compatibility check
    if let Err(type_errors) = check_type_compatibility_with_source(program, source) {
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
            HirStatement::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    check_expr_for_undeclared(expr, &declared_vars, &mut errors);
                }
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
fn check_type_compatibility_with_source(program: &HirProgram, source: &str) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    // Check each statement for type compatibility
    for stmt in &program.statements {
        check_statement_types_with_source(stmt, program, source, &mut errors);
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Check type compatibility for a statement
fn check_statement_types_with_source(stmt: &HirStatement, program: &HirProgram, source: &str, errors: &mut Vec<ValidationError>) {
    match stmt {
        HirStatement::Declaration(var) => {
            // Check initializer type if present
            if let Some(init) = &var.initializer {
                let init_type = infer_expr_type(init, program);
                
                if init_type != var.typ {
                    // Try to get source location from expression
                    let location = match init {
                        HirExpression::Variable(_, _, loc) => {
                            loc.as_ref().map(|l| crate::hir::scope::SourceLocation {
                                line: l.start.line,
                                column: l.start.column,
                                file: format!("file_{}", l.file_id),
                            })
                        },
                        _ => None,
                    };
                    
                    errors.push(ValidationError::TypeMismatch {
                        expected: var.typ.clone(),
                        actual: init_type,
                        context: format!("initialization of variable '{}'", var.name),
                        location,
                    });
                }
            }
        },
        HirStatement::Assignment(assign) => {
            // Get target variable type
            if let Some(target_type) = program.type_info.variables.get(&assign.target) {
                let value_type = infer_expr_type(&assign.value, program);
                
                if value_type != *target_type {
                    let location = if let HirExpression::Variable(_, _, loc) = &assign.value {
                        loc.as_ref().map(|l| crate::hir::scope::SourceLocation {
                            line: l.start.line,
                            column: l.start.column,
                            file: format!("file_{}", l.file_id),
                        })
                    } else {
                        None
                    };
                    
                    errors.push(ValidationError::TypeMismatch {
                        expected: target_type.clone(),
                        actual: value_type,
                        context: format!("assignment to variable '{}'", assign.target),
                        location,
                    });
                }
            }
        },
        HirStatement::Return(expr_opt) => {
            // Find the enclosing function (simplified - in a real compiler we'd track scope)
            // For now, just use the first function we find with a matching return type
            if let Some(expr) = expr_opt {
                for stmt in &program.statements {
                    if let HirStatement::Function(func) = stmt {
                        if let Some(return_type) = &func.return_type {
                            let expr_type = infer_expr_type(expr, program);
                            if expr_type != *return_type {
                                errors.push(ValidationError::TypeMismatch {
                                    expected: return_type.clone(),
                                    actual: expr_type,
                                    context: format!("return value in function '{}'", func.name),
                                    location: None,
                                });
                            }
                        }
                        break;
                    }
                }
            }
        },
        HirStatement::Function(func) => {
            // Check function body
            for stmt in &func.body {
                check_statement_types_with_source(stmt, program, source, errors);
            }
        },
        HirStatement::Block(statements) => {
            // Check each statement in the block
            for stmt in statements {
                check_statement_types_with_source(stmt, program, source, errors);
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
        HirExpression::Variable(name, _, _) => {
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
        HirExpression::Conditional { condition, then_expr, else_expr, .. } => {
            check_expr_for_undeclared(condition, declared, errors);
            check_expr_for_undeclared(then_expr, declared, errors);
            check_expr_for_undeclared(else_expr, declared, errors);
        },
        HirExpression::Cast { expr, .. } => {
            check_expr_for_undeclared(expr, declared, errors);
        },
        HirExpression::Peak(expr) => {
            check_expr_for_undeclared(expr, declared, errors);
        },
        HirExpression::Clone(expr) => {
            check_expr_for_undeclared(expr, declared, errors);
        },
        // Literals don't contain variables to check
        HirExpression::Integer(_, _) => {},
        HirExpression::Boolean(_) => {},
        HirExpression::String(_) => {},
    }
}

/// Infer the type of an expression
fn infer_expr_type(expr: &HirExpression, program: &HirProgram) -> front_end::types::Type {
    match expr {
        HirExpression::Integer(_, _) => front_end::types::Type::Int,
        
        HirExpression::Variable(name, typ, _) => {
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
        
        // Add implementations for the new expression types
        HirExpression::Boolean(_) => front_end::types::Type::Bool,
        
        HirExpression::String(_) => front_end::types::Type::String,
        
        HirExpression::Conditional { result_type, .. } => result_type.clone(),
        
        HirExpression::Cast { target_type, .. } => target_type.clone(),
    }
}
