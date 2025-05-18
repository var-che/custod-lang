use std::collections::{HashMap, HashSet};
use crate::types::{Type, Permission, PermissionedType};
use crate::ast::{Statement, Expression};

/// Location information for better error reporting
#[derive(Debug, Clone)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

/// Symbol represents a variable or function in the code
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub typ: PermissionedType,
    pub kind: SymbolKind,
    pub location: Location,
}

/// Different kinds of symbols
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable,
    Parameter,
    Function,
}

/// A scope represents a lexical block with its own variable declarations
#[derive(Debug)]
struct Scope {
    symbols: HashMap<String, Symbol>,
    parent: Option<usize>, // Index of parent scope in SymbolTable's scopes vec
}

/// Error type for symbol resolution
#[derive(Debug)]
pub enum ResolutionError {
    DuplicateSymbol{name: String, first: Location, second: Location},
    UndefinedSymbol{name: String, location: Location},
    ImmutableAssignment{name: String, location: Location},
    PermissionViolation{name: String, required: String, provided: String, location: Location},
    // Add more error types as needed
}

impl std::fmt::Display for ResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ResolutionError::DuplicateSymbol{name, first, second} => {
                write!(f, "Error: Variable '{}' already defined at line {}, but redeclared at line {}",
                    name, first.line, second.line)
            },
            ResolutionError::UndefinedSymbol{name, location} => {
                write!(f, "Error at line {}: Variable '{}' not defined in this scope",
                    location.line, name)
            },
            ResolutionError::ImmutableAssignment{name, location} => {
                write!(f, "Error at line {}: Cannot assign to immutable variable '{}'",
                    location.line, name)
            },
            ResolutionError::PermissionViolation{name, required, provided, location} => {
                write!(f, "Error at line {}: Variable '{}' requires permission '{}' but has '{}'",
                    location.line, name, required, provided)
            },
        }
    }
}

/// The Symbol Table manages variable scopes and provides methods for resolving variables
pub struct SymbolTable {
    scopes: Vec<Scope>,
    current_scope: usize,
    errors: Vec<ResolutionError>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut scopes = Vec::new();
        
        // Start with global scope
        scopes.push(Scope {
            symbols: HashMap::new(),
            parent: None,
        });
        
        Self {
            scopes,
            current_scope: 0,
            errors: Vec::new(),
        }
    }
    
    pub fn begin_scope(&mut self) -> usize {
        let parent = self.current_scope;
        let new_scope_idx = self.scopes.len();
        
        self.scopes.push(Scope {
            symbols: HashMap::new(),
            parent: Some(parent),
        });
        
        self.current_scope = new_scope_idx;
        new_scope_idx
    }
    
    pub fn end_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current_scope].parent {
            self.current_scope = parent;
        }
    }
    
    pub fn define(&mut self, symbol: Symbol) {
        // Check for duplicate in current scope
        if let Some(existing) = self.scopes[self.current_scope].symbols.get(&symbol.name) {
            self.errors.push(ResolutionError::DuplicateSymbol{
                name: symbol.name.clone(),
                first: existing.location.clone(),
                second: symbol.location.clone(),
            });
            return;
        }
        
        // Add to current scope
        self.scopes[self.current_scope].symbols.insert(
            symbol.name.clone(), symbol
        );
    }
    
    pub fn resolve(&mut self, name: &str, location: Location) -> Option<&Symbol> {
        let mut scope_idx = self.current_scope;
        
        loop {
            if let Some(symbol) = self.scopes[scope_idx].symbols.get(name) {
                return Some(symbol);
            }
            
            // Move to parent scope if it exists
            match self.scopes[scope_idx].parent {
                Some(parent_idx) => scope_idx = parent_idx,
                None => break, // We've reached the global scope
            }
        }
        
        // Symbol not found
        self.errors.push(ResolutionError::UndefinedSymbol{
            name: name.to_string(),
            location,
        });
        None
    }
    
    pub fn check_assignment(&mut self, name: &str, location: Location) -> Result<(), ResolutionError> {
        match self.resolve(name, location.clone()) {
            Some(symbol) => {
                // Check if variable has write permission
                if symbol.typ.permissions.contains(&Permission::Write) ||
                   symbol.typ.permissions.contains(&Permission::Writes) {
                    Ok(())
                } else {
                    Err(ResolutionError::ImmutableAssignment{
                        name: name.to_string(),
                        location,
                    })
                }
            },
            None => Err(ResolutionError::UndefinedSymbol{
                name: name.to_string(),
                location,
            }),
        }
    }
    
    // Add visitor methods to process AST nodes
    pub fn process_statement(&mut self, stmt: &Statement, token_locations: &HashMap<usize, Location>) {
        match stmt {
            Statement::Declaration{name, typ, initializer} => {
                let location = token_locations.get(&self.current_scope)
                    .cloned().unwrap_or(Location{line: 0, column: 0});
                
                self.define(Symbol {
                    name: name.clone(),
                    typ: typ.clone(),
                    kind: SymbolKind::Variable,
                    location,
                });
                
                // Process initializer if present
                if let Some(expr) = initializer {
                    self.process_expression(expr, token_locations);
                }
            },
            Statement::Assignment{target, value, ..} => {
                // Check if variable exists and is writable
                let location = token_locations.get(&self.current_scope)
                    .cloned().unwrap_or(Location{line: 0, column: 0});
                
                let _ = self.check_assignment(target, location);
                self.process_expression(value, token_locations);
            },
            // Handle other statement types...
            _ => {}
        }
    }
    
    pub fn process_expression(&mut self, expr: &Expression, token_locations: &HashMap<usize, Location>) {
        match expr {
            Expression::Variable(name) => {
                let location = token_locations.get(&self.current_scope)
                    .cloned().unwrap_or(Location{line: 0, column: 0});
                
                let _ = self.resolve(name, location);
            },
            Expression::Binary{left, right, ..} => {
                self.process_expression(left, token_locations);
                self.process_expression(right, token_locations);
            },
            // Handle other expression types...
            _ => {}
        }
    }
    
    pub fn get_errors(&self) -> &[ResolutionError] {
        &self.errors
    }
}