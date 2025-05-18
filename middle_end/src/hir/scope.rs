//! Scope handling for HIR
//!
//! This module implements a scope tracking system for the HIR representation.

use std::collections::{HashMap, HashSet};
use front_end::types::Type;
use crate::hir::types::*;

/// Represents a symbol in the symbol table
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    
    /// Symbol type
    pub typ: Type,
    
    /// Symbol permissions
    pub permissions: Vec<Permission>,
    
    /// Is this symbol a function?
    pub is_function: bool,
    
    /// Symbol definition location
    pub location: Option<SourceLocation>,
}

/// Source location information
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// Line number (1-based)
    pub line: usize,
    
    /// Column number (1-based)
    pub column: usize,
    
    /// Source file name
    pub file: String,
}

/// A symbol table that tracks scopes and symbols
pub struct SymbolTable {
    /// Stack of scopes, with innermost scope at the end
    scopes: Vec<HashMap<String, Symbol>>,
    
    /// Track name usage to detect shadowing
    used_names: HashSet<String>,
}

/// Error information for scope and name resolution issues
#[derive(Debug, Clone)]
pub enum ScopeError {
    /// Symbol already defined in the current scope
    AlreadyDefined {
        /// Symbol name
        name: String,
        
        /// Previous definition location
        previous: Option<SourceLocation>,
    },
    
    /// Symbol shadows another definition
    Shadowing {
        /// Symbol name
        name: String,
        
        /// Previous definition location
        previous: Option<SourceLocation>,
    },
    
    /// Symbol not found in any scope
    NotFound {
        /// Symbol name
        name: String,
    },
}

impl SymbolTable {
    /// Create a new symbol table with a global scope
    pub fn new() -> Self {
        let mut table = Self {
            scopes: Vec::new(),
            used_names: HashSet::new(),
        };
        
        // Initialize with global scope
        table.enter_scope();
        
        table
    }
    
    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    
    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {  // Keep at least the global scope
            self.scopes.pop();
        }
    }
    
    /// Add a symbol to the current scope
    pub fn add_symbol(&mut self, symbol: Symbol) -> Result<(), ScopeError> {
        let name = symbol.name.clone();
        
        // Check if already defined in current scope
        if let Some(current_scope) = self.scopes.last() {
            if current_scope.contains_key(&name) {
                let previous = current_scope.get(&name).and_then(|sym| sym.location.clone());
                return Err(ScopeError::AlreadyDefined { name, previous });
            }
        }
        
        // Check for shadowing in outer scopes
        let mut previous_location = None;
        let is_shadowing = self.scopes.len() > 1 && self.used_names.contains(&name);
        
        if is_shadowing {
            // Find previous definition without borrowing self.scopes mutably
            // This avoids the borrowing conflict
            for scope in &self.scopes[0..self.scopes.len()-1] {
                if let Some(sym) = scope.get(&name) {
                    previous_location = sym.location.clone();
                    break;
                }
            }
        }
        
        // Now do the mutable borrow safely
        if let Some(current_scope) = self.scopes.last_mut() {
            // Add to current scope
            current_scope.insert(name.clone(), symbol);
            self.used_names.insert(name.clone());
            
            // Report shadowing if found
            if is_shadowing {
                return Err(ScopeError::Shadowing { 
                    name, 
                    previous: previous_location 
                });
            }
        }
        
        Ok(())
    }
    
    /// Look up a symbol in all scopes, starting from innermost
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        // Search from innermost scope to outermost
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        
        None
    }
    
    /// Look up a symbol in the current scope only
    pub fn lookup_in_current_scope(&self, name: &str) -> Option<&Symbol> {
        self.scopes.last().and_then(|scope| scope.get(name))
    }
    
    /// Get all symbols defined in the current scope
    pub fn get_current_scope_symbols(&self) -> Vec<&Symbol> {
        if let Some(current_scope) = self.scopes.last() {
            current_scope.values().collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get the current scope depth (0 = global)
    pub fn scope_depth(&self) -> usize {
        self.scopes.len() - 1
    }
}
