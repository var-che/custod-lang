//! Memory management for the interpreter
//!
//! Handles variable scopes, shared writes, and peak views.

use std::collections::{HashMap, VecDeque};

/// Manages memory for the interpreter
#[derive(Debug)]
pub struct MemoryManager {
    /// Global scope for variables
    global_variables: HashMap<String, i64>,
    
    /// Stack of local scopes for function calls
    scopes: VecDeque<VariableScope>,
    
    /// Track shared write aliases
    shared_writes: HashMap<String, Vec<String>>,
    
    /// Track peak view relationships (view -> source)
    peak_views: HashMap<String, String>,
}

/// Represents a single variable scope
#[derive(Debug)]
pub struct VariableScope {
    /// Variables in this scope
    pub variables: HashMap<String, i64>,
}

impl VariableScope {
    /// Create a new empty scope
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new() -> Self {
        Self {
            global_variables: HashMap::new(),
            scopes: VecDeque::new(),
            shared_writes: HashMap::new(),
            peak_views: HashMap::new(),
        }
    }
    
    /// Get a variable from any scope
    pub fn get_variable(&self, name: &str) -> Option<i64> {
        // First check if this is a peak view
        if let Some(source) = self.peak_views.get(name) {
            // Return the source's current value (could be in any scope)
            return self.get_variable(source);
        }
        
        // Check current scope first (most recently pushed)
        for scope in &self.scopes {
            if let Some(value) = scope.variables.get(name) {
                return Some(*value);
            }
        }
        
        // If not found in local scopes, check global scope
        self.global_variables.get(name).copied()
    }
    
    /// Set variable in the appropriate scope
    pub fn set_variable(&mut self, name: &str, value: i64) {
        // First try to set in local scopes
        for scope in &mut self.scopes {
            if scope.variables.contains_key(name) {
                scope.variables.insert(name.to_string(), value);
                
                // Update all variables that share write access with the target
                self.update_shared_writes(name, value);
                return;
            }
        }
        
        // If not found in any scope, add to current scope
        // (or global if no scopes exist)
        if !self.scopes.is_empty() {
            // Add to most recent scope (front of the deque)
            self.scopes.front_mut().unwrap().variables.insert(name.to_string(), value);
        } else {
            // Add to global scope if no local scopes exist
            self.global_variables.insert(name.to_string(), value);
        }
        
        // Update all variables that share write access with the target
        self.update_shared_writes(name, value);
    }
    
    /// Update all variables that share write access
    fn update_shared_writes(&mut self, name: &str, value: i64) {
        if let Some(aliases) = self.shared_writes.get(name).cloned() {
            for alias in aliases {
                if alias != name {
                    // Skip the name itself to avoid infinite recursion
                    // Set directly in the appropriate scope
                    for scope in &mut self.scopes {
                        if scope.variables.contains_key(&alias) {
                            scope.variables.insert(alias.clone(), value);
                            return;
                        }
                    }
                    
                    // If not in any scope, update global
                    if self.global_variables.contains_key(&alias) {
                        self.global_variables.insert(alias.clone(), value);
                    }
                }
            }
        }
    }
    
    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.scopes.push_front(VariableScope::new());
    }
    
    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        if let Some(scope) = self.scopes.pop_front() {
            // Clean up any references to variables in this scope
            let variables_to_remove: Vec<String> = scope.variables.keys().cloned().collect();
            
            // Clean up shared write relationships
            for var_name in &variables_to_remove {
                if let Some(aliases) = self.shared_writes.get(var_name).cloned() {
                    // Remove relationships that involve this variable
                    for alias in aliases {
                        if let Some(related_vars) = self.shared_writes.get_mut(&alias) {
                            related_vars.retain(|v| v != var_name);
                        }
                    }
                }
                // Remove this variable's shared write entry
                self.shared_writes.remove(var_name);
            }
            
            // Clean up peak views that reference variables in this scope
            let mut peak_views_to_remove = Vec::new();
            for (view, source) in &self.peak_views {
                if variables_to_remove.contains(source) {
                    peak_views_to_remove.push(view.clone());
                }
            }
            for view in peak_views_to_remove {
                self.peak_views.remove(&view);
            }
        }
    }
    
    /// Create a shared write relationship between variables
    pub fn share_write(&mut self, source: &str, target: &str) -> Result<(), String> {
        // Ensure both variables exist
        if self.get_variable(source).is_none() {
            return Err(format!("Source variable {} not found", source));
        }
        if self.get_variable(target).is_none() {
            return Err(format!("Target variable {} not found", target));
        }
        
        // Establish shared write relationship between source and target
        self.shared_writes
            .entry(source.to_string())
            .or_insert_with(Vec::new)
            .push(target.to_string());
            
        self.shared_writes
            .entry(target.to_string())
            .or_insert_with(Vec::new)
            .push(source.to_string());
            
        Ok(())
    }
    
    /// Create a reference between variables
    pub fn create_reference(&mut self, target: &str, source: &str) -> Result<(), String> {
        self.share_write(source, target)
    }
    
    /// Create a peak view relationship
    pub fn create_peak_view(&mut self, source: &str, target: &str) -> Result<(), String> {
        // Ensure source variable exists
        if self.get_variable(source).is_none() {
            return Err(format!("Source variable {} not found", source));
        }
        
        // Register the peak view
        self.peak_views.insert(target.to_string(), source.to_string());
        
        Ok(())
    }
    
    /// Print debug information about the current memory state
    pub fn print_debug_info(&self) {
        println!("\nVariables:");
        
        // Print local scopes first (most recent at top)
        for (i, scope) in self.scopes.iter().enumerate() {
            println!("Scope {}:", i);
            for (name, value) in &scope.variables {
                println!("  {} = {}", name, value);
            }
        }
        
        // Print globals
        println!("Global scope:");
        for (name, value) in &self.global_variables {
            println!("  {} = {}", name, value);
        }
        
        println!("\nShared writes:");
        for (source, aliases) in &self.shared_writes {
            println!("{} shares writes with: {:?}", source, aliases);
        }
        
        println!("\nPeak views:");
        for (view, source) in &self.peak_views {
            println!("{} is a peak view of {}", view, source);
        }
    }
}