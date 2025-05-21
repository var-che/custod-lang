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
    
    /// Track variable locations
    locations: HashMap<String, (usize, usize)>, // (line, column)
}

impl PermissionChecker {
    /// Create a new permission checker
    pub fn new() -> Self {
        Self {
            permissions: HashMap::new(),
            aliases: HashMap::new(),
            exclusive_access: HashMap::new(),
            errors: Vec::new(),
            locations: HashMap::new(), // Add locations tracking
        }
    }
    
    /// Check permissions for a HIR program
    pub fn check_program(&mut self, program: &HirProgram) -> Vec<PermissionError> {
        for statement in &program.statements {
            self.check_statement(statement);
        }
        
        self.errors.clone()
    }
    
    /// Check program permissions with source code
    pub fn check_program_with_source(&mut self, program: &HirProgram, source: &str) -> Vec<PermissionError> {
        // Extract line information from source
        let lines: Vec<&str> = source.lines().collect();
        
        // First collect all variable declarations and their permissions
        for stmt in &program.statements {
            match stmt {
                HirStatement::Declaration(var) => {
                    self.permissions.insert(var.name.clone(), var.permissions.clone());
                    
                    // Try to find the line containing this variable
                    for (i, line) in lines.iter().enumerate() {
                        if line.contains(&var.name) {
                            let column = line.find(&var.name).unwrap_or(0) + 1;
                            self.locations.insert(var.name.clone(), (i + 1, column));
                            break;
                        }
                    }
                },
                _ => {}, // Handle other statement types appropriately
            }
        }
        
        // Then check all statements for permission violations
        for stmt in &program.statements {
            self.check_statement(stmt);
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
        if !self.check_write_permission(target) {
            return;
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
                self.check_read_permission(name);
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
                    self.check_peak_permission(name);
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
        let (has_shareable_perm, source_perms) = self.check_aliasing_permission(source_name);
        
        if !has_shareable_perm {
            return;
        }
        
        // Check write permission conflicts
        if target_perms.contains(&Permission::Write) {
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
        let source_aliases = self.aliases.get(source_name).cloned().unwrap_or_default();
        
        let mut updated_set = source_aliases.clone();
        updated_set.insert(target_name.to_string());
        self.aliases.insert(target_name.to_string(), updated_set.clone());
        
        if let Some(source_set) = self.aliases.get_mut(source_name) {
            source_set.insert(target_name.to_string());
        }
        
        for alias in &source_aliases {
            if alias != target_name && alias != source_name {
                if let Some(other_set) = self.aliases.get_mut(alias) {
                    other_set.insert(target_name.to_string());
                }
            }
        }
    }
    
    /// Check permissions for a function call expression
    pub fn check_function_call(&mut self, function_name: &str, arguments: &[HirExpression]) {
        for arg in arguments {
            self.check_expression_permissions(arg);
        }
    }
    
    /// Check if a variable can be passed to a parameter with given permissions
    pub fn check_parameter_compatibility(&mut self, var_name: &str, param_name: &str, param_perms: &[Permission]) {
        if let Some(var_perms) = self.permissions.get(var_name) {
            let param_needs_exclusive = param_perms.contains(&Permission::Read) && 
                                       param_perms.contains(&Permission::Write) && 
                                       !param_perms.contains(&Permission::Reads) && 
                                       !param_perms.contains(&Permission::Writes);
                                       
            if param_needs_exclusive {
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
            
            if (param_perms.contains(&Permission::Read) || param_perms.contains(&Permission::Reads))
                && !var_perms.contains(&Permission::Read) && !var_perms.contains(&Permission::Reads) {
                self.errors.push(PermissionError {
                    message: format!("Cannot pass '{}' to parameter '{}' - parameter requires read permission", 
                                   var_name, param_name),
                    location: None,
                });
            }
            
            if (param_perms.contains(&Permission::Write) || param_perms.contains(&Permission::Writes))
                && !var_perms.contains(&Permission::Write) && !var_perms.contains(&Permission::Writes) {
                self.errors.push(PermissionError {
                    message: format!("Cannot pass '{}' to parameter '{}' - parameter requires write permission", 
                                   var_name, param_name),
                    location: None,
                });
            }
            
            if param_perms.contains(&Permission::Write) && !param_perms.contains(&Permission::Writes) {
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
        let param_needs_exclusive = param_permissions.contains(&Permission::Read) && 
                                  param_permissions.contains(&Permission::Write) && 
                                  !param_permissions.contains(&Permission::Reads) && 
                                  !param_permissions.contains(&Permission::Writes);
        
        if param_needs_exclusive {
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
    
    /// Check write permissions for an assignment
    fn check_write_permission(&mut self, target: &str) -> bool {
        match self.permissions.get(target) {
            Some(perms) => {
                let has_write = perms.contains(&Permission::Write) || perms.contains(&Permission::Writes);
                if !has_write {
                    let mut message = format!("Cannot write to '{}' - no write permission", target);
                    
                    if let Some(location) = self.locations.get(target) {
                        message = format!("{} | x = y\n    ~ -> Cannot write to '{}' - no write permission", location.0, target);
                        
                        if perms.contains(&Permission::Reads) {
                            message.push_str(&format!("\nsuggestion: reads write {} -> add write permission", target));
                        } else if perms.contains(&Permission::Read) {
                            message.push_str(&format!("\nsuggestion: read write {} -> add write permission", target));
                        } else {
                            message.push_str(&format!("\nsuggestion: write {} -> add write permission", target));
                        }
                    } else {
                        if perms.contains(&Permission::Reads) {
                            message.push_str(&format!("\n\nSuggestion:\nreads write {0}: Int = ...\n      ~~~~~ -> add write permission here", target));
                        } else if perms.contains(&Permission::Read) {
                            message.push_str(&format!("\n\nSuggestion:\nread write {0}: Int = ...\n     ~~~~~ -> add write permission here", target));
                        } else {
                            message.push_str(&format!("\n\nSuggestion:\nwrite {0}: Int = ...\n~~~~~ -> add write permission here", target));
                        }
                    }
                    
                    self.errors.push(PermissionError {
                        message,
                        location: None,
                    });
                }
                has_write
            },
            None => {
                self.errors.push(PermissionError {
                    message: format!("Cannot write to '{}' - variable not found", target),
                    location: None,
                });
                false
            }
        }
    }

    /// Check read permissions for variable access
    fn check_read_permission(&mut self, target: &str) -> bool {
        match self.permissions.get(target) {
            Some(perms) => {
                let has_read = perms.contains(&Permission::Read) || perms.contains(&Permission::Reads);
                if !has_read {
                    let mut message = format!("Cannot read from '{}' - no read permission", target);
                    
                    if let Some(location) = self.locations.get(target) {
                        message = format!("{} | x = y\n    ~ -> Cannot read from '{}' - no read permission", location.0, target);
                        
                        if perms.contains(&Permission::Writes) {
                            message.push_str(&format!("\nsuggestion: reads writes {} -> add reads permission", target));
                        } else if perms.contains(&Permission::Write) {
                            message.push_str(&format!("\nsuggestion: read write {} -> add read permission", target));
                        } else {
                            message.push_str(&format!("\nsuggestion: read {} -> add read permission", target));
                        }
                    } else {
                        if perms.contains(&Permission::Writes) {
                            message.push_str(&format!("\n\nSuggestion:\nreads writes {0}: Int = ...\n~~~~~ -> add reads permission here", target));
                        } else if perms.contains(&Permission::Write) {
                            message.push_str(&format!("\n\nSuggestion:\nread write {0}: Int = ...\n~~~~ -> add read permission here", target));
                        } else {
                            message.push_str(&format!("\n\nSuggestion:\nread {0}: Int = ...\n~~~~ -> add read permission here", target));
                        }
                    }
                    
                    self.errors.push(PermissionError {
                        message,
                        location: None,
                    });
                }
                has_read
            },
            None => {
                self.errors.push(PermissionError {
                    message: format!("Cannot read from '{}' - variable not found", target),
                    location: None,
                });
                false
            }
        }
    }
    
    /// Check permissions for peak operation
    fn check_peak_permission(&mut self, target: &str) -> bool {
        match self.permissions.get(target) {
            Some(perms) => {
                let has_read = perms.contains(&Permission::Read) || perms.contains(&Permission::Reads);
                if !has_read {
                    let mut message = format!("Cannot peak '{}' - peak requires read permission", target);
                    
                    if let Some(location) = self.locations.get(target) {
                        message = format!("{} | x = y\n    ~ -> Cannot peak '{}' - peak requires read permission", location.0, target);
                        
                        if perms.contains(&Permission::Write) {
                            message.push_str(&format!("\nsuggestion: read write {} -> add read permission", target));
                        } else if perms.contains(&Permission::Writes) {
                            message.push_str(&format!("\nsuggestion: reads writes {} -> add reads permission", target));
                        } else {
                            message.push_str(&format!("\nsuggestion: read {} -> add read permission", target));
                        }
                    } else {
                        if perms.contains(&Permission::Write) {
                            message.push_str(&format!("\n\nSuggestion:\nread write {0}: Int = ...\n~~~~ -> add read permission here", target));
                        } else if perms.contains(&Permission::Writes) {
                            message.push_str(&format!("\n\nSuggestion:\nreads writes {0}: Int = ...\n~~~~~ -> add reads permission here", target));
                        } else {
                            message.push_str(&format!("\n\nSuggestion:\nread {0}: Int = ...\n~~~~ -> add read permission here", target));
                        }
                    }
                    
                    self.errors.push(PermissionError {
                        message,
                        location: None,
                    });
                }
                has_read
            },
            None => {
                self.errors.push(PermissionError {
                    message: format!("Cannot peak '{}' - variable not found", target),
                    location: None,
                });
                false
            }
        }
    }
    
    /// Check if aliasing is allowed for a variable
    fn check_aliasing_permission(&mut self, source: &str) -> (bool, Vec<Permission>) {
        match self.permissions.get(source) {
            Some(perms) => {
                let has_shareable_perm = perms.iter().any(|p| 
                    matches!(p, Permission::Reads | Permission::Writes)
                );
                
                let has_read = perms.contains(&Permission::Read) || perms.contains(&Permission::Reads);
                
                if !has_shareable_perm && has_read {
                    let mut message = format!("Cannot create alias to '{}' - variable has non-shareable permissions", source);
                    
                    if perms.contains(&Permission::Read) && perms.contains(&Permission::Write) {
                        message.push_str(&format!("\nsuggestion: reads writes {} -> use shareable permissions instead of read write", source));
                    } else if perms.contains(&Permission::Read) {
                        message.push_str(&format!("\nsuggestion: reads {} -> use reads instead of read", source));
                    } else if perms.contains(&Permission::Write) {
                        message.push_str(&format!("\nsuggestion: writes {} -> use writes instead of write", source));
                    }
                    
                    self.errors.push(PermissionError {
                        message,
                        location: None,
                    });
                }
                
                (has_shareable_perm, perms.clone())
            },
            None => {
                self.errors.push(PermissionError {
                    message: format!("Cannot alias '{}' - variable not found", source),
                    location: None,
                });
                (false, vec![])
            }
        }
    }
}

/// Create a new public function to check permissions with source code
pub fn check_permissions_with_source(program: &HirProgram, source: &str) -> Vec<PermissionError> {
    let mut checker = PermissionChecker::new();
    checker.check_program_with_source(program, source)
}