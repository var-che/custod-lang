//! Function Analysis for HIR
//!
//! This module handles function-specific analysis including:
//! - Permission tracking across function calls
//! - Permissions propagation for function parameters and return values
//! - Detection of capability violations in function calls

use crate::hir::types::*;
use crate::hir::permissions::{PermissionChecker, PermissionError};
use front_end::types::{Permission, Type};
use std::collections::{HashMap, HashSet};

/// Function permissions context
pub struct FunctionPermissionsContext {
    /// Maps function names to their signature permissions
    function_signatures: HashMap<String, FunctionSignature>,
    
    /// Permission errors found during analysis
    errors: Vec<PermissionError>,
}

/// A function signature with permission information
#[derive(Clone, Debug)]
pub struct FunctionSignature {
    /// Function name
    name: String,
    
    /// Parameter types and permissions
    parameters: Vec<(String, Type, Vec<Permission>)>,
    
    /// Return type and permissions
    return_type: Option<(Type, Vec<Permission>)>,
}

impl FunctionPermissionsContext {
    /// Create a new function permissions context
    pub fn new() -> Self {
        Self {
            function_signatures: HashMap::new(),
            errors: Vec::new(),
        }
    }
    
    /// Register a function signature
    pub fn register_function(&mut self, func: &HirFunction) {
        let mut params = Vec::new();
        
        for param in &func.parameters {
            params.push((
                param.name.clone(),
                param.typ.clone(),
                param.permissions.clone()
            ));
        }
        
        // For now, assumed permissions on return values
        // In a full implementation, we'd extract these from the function signature
        let return_info = func.return_type.clone().map(|typ| {
            // Default to read+write (isolated) permissions for return values
            // This is similar to Pony's approach where returned values default to iso
            let permissions = vec![Permission::Read, Permission::Write];
            (typ, permissions)
        });
        
        let signature = FunctionSignature {
            name: func.name.clone(),
            parameters: params,
            return_type: return_info,
        };
        
        self.function_signatures.insert(func.name.clone(), signature);
    }
    
    /// Analyze function calls in a program
    pub fn analyze_program(&mut self, program: &HirProgram) -> Vec<PermissionError> {
        // First register all function signatures
        for stmt in &program.statements {
            if let HirStatement::Function(func) = stmt {
                self.register_function(func);
            }
        }
        
        // Then analyze function bodies
        for stmt in &program.statements {
            if let HirStatement::Function(func) = stmt {
                self.analyze_function_body(func);
            }
        }
        
        // Run a second phase of analysis for call sites
        for stmt in &program.statements {
            self.analyze_statement_for_calls(stmt);
        }
        
        self.errors.clone()
    }
    
    /// Analyze function body for permission issues
    fn analyze_function_body(&mut self, func: &HirFunction) {
        // Create a permission checker for this function context
        let mut checker = PermissionChecker::new();
        
        // Register parameters with their permissions
        for param in &func.parameters {
            checker.register_parameter(&param.name, &param.permissions);
        }
        
        // Check the body statements
        for stmt in &func.body {
            checker.check_statement(stmt);
        }
        
        // Collect any errors
        self.errors.extend(checker.get_errors());
    }
    
    /// Analyze a statement for function calls
    fn analyze_statement_for_calls(&mut self, stmt: &HirStatement) {
        match stmt {
            HirStatement::Expression(expr) => {
                self.analyze_expression_for_calls(expr);
            },
            HirStatement::Assignment(assign) => {
                self.analyze_expression_for_calls(&assign.value);
            },
            HirStatement::Print(expr) => {
                self.analyze_expression_for_calls(expr);
            },
            HirStatement::Block(statements) => {
                for stmt in statements {
                    self.analyze_statement_for_calls(stmt);
                }
            },
            HirStatement::Function(func) => {
                for stmt in &func.body {
                    self.analyze_statement_for_calls(stmt);
                }
            },
            HirStatement::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    self.analyze_expression_for_calls(expr);
                }
            },
            HirStatement::Declaration(var) => {
                if let Some(init) = &var.initializer {
                    self.analyze_expression_for_calls(init);
                }
            },
            // Other statement types don't contain function calls
            _ => {},
        }
    }
    
    /// Analyze an expression for function calls
    fn analyze_expression_for_calls(&mut self, expr: &HirExpression) {
        match expr {
            HirExpression::Call { function, arguments, .. } => {
                // Check if the function signature is registered
                if let Some(signature) = self.function_signatures.get(function) {
                    // Clone the signature to avoid borrowing conflicts
                    let signature_clone = signature.clone();
                    // Check argument permissions match parameter requirements
                    self.check_argument_permissions(function, arguments, &signature_clone);
                } else {
                    // Unknown function
                    self.errors.push(PermissionError {
                        message: format!("Call to unknown function '{}'", function),
                        location: None,
                    });
                }
            },
            HirExpression::Binary { left, right, .. } => {
                self.analyze_expression_for_calls(left);
                self.analyze_expression_for_calls(right);
            },
            HirExpression::Conditional { condition, then_expr, else_expr, .. } => {
                self.analyze_expression_for_calls(condition);
                self.analyze_expression_for_calls(then_expr);
                self.analyze_expression_for_calls(else_expr);
            },
            HirExpression::Cast { expr, .. } => {
                self.analyze_expression_for_calls(expr);
            },
            HirExpression::Peak(expr) => {
                self.analyze_expression_for_calls(expr);
            },
            HirExpression::Clone(expr) => {
                self.analyze_expression_for_calls(expr);
            },
            // Literals and variables don't contain function calls
            _ => {},
        }
    }
    
    /// Check argument permissions against parameter requirements
    fn check_argument_permissions(
        &mut self, 
        function_name: &str,
        arguments: &[HirExpression],
        signature: &FunctionSignature
    ) {
        // This is where we'd implement Pony-like permission compatibility rules
        
        // Check if we have the right number of arguments
        if arguments.len() != signature.parameters.len() {
            self.errors.push(PermissionError {
                message: format!(
                    "Function '{}' expects {} arguments, but {} were provided",
                    function_name,
                    signature.parameters.len(),
                    arguments.len()
                ),
                location: None,
            });
            return;
        }
        
        // For each argument, check permission compatibility
        for (i, arg) in arguments.iter().enumerate() {
            if let HirExpression::Variable(name, _, _) = arg {
                // Example check for an arg that's a variable reference
                self.check_variable_permission_for_arg(
                    name, 
                    &signature.parameters[i].2,
                    function_name,
                    i
                );
            }
        }
    }
    
    /// Check if a variable has appropriate permissions to be passed as an argument
    fn check_variable_permission_for_arg(
        &mut self,
        var_name: &str,
        param_permissions: &[Permission],
        function_name: &str,
        param_index: usize
    ) {
        // Check for exclusive permissions required by parameter
        let has_exclusive_param = param_permissions.contains(&Permission::Read) && 
                                param_permissions.contains(&Permission::Write) &&
                                !param_permissions.contains(&Permission::Reads) &&
                                !param_permissions.contains(&Permission::Writes);
                                
        if has_exclusive_param {
            // For parameters requiring exclusive access, we need to verify the variable
            // actually has exclusive permissions compatible with Pony's iso capability
            
            // Create a temporary full permission checker to find the variable's actual permissions
            let mut temp_checker = PermissionChecker::new();
            let err = temp_checker.check_variable_aliasing_for_function_arg(
                var_name, param_permissions, function_name, param_index
            );
            
            if let Some(error) = err {
                self.errors.push(error);
            }
        }
    }
}
