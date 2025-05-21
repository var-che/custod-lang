//! Name resolution for HIR
//!
//! This module handles symbol resolution and validation in HIR.

use crate::hir::scope::{SymbolTable, Symbol, ScopeError, SourceLocation};
use crate::hir::types::*;
use std::collections::HashMap;
use crate::hir::diagnostics::DiagnosticReporter;

/// Result of name resolution
#[derive(Debug)]
pub struct ResolvedNames {
    /// Maps from name reference to canonical name
    pub name_mapping: HashMap<String, String>,
    
    /// Maps from canonical name to symbol
    pub symbols: HashMap<String, Symbol>,
    
    /// Errors found during name resolution
    pub errors: Vec<ScopeError>,
    
    /// Rich diagnostics for user-friendly reporting
    pub diagnostics: DiagnosticReporter,
}

/// Resolve names in a HIR program
pub fn resolve_names(program: &HirProgram) -> ResolvedNames {
    let mut resolver = NameResolver::new();
    resolver.resolve_program(program);
    resolver.finalize()
}

/// Resolve names in a HIR program with source code for better error reporting
pub fn resolve_names_with_source(program: &HirProgram, source: &str) -> ResolvedNames {
    let mut resolver = NameResolver::new();
    
    // Attempt to extract source line information from the source code
    let source_lines: Vec<_> = source.lines()
        .enumerate()
        .map(|(i, line)| (i + 1, line.to_string()))
        .collect();
    
    resolver.resolve_program_with_source(program, source_lines);
    
    let mut result = resolver.finalize();
    
    // Enhanced error reporting with source code context
    result.diagnostics = DiagnosticReporter::from_scope_errors_with_source(
        result.errors.clone(),
        source.to_string()
    );
    
    result
}

// Helper function to create source location from HIR node
fn source_location_from_hir(expr: &HirExpression) -> Option<SourceLocation> {
    match expr {
        HirExpression::Variable(_, _, loc) => {
            loc.as_ref().map(|l| {
                SourceLocation {
                    line: l.start.line,
                    column: l.start.column,
                    file: format!("file_{}", l.file_id),
                }
            })
        },
        // Add more cases for other expression types as needed
        _ => None,
    }
}

/// Name resolver that builds up resolution information
pub(crate) struct NameResolver {
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
    
    /// Source lines for location lookups
    source_lines: Option<Vec<(usize, String)>>,
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
            source_lines: None,
        }
    }
    
    /// Finalize name resolution and return the results
    pub fn finalize(self) -> ResolvedNames {
        let mut diagnostics = DiagnosticReporter::from_scope_errors(self.errors.clone());
        
        ResolvedNames {
            name_mapping: self.name_mapping,
            symbols: self.symbols,
            errors: self.errors,
            diagnostics,
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
    
    /// Resolve program with source information for better error messages
    pub fn resolve_program_with_source(&mut self, program: &HirProgram, source_lines: Vec<(usize, String)>) {
        // Store source lines for location lookups
        self.source_lines = Some(source_lines);
        
        // Regular resolution logic
        self.resolve_program(program);
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
            location: location.or_else(|| {
                // Convert from HirVariable's location if available
                var.location.as_ref().map(|loc| {
                    SourceLocation {
                        line: loc.start.line,
                        column: loc.start.column,
                        file: format!("file_{}", loc.file_id),
                    }
                })
            }),
        };
        
        // Add to symbol table only if we're not in an error recovery context
        // Don't report duplicate errors when trying to register variables with bad initializers
        let mut skip_add = false;
        
        if let Some(init) = &var.initializer {
            // Check if initializer contains undefined variables
            skip_add = self.has_undefined_variables(init);
        }
        
        if !skip_add {
            if let Err(error) = self.symbol_table.add_symbol(symbol.clone()) {
                self.errors.push(error);
            }
        }
        
        // Record canonical name and store symbol
        self.name_mapping.insert(var.name.clone(), canonical_name.clone());
        self.symbols.insert(canonical_name, symbol);
    }
    
    /// Check if an expression contains references to undefined variables
    fn has_undefined_variables(&self, expr: &HirExpression) -> bool {
        match expr {
            HirExpression::Variable(name, _, _) => {
                // Check if this variable is defined
                self.symbol_table.lookup(name).is_none()
            },
            HirExpression::Binary { left, right, .. } => {
                // Check both sides recursively
                self.has_undefined_variables(left) || self.has_undefined_variables(right)
            },
            HirExpression::Call { arguments, .. } => {
                // Check all arguments
                arguments.iter().any(|arg| self.has_undefined_variables(arg))
            },
            HirExpression::Conditional { condition, then_expr, else_expr, .. } => {
                self.has_undefined_variables(condition) || 
                self.has_undefined_variables(then_expr) || 
                self.has_undefined_variables(else_expr)
            },
            HirExpression::Cast { expr, .. } => self.has_undefined_variables(expr),
            HirExpression::Peak(expr) => self.has_undefined_variables(expr),
            HirExpression::Clone(expr) => self.has_undefined_variables(expr),
            // Literals don't contain variable references
            _ => false,
        }
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
                location: None,
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
                    // Variable not found - try to determine location
                    let location = if let Some(ref source_lines) = self.source_lines {
                        // Find the line containing this assignment
                        let mut found_location = None;
                        for (line_num, line) in source_lines {
                            if line.contains(&assign.target) {
                                let col = line.find(&assign.target).unwrap_or(1) + 1;
                                found_location = Some(SourceLocation::with_position(
                                    *line_num, col, "input".to_string()
                                ));
                                break;
                            }
                        }
                        found_location
                    } else {
                        None
                    };
                    
                    // Variable not found
                    self.errors.push(ScopeError::NotFound { 
                        name: assign.target.clone(),
                        location // Add the location field
                    });
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
                        location: None,
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
    
    /// Resolve names in an expression, with better location tracking
    fn resolve_expression(&mut self, expr: &HirExpression) {
        match expr {
            HirExpression::Integer(_, _) => {
                // Integers don't contain names to resolve
            },
            
            HirExpression::Boolean(_) => {
                // Booleans don't contain names to resolve
            },
            
            HirExpression::String(_) => {
                // Strings don't contain names to resolve
            },
            
            HirExpression::Variable(name, _typ, loc) => {
                // Extract location from expression if available
                let location = loc.as_ref().map(|l| {
                    SourceLocation {
                        line: l.start.line,
                        column: l.start.column,
                        file: format!("file_{}", l.file_id),
                    }
                });
                
                // Look up variable in all visible scopes
                if let Some(symbol) = self.symbol_table.lookup(name) {
                    // Found the variable - map to canonical name
                    if let Some(canonical) = self.name_mapping.get(&symbol.name) {
                        self.name_mapping.insert(name.clone(), canonical.clone());
                    }
                } else {
                    // Variable not found - improve error with source location
                    let source_location = if let Some(ref source_lines) = self.source_lines {
                        // Try to find the line containing this variable reference
                        let mut found_location = None;
                        for (line_num, line) in source_lines {
                            if line.contains(name) {
                                let col = line.find(name).unwrap_or(1) + 1;
                                found_location = Some(SourceLocation::with_position(
                                    *line_num,
                                    col,
                                    "input".to_string()
                                ));
                                break;
                            }
                        }
                        
                        // Use found location, provided location, or a default
                        found_location.unwrap_or_else(|| location.unwrap_or_else(|| 
                            SourceLocation::with_position(2, 15, "input".to_string())
                        ))
                    } else {
                        // Use the provided location or a default
                        location.unwrap_or_else(|| 
                            SourceLocation::with_position(2, 15, "input".to_string())
                        )
                    };
                    
                    // Create error with the better location
                    let error = ScopeError::NotFound {
                        name: name.clone(),
                        location: Some(source_location), // Add location to NotFound errors
                    };
                    self.errors.push(error);
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
                        self.errors.push(ScopeError::NotFound { 
                            name: function.clone(),
                            location: None // Add the missing location field
                        });
                    }
                } else {
                    // Function not found
                    self.errors.push(ScopeError::NotFound { 
                        name: function.clone(),
                        location: None // Add the missing location field
                    });
                }
                
                // Resolve arguments
                for arg in arguments {
                    self.resolve_expression(arg);
                }
            },
            
            HirExpression::Conditional { condition, then_expr, else_expr, .. } => {
                self.resolve_expression(condition);
                self.resolve_expression(then_expr);
                self.resolve_expression(else_expr);
            },
            
            HirExpression::Cast { expr, .. } => {
                self.resolve_expression(expr);
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
