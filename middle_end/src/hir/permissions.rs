//! Permission checking for HIR
//!
//! This module implements the permission checking system for the HIR representation.

use front_end::types::Permission;

use crate::hir::types::*;
use std::collections::{HashMap, HashSet};

/// Error information for permission checking
#[derive(Debug, Clone)]
pub struct PermissionError {
    /// Error message
    pub message: String,
    
    /// Source location information (line, column)
    pub location: Option<(usize, usize)>,
}

/// Permission checking context
pub struct PermissionChecker {
    /// Maps variable names to their permissions
    permissions: HashMap<String, Vec<Permission>>,
    
    /// Tracks which variables alias the same memory
    aliases: HashMap<String, HashSet<String>>,
    
    /// Tracks exclusive access variables
    exclusive_access: HashMap<String, String>,
    
    /// Errors found during permission checking
    errors: Vec<PermissionError>,
}

impl PermissionChecker {
    /// Create a new permission checker
    pub fn new() -> Self {
        Self {
            permissions: HashMap::new(),
            aliases: HashMap::new(),
            exclusive_access: HashMap::new(),
            errors: Vec::new(),
        }
    }
    
    /// Check permissions for a HIR program
    pub fn check_program(&mut self, program: &HirProgram) -> Vec<PermissionError> {
        // Adapt to the structure that HirProgram actually has
        // If it has 'statements' instead of 'items', use that
        for statement in &program.statements {
            self.check_statement(statement);
        }
        
        self.errors.clone()
    }
    
    /// Register a variable with its permissions
    fn register_variable(&mut self, name: &str, perms: &[Permission]) {
        self.permissions.insert(name.to_string(), perms.to_vec());
        
        // Track exclusive access
        if perms.contains(&Permission::Read) && 
           perms.contains(&Permission::Write) && 
           !perms.contains(&Permission::Reads) && 
           !perms.contains(&Permission::Writes) {
            
            self.exclusive_access.insert(name.to_string(), name.to_string());
        }
        
        // Initialize alias set
        let mut alias_set = HashSet::new();
        alias_set.insert(name.to_string());
        self.aliases.insert(name.to_string(), alias_set);
    }
    
    /// Check permissions for a statement
    pub fn check_statement(&mut self, stmt: &HirStatement) {
        match stmt {
            HirStatement::Declaration(var) => self.check_variable_declaration(var),
            HirStatement::Assignment(assign) => self.check_assignment(&assign.target, &assign.value),
            HirStatement::Expression(expr) => { self.check_expression_permissions(expr); },
            HirStatement::Return(expr) => {
                if let Some(expr) = expr {
                    self.check_expression_permissions(expr);
                }
            },
            HirStatement::Print(expr) => {
                self.check_expression_permissions(expr);
            },
            HirStatement::Block(statements) => {
                // Create a new scope
                let old_permissions = self.permissions.clone();
                let old_aliases = self.aliases.clone();
                let old_exclusive = self.exclusive_access.clone();
                
                // Check each statement in the block
                for stmt in statements {
                    self.check_statement(stmt);
                }
                
                // Restore old scope
                self.permissions = old_permissions;
                self.aliases = old_aliases;
                self.exclusive_access = old_exclusive;
            },
            HirStatement::Function(func) => self.check_function(func),
            _ => {}, // Handle other statement types appropriately
        }
    }
    
    /// Check permissions for a function
    fn check_function(&mut self, func: &HirFunction) {
        // Create a new scope for function parameters
        let old_permissions = self.permissions.clone();
        let old_aliases = self.aliases.clone();
        let old_exclusive = self.exclusive_access.clone();
        
        // Add parameters to scope
        for param in &func.parameters {
            self.register_variable(&param.name, &param.permissions);
        }
        
        // Check function body
        for stmt in &func.body {
            self.check_statement(stmt);
        }
        
        // Restore old scope
        self.permissions = old_permissions;
        self.aliases = old_aliases;
        self.exclusive_access = old_exclusive;
    }
    
    /// Check permissions for a variable declaration
    fn check_variable_declaration(&mut self, var: &HirVariable) {
        // Log permissions for debugging
        if cfg!(test) {
            println!("Registering variable '{}' with permissions: {:?}", var.name, var.permissions);
        }
        
        // Register variable with its permissions
        self.register_variable(&var.name, &var.permissions);
        
        // Check initializer permissions
        if let Some(init) = &var.initializer {
            self.check_expression_permissions(init);
            
            // If it's a variable reference, handle aliasing
            if let HirExpression::Variable(source_name, _, _) = init {
                if cfg!(test) {
                    println!("Checking aliasing from '{}' to '{}'", source_name, var.name);
                    
                    // Print permissions for both variables
                    if let Some(source_perms) = self.permissions.get(source_name) {
                        println!("  Source '{}' permissions: {:?}", source_name, source_perms);
                    }
                    println!("  Target '{}' permissions: {:?}", var.name, var.permissions);
                }
                
                self.check_aliasing(&var.name, source_name, &var.permissions);
            }
        }
    }
    
    /// Check permissions for an assignment
    fn check_assignment(&mut self, target: &str, value: &HirExpression) {
        // Check if target has write permission
        if let Some(perms) = self.permissions.get(target) {
            if !perms.contains(&Permission::Write) && !perms.contains(&Permission::Writes) {
                self.errors.push(PermissionError {
                    message: format!("Cannot write to '{}' - no write permission", target),
                    location: None,
                });
            }
        } else {
            self.errors.push(PermissionError {
                message: format!("Assignment to undefined variable '{}'", target),
                location: None,
            });
        }
        
        // Check value permissions
        self.check_expression_permissions(value);
    }
    
    /// Check permissions for an expression
    fn check_expression_permissions(&mut self, expr: &HirExpression) {
        match expr {
            HirExpression::Integer(_, _) => (), // No permission checking needed for literals
            HirExpression::Boolean(_) => (), // No permission checking needed for literals
            HirExpression::String(_) => (),  // No permission checking needed for literals
            
            HirExpression::Variable(name, _, _) => {
                // Check if variable has read permission
                if let Some(perms) = self.permissions.get(name) {
                    if !perms.contains(&Permission::Read) && !perms.contains(&Permission::Reads) {
                        self.errors.push(PermissionError {
                            message: format!("Cannot read from '{}' - no read permission", name),
                            location: None,
                        });
                    }
                } else {
                    self.errors.push(PermissionError {
                        message: format!("Reference to undefined variable '{}'", name),
                        location: None,
                    });
                }
            },
            
            HirExpression::Binary { left, right, .. } => {
                self.check_expression_permissions(left);
                self.check_expression_permissions(right);
            },
            
            HirExpression::Call { arguments, .. } => {
                for arg in arguments {
                    self.check_expression_permissions(arg);
                }
            },
            
            HirExpression::Conditional { condition, then_expr, else_expr, .. } => {
                self.check_expression_permissions(condition);
                self.check_expression_permissions(then_expr);
                self.check_expression_permissions(else_expr);
            },
            
            HirExpression::Cast { expr, .. } => {
                self.check_expression_permissions(expr);
            },
            
            HirExpression::Peak(expr) => {
                // For Peak, we need to check special permission rules
                if let HirExpression::Variable(name, _, _) = &**expr {
                    if let Some(perms) = self.permissions.get(name) {
                        if !perms.contains(&Permission::Read) && !perms.contains(&Permission::Reads) {
                            self.errors.push(PermissionError {
                                message: format!("Cannot peak '{}' - requires read/reads permission", name),
                                location: None,
                            });
                        }
                    }
                } else {
                    self.check_expression_permissions(expr);
                }
            },
            
            HirExpression::Clone(expr) => {
                self.check_expression_permissions(expr);
            },
        }
    }
    
    /// Check for proper aliasing permissions
    fn check_aliasing(&mut self, target_name: &str, source_name: &str, target_perms: &[Permission]) {
        if let Some(source_perms) = self.permissions.get(source_name) {
            // Check for non-shareable permissions first
            let source_has_read = source_perms.contains(&Permission::Read);
            let source_has_write = source_perms.contains(&Permission::Write);
            let source_has_reads = source_perms.contains(&Permission::Reads);
            let source_has_writes = source_perms.contains(&Permission::Writes);
            
            // Debug output for tests
            if cfg!(test) {
                println!("  Aliasing check: '{}' non-shareable={}", 
                         source_name, 
                         (source_has_read || source_has_write) && !(source_has_reads || source_has_writes));
            }
            
            // Non-shareable permissions (read/write without reads/writes) can't be aliased
            if (source_has_read || source_has_write) && !(source_has_reads || source_has_writes) {
                self.errors.push(PermissionError {
                    message: format!(
                        "Cannot create alias to '{}' - it has non-shareable permissions (read/write without reads/writes)",
                        source_name
                    ),
                    location: None,
                });
                return; // Early return after detecting non-shareable aliasing violation
            }
            
            // Check for exclusive access conflicts - add additional debug information
            let is_exclusive = source_has_read && 
                              source_has_write && 
                              !source_has_reads && 
                              !source_has_writes;
                              
            if cfg!(test) {
                println!("  Aliasing check: '{}' exclusive={}", source_name, is_exclusive);
                
                if is_exclusive && target_perms.contains(&Permission::Write) {
                    println!("  PERMISSION ERROR: Cannot create write alias to an exclusive variable");
                }
            }
                              
            if is_exclusive {
                // Source has exclusive access - can't alias with write permission
                if target_perms.contains(&Permission::Write) {
                    self.errors.push(PermissionError {
                        message: format!(
                            "Cannot create write alias to '{}' - it has exclusive permissions (Read+Write without Reads/Writes)", 
                            source_name
                        ),
                        location: None,
                    });
                }
            }
            
            // Check write permission conflicts
            if target_perms.contains(&Permission::Write) {
                // Fix aliasing check to avoid borrowing conflicts
                let conflicting_aliases = self.aliases.get(source_name)
                    .map(|aliases| {
                        aliases.iter()
                            .filter(|&alias| alias != target_name)
                            .filter_map(|alias| {
                                self.permissions.get(alias).map(|perms| 
                                    (alias.clone(), perms.contains(&Permission::Write))
                                )
                            })
                            .filter(|(_, has_write)| *has_write)
                            .map(|(name, _)| name)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                
                for existing in &conflicting_aliases {
                    self.errors.push(PermissionError {
                        message: format!("Cannot create write alias to '{}' - '{}' already has write permission", 
                                       source_name, existing),
                        location: None,
                    });
                }
            }
            
            // Update alias sets safely
            // First collect all the alias information
            let source_aliases = self.aliases.get(source_name).cloned().unwrap_or_default();
            
            // Update target's alias set
            let mut updated_set = source_aliases.clone();
            updated_set.insert(target_name.to_string());
            self.aliases.insert(target_name.to_string(), updated_set.clone());
            
            // Update source's alias set
            if let Some(source_set) = self.aliases.get_mut(source_name) {
                source_set.insert(target_name.to_string());
            }
            
            // Update all other aliases to include the new alias
            for alias in &source_aliases {
                if alias != target_name && alias != source_name {
                    if let Some(other_set) = self.aliases.get_mut(alias) {
                        other_set.insert(target_name.to_string());
                    }
                }
            }
        }
    }
    
    /// Check permissions for a function call expression
    pub fn check_function_call(&mut self, function_name: &str, arguments: &[HirExpression]) {
        // Check permissions for each argument
        for arg in arguments {
            self.check_expression_permissions(arg);
        }
        
        // For a full implementation, we would:
        // 1. Look up the function signature
        // 2. Check if argument permissions are compatible with parameter requirements
        // 3. Apply Pony-style reference capability conversion rules
        
        // This would be integrated with the FunctionPermissionsContext
    }
    
    /// Check if a variable can be passed to a parameter with given permissions
    pub fn check_parameter_compatibility(&mut self, var_name: &str, param_name: &str, param_perms: &[Permission]) {
        if let Some(var_perms) = self.permissions.get(var_name) {
            // Check if the variable's permissions are compatible with parameter requirements
            
            // 1. If parameter needs exclusive access (read+write without reads/writes)
            let param_needs_exclusive = param_perms.contains(&Permission::Read) && 
                                       param_perms.contains(&Permission::Write) && 
                                       !param_perms.contains(&Permission::Reads) && 
                                       !param_perms.contains(&Permission::Writes);
                                       
            if param_needs_exclusive {
                // Variable must have equivalent or stronger permissions
                let var_has_exclusive = var_perms.contains(&Permission::Read) && 
                                       var_perms.contains(&Permission::Write) && 
                                       !var_perms.contains(&Permission::Reads) && 
                                       !var_perms.contains(&Permission::Writes);
                                       
                if !var_has_exclusive {
                    self.errors.push(PermissionError {
                        message: format!("Cannot pass '{}' to parameter '{}' - parameter requires exclusive access", 
                                       var_name, param_name),
                        location: None,
                    });
                }
            }
            
            // 2. If parameter needs read permission
            if (param_perms.contains(&Permission::Read) || param_perms.contains(&Permission::Reads))
                && !var_perms.contains(&Permission::Read) && !var_perms.contains(&Permission::Reads) {
                self.errors.push(PermissionError {
                    message: format!("Cannot pass '{}' to parameter '{}' - parameter requires read permission", 
                                   var_name, param_name),
                    location: None,
                });
            }
            
            // 3. If parameter needs write permission
            if (param_perms.contains(&Permission::Write) || param_perms.contains(&Permission::Writes))
                && !var_perms.contains(&Permission::Write) && !var_perms.contains(&Permission::Writes) {
                self.errors.push(PermissionError {
                    message: format!("Cannot pass '{}' to parameter '{}' - parameter requires write permission", 
                                   var_name, param_name),
                    location: None,
                });
            }
            
            // Check aliasing rules similar to Pony's reference capabilities
            if param_perms.contains(&Permission::Write) && !param_perms.contains(&Permission::Writes) {
                // For non-shareable write parameters, check if the variable has aliases
                if let Some(aliases) = self.aliases.get(var_name) {
                    if aliases.len() > 1 {
                        self.errors.push(PermissionError {
                            message: format!("Cannot pass aliased variable '{}' to parameter '{}' requiring exclusive write access", 
                                           var_name, param_name),
                            location: None,
                        });
                    }
                }
            }
        }
    }
    
    /// Register a function parameter with its permissions
    pub fn register_parameter(&mut self, name: &str, permissions: &[Permission]) {
        self.register_variable(name, permissions);
    }
    
    /// Get all accumulated errors
    pub fn get_errors(&self) -> Vec<PermissionError> {
        self.errors.clone()
    }
    
    /// Check if a variable can be passed to a function argument with given permissions
    pub fn check_variable_aliasing_for_function_arg(
        &mut self,
        var_name: &str, 
        param_permissions: &[Permission],
        function_name: &str,
        param_index: usize
    ) -> Option<PermissionError> {
        // In a real implementation, you'd look up the variable's actual permissions
        // and check if they're compatible with the parameter permissions.
        
        // For now, just check if we're requiring exclusive permission
        let param_needs_exclusive = param_permissions.contains(&Permission::Read) && 
                                  param_permissions.contains(&Permission::Write) && 
                                  !param_permissions.contains(&Permission::Reads) && 
                                  !param_permissions.contains(&Permission::Writes);
        
        if param_needs_exclusive {
            // Similar to Pony's iso capability, exclusive access parameters
            // can only accept variables with exclusive access that aren't aliased.
            
            // This is a placeholder implementation. In a real implementation,
            // you would check if the variable has proper exclusive permissions
            // and is not aliased elsewhere.
            
            return Some(PermissionError {
                message: format!(
                    "Parameter {} of function '{}' requires exclusive permission (like Pony's iso), but this cannot be guaranteed for '{}'",
                    param_index + 1,
                    function_name,
                    var_name
                ),
                location: None,
            });
        }
        
        None
    }
}