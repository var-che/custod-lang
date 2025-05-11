//! Function management and execution
//!
//! Handles function registration, lookup, and invocation.

use crate::mir::{MirFunction, MirInstruction, MirValue};
use std::collections::HashMap;

/// Context for a function
#[derive(Debug, Clone)]
pub struct FunctionContext {
    /// Name of the function
    pub name: String,
    
    /// MIR code for the function
    pub code: MirFunction,
    
    /// Parameter names
    pub parameters: Vec<String>,
}

/// Manages function registration and execution
#[derive(Debug, Default)]
pub struct FunctionManager {
    /// Registered functions
    functions: HashMap<String, FunctionContext>,
}

impl FunctionManager {
    /// Create a new function manager
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }
    
    /// Register a function
    pub fn register_function(&mut self, name: String, function: MirFunction) {
        // Extract parameters
        let parameters = Self::extract_parameters(&function);
        
        let context = FunctionContext {
            name: name.clone(),
            code: function,
            parameters,
        };
        
        self.functions.insert(name, context);
    }
    
    /// Handle a function declaration instruction
    pub fn handle_function_declaration(&self, function_name: &str) {
        // For now, we just acknowledge the declaration but don't process it
        // Functions are properly executed when called, not when declared
        println!("Function declaration: {}", function_name);
    }
    
    /// Execute a function by name with given arguments
    pub fn execute_function(
        &self,
        name: &str,
        arguments: &[i64],
        interpreter: &mut super::core::Interpreter,
    ) -> Result<i64, String> {
        // Find the function
        let function = self.functions.get(name)
            .ok_or_else(|| format!("Function {} not found", name))?;
        
        // Enter function scope
        interpreter.memory.enter_scope();
        
        // Bind parameters to arguments
        for (i, param_name) in function.parameters.iter().enumerate() {
            if i < arguments.len() {
                interpreter.memory.set_variable(param_name, arguments[i]);
            } else {
                return Err(format!("Missing argument {} for function {}", i, name));
            }
        }
        
        // Execute function body
        let result = interpreter.execute_instructions(&function.code.instructions)?;
        
        // Exit function scope
        interpreter.memory.exit_scope();
        
        Ok(result)
    }
    
    /// Extract parameter names from function code
    fn extract_parameters(function: &MirFunction) -> Vec<String> {
        // Analyze instruction sequence to identify parameter stores
        let mut parameters = Vec::new();
        
        // Look for initial Store instructions with Temporary arguments
        for inst in &function.instructions {
            if let MirInstruction::Store { target, value } = inst {
                if let MirValue::Temporary(_) = value {
                    // Parameter initialization
                    parameters.push(target.clone());
                } else {
                    // If we find a store with a non-temporary value, 
                    // we're out of parameter initialization
                    break;
                }
            } else if !parameters.is_empty() {
                // If we've started finding parameters and hit a non-store instruction,
                // we're done with parameters
                break;
            }
        }
        
        parameters
    }
}