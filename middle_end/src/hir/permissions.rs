//! Permission checking for HIR
//!
//! This module implements the permission checking system for the HIR representation.

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
    fn check_statement(&mut self, stmt: &HirStatement) {
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
        // Register variable with its permissions
        self.register_variable(&var.name, &var.permissions);
        
        // Check initializer permissions
        if let Some(init) = &var.initializer {
            self.check_expression_permissions(init);
            
            // If it's a variable reference, handle aliasing
            if let HirExpression::Variable { name: ref source_name, .. } = init {
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
            HirExpression::Literal(_) => (), // No permission checking needed for literals
            HirExpression::Variable { name, .. } => {
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
            HirExpression::Field { object, .. } => {
                self.check_expression_permissions(object);
            },
            HirExpression::Peak { expr } => {
                // For Peak, we need to check special permission rules
                if let HirExpression::Variable { name, .. } = &**expr {
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
            HirExpression::Clone { expr } => {
                self.check_expression_permissions(expr);
            },
        }
    }
    
    /// Check for proper aliasing permissions
    fn check_aliasing(&mut self, target_name: &str, source_name: &str, target_perms: &[Permission]) {
        if let Some(source_perms) = self.permissions.get(source_name) {
            // Check for exclusive access conflicts
            if source_perms.contains(&Permission::Read) && 
               source_perms.contains(&Permission::Write) && 
               !source_perms.contains(&Permission::Reads) && 
               !source_perms.contains(&Permission::Writes) {
                
                // Source has exclusive access - can't alias with write permission
                if target_perms.contains(&Permission::Write) {
                    self.errors.push(PermissionError {
                        message: format!("Cannot create write alias to '{}' - it has exclusive permissions", source_name),
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
}