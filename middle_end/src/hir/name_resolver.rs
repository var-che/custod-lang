//! Name resolution for HIR
//!
//! This module handles symbol resolution and validation in HIR.

use crate::hir::scope::{SymbolTable, Symbol, ScopeError, SourceLocation};
use crate::hir::types::*;
use std::collections::HashMap;

/// Result of name resolution
#[derive(Debug)]
pub struct ResolvedNames {
    /// Maps from name reference to canonical name
    pub name_mapping: HashMap<String, String>,
    
    /// Maps from canonical name to symbol
    pub symbols: HashMap<String, Symbol>,
    
    /// Errors found during name resolution
    pub errors: Vec<ScopeError>,
}

/// Resolve names in a HIR program
pub fn resolve_names(program: &HirProgram) -> ResolvedNames {
    let mut resolver = NameResolver::new();
    resolver.resolve_program(program);
    resolver.finalize()
}

/// Name resolver that builds up resolution information
struct NameResolver {
    /// Symbol table to track scopes and symbols
    symbol_table: SymbolTable,
    
    /// Maps each variable use to its canonical declaration
    name_mapping: HashMap<String, String>,
    
    /// Maps canonical names to their symbols
    symbols: HashMap<String, Symbol>,
    
    /// Unique counter for generating canonical names
    unique_counter: usize,
    
    /// Errors encountered during resolution
    errors: Vec<ScopeError>,
}

impl NameResolver {
    /// Create a new name resolver
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            name_mapping: HashMap::new(),
            symbols: HashMap::new(),
            unique_counter: 0,
            errors: Vec::new(),
        }
    }
    
    /// Finalize name resolution and return the results
    pub fn finalize(self) -> ResolvedNames {
        ResolvedNames {
            name_mapping: self.name_mapping,
            symbols: self.symbols,
            errors: self.errors,
        }
    }
    
    /// Generate a unique canonical name
    fn generate_canonical_name(&mut self, base_name: &str) -> String {
        let canonical = format!("{}_{}", base_name, self.unique_counter);
        self.unique_counter += 1;
        canonical
    }
    
    /// Resolve names in a program
    pub fn resolve_program(&mut self, program: &HirProgram) {
        // First pass: register all top-level declarations
        for statement in &program.statements {
            match statement {
                HirStatement::Declaration(var) => {
                    self.register_variable(var, None);
                },
                HirStatement::Function(func) => {
                    self.register_function(func, None);
                },
                _ => {}
            }
        }
        
        // Second pass: resolve variable references in bodies
        for statement in &program.statements {
            self.resolve_statement(statement);
        }
    }
    
    /// Register a variable declaration in the symbol table
    fn register_variable(&mut self, var: &HirVariable, location: Option<SourceLocation>) {
        let canonical_name = self.generate_canonical_name(&var.name);
        
        // Create a symbol for the variable
        let symbol = Symbol {
            name: var.name.clone(),
            typ: var.typ.clone(),
            permissions: var.permissions.clone(),
            is_function: false,
            location,
        };
        
        // Add to symbol table and track any errors
        if let Err(error) = self.symbol_table.add_symbol(symbol.clone()) {
            self.errors.push(error);
        }
        
        // Record canonical name and store symbol
        self.name_mapping.insert(var.name.clone(), canonical_name.clone());
        self.symbols.insert(canonical_name, symbol);
    }
    
    /// Register a function declaration in the symbol table
    fn register_function(&mut self, func: &HirFunction, location: Option<SourceLocation>) {
        let canonical_name = self.generate_canonical_name(&func.name);
        
        // Create a symbol for the function
        let symbol = Symbol {
            name: func.name.clone(),
            typ: func.return_type.clone().unwrap_or(front_end::types::Type::Unit),
            permissions: Vec::new(), // Functions don't have permissions
            is_function: true,
            location,
        };
        
        // Add to symbol table and track any errors
        if let Err(error) = self.symbol_table.add_symbol(symbol.clone()) {
            self.errors.push(error);
        }
        
        // Record canonical name and store symbol
        self.name_mapping.insert(func.name.clone(), canonical_name.clone());
        self.symbols.insert(canonical_name, symbol);
        
        // Process function body with a new scope
        self.symbol_table.enter_scope();
        
        // Register parameters
        for param in &func.parameters {
            self.register_variable(&HirVariable {
                name: param.name.clone(),
                typ: param.typ.clone(),
                permissions: param.permissions.clone(),
                initializer: None,
            }, None);
        }
        
        // Resolve body statements
        for stmt in &func.body {
            self.resolve_statement(stmt);
        }
        
        self.symbol_table.exit_scope();
    }
    
    /// Resolve names in a statement
    fn resolve_statement(&mut self, stmt: &HirStatement) {
        match stmt {
            HirStatement::Declaration(var) => {
                // First handle the initializer if present
                if let Some(init) = &var.initializer {
                    self.resolve_expression(init);
                }
                
                // Then register the variable in the current scope
                self.register_variable(var, None);
            },
            
            HirStatement::Assignment(assign) => {
                // Resolve the right-hand side expression
                self.resolve_expression(&assign.value);
                
                // Resolve the target variable
                if let Some(symbol) = self.symbol_table.lookup(&assign.target) {
                    // Found the variable - map to canonical name
                    if let Some(canonical) = self.name_mapping.get(&symbol.name) {
                        self.name_mapping.insert(assign.target.clone(), canonical.clone());
                    }
                } else {
                    // Variable not found
                    self.errors.push(ScopeError::NotFound { name: assign.target.clone() });
                }
            },
            
            HirStatement::Expression(expr) => {
                self.resolve_expression(expr);
            },
            
            HirStatement::Print(expr) => {
                self.resolve_expression(expr);
            },
            
            HirStatement::Block(statements) => {
                // Create a new scope for the block
                self.symbol_table.enter_scope();
                
                for stmt in statements {
                    self.resolve_statement(stmt);
                }
                
                self.symbol_table.exit_scope();
            },
            
            HirStatement::Function(func) => {
                // Already handled in the first pass
                // But we might need to resolve names within the function body
                self.symbol_table.enter_scope();
                
                // Register parameters again to ensure proper scoping
                for param in &func.parameters {
                    self.register_variable(&HirVariable {
                        name: param.name.clone(),
                        typ: param.typ.clone(),
                        permissions: param.permissions.clone(),
                        initializer: None,
                    }, None);
                }
                
                // Resolve body statements
                for stmt in &func.body {
                    self.resolve_statement(stmt);
                }
                
                self.symbol_table.exit_scope();
            },
            
            _ => {}  // Handle other statements as needed
        }
    }
    
    /// Resolve names in an expression
    fn resolve_expression(&mut self, expr: &HirExpression) {
        match expr {
            HirExpression::Integer(_) => {
                // Integers don't contain names to resolve
            },
            
            HirExpression::Variable(name, _typ) => {
                // Look up variable in all visible scopes
                if let Some(symbol) = self.symbol_table.lookup(name) {
                    // Found the variable - map to canonical name
                    if let Some(canonical) = self.name_mapping.get(&symbol.name) {
                        self.name_mapping.insert(name.clone(), canonical.clone());
                    }
                } else {
                    // Variable not found
                    self.errors.push(ScopeError::NotFound { name: name.clone() });
                }
            },
            
            HirExpression::Binary { left, right, .. } => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            },
            
            HirExpression::Call { function, arguments, .. } => {
                // Resolve function name
                if let Some(symbol) = self.symbol_table.lookup(function) {
                    if symbol.is_function {
                        // Found the function - map to canonical name
                        if let Some(canonical) = self.name_mapping.get(&symbol.name) {
                            self.name_mapping.insert(function.clone(), canonical.clone());
                        }
                    } else {
                        // Symbol exists but is not a function
                        self.errors.push(ScopeError::NotFound { name: function.clone() });
                    }
                } else {
                    // Function not found
                    self.errors.push(ScopeError::NotFound { name: function.clone() });
                }
                
                // Resolve arguments
                for arg in arguments {
                    self.resolve_expression(arg);
                }
            },
        
            
            HirExpression::Peak(expr) => {
                self.resolve_expression(expr);
            },
            
            HirExpression::Clone(expr) => {
                self.resolve_expression(expr);
            },
        }
    }
}
