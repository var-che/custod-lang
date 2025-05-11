//! MIR Function and Scope Handling
//!
//! This module provides utilities for working with MIR functions and scopes.

use crate::mir::types::{MirFunction, MirInstruction, MirValue};
use std::collections::{HashMap, HashSet};

/// Information about a function in MIR
#[derive(Debug)]
pub struct FunctionInfo {
    pub name: String,
    pub instruction_count: usize,
    pub variables: HashSet<String>,
    pub parameters: Vec<String>,  // Ordered list of parameters
    pub has_return: bool,
}

impl FunctionInfo {
    pub fn new(name: String) -> Self {
        FunctionInfo {
            name,
            instruction_count: 0,
            variables: HashSet::new(),
            parameters: Vec::new(),
            has_return: false,
        }
    }
}

/// Track scope information in a MIR function
pub fn analyze_scopes(function: &MirFunction) -> Vec<(usize, usize)> {
    let mut scopes = Vec::new();
    let mut scope_stack = Vec::new();
    
    for (i, instruction) in function.instructions.iter().enumerate() {
        match instruction {
            MirInstruction::EnterScope => {
                scope_stack.push(i);
            },
            MirInstruction::ExitScope => {
                if let Some(start) = scope_stack.pop() {
                    scopes.push((start, i));
                }
            },
            _ => {}
        }
    }
    
    scopes
}

/// Detect parameters in a MIR function
fn detect_parameters(instructions: &[MirInstruction]) -> Vec<String> {
    let mut parameters = Vec::new();
    
    // Look for Store instructions at the beginning of the function that use temporaries
    // These are most likely parameter initializations
    for instruction in instructions {
        match instruction {
            MirInstruction::Store { target, value } => {
                if let MirValue::Temporary(_) = value {
                    // This is likely a parameter
                    parameters.push(target.clone());
                } else {
                    // If we hit a store with a non-temporary value, we're
                    // likely done with parameter initialization
                    break;
                }
            },
            // Once we hit a non-Store instruction, we've moved past parameter initialization
            _ => break,
        }
    }
    
    parameters
}

/// Analyze a MIR function to extract useful information
pub fn analyze_function(function: &MirFunction) -> FunctionInfo {
    let mut variables = HashSet::new();
    let mut parameters = Vec::new();
    let mut has_return = false;
    
    // Find scope boundaries
    let scopes = analyze_scopes(function);
    
    if let Some((start, _)) = scopes.first() {
        // Parameters are variables stored immediately after entering the first scope
        parameters = detect_parameters(&function.instructions[start + 1..]);
        variables.extend(parameters.iter().cloned());
    }
    
    // Gather all variables and check for return
    for instruction in &function.instructions {
        match instruction {
            MirInstruction::Store { target, .. } => {
                variables.insert(target.clone());
            },
            MirInstruction::Return { .. } => {
                has_return = true;
            },
            _ => {}
        }
    }
    
    FunctionInfo {
        name: String::new(),
        variables,
        parameters,
        has_return,
        instruction_count: function.instructions.len(),
    }
}

/// Extract function information from a slice of instructions
fn extract_function_info(instructions: &[MirInstruction]) -> FunctionInfo {
    let mut variables = HashSet::new();
    let mut parameters = Vec::new();
    let mut has_return = false;

    for instruction in instructions {
        match instruction {
            MirInstruction::Store { target, .. } => {
                variables.insert(target.clone());
            },
            MirInstruction::Return { .. } => {
                has_return = true;
            },
            _ => {}
        }
    }

    FunctionInfo {
        name: String::new(),
        variables,
        parameters,
        has_return,
        instruction_count: instructions.len(),
    }
}

/// Find all functions defined in a MirFunction by analyzing scope patterns
pub fn find_functions(mir: &MirFunction) -> HashMap<String, FunctionInfo> {
    let mut functions = HashMap::new();
    let scopes = analyze_scopes(mir);
    
    // Scan for Call instructions and match them to scopes
    for i in 0..mir.instructions.len() {
        if let MirInstruction::Call { function } = &mir.instructions[i] {
            // Find the scope that starts right after this Call
            for &(start, end) in &scopes {
                if start == i + 1 {
                    // Found the function's scope
                    let function_slice = &mir.instructions[start..=end];
                    
                    // Extract function info including parameters
                    let mut info = extract_function_info(function_slice);
                    
                    // Detect parameters - add this line
                    info.parameters = detect_parameters(function_slice);
                    
                    info.name = function.clone();
                    functions.insert(function.clone(), info);
                    break;
                }
            }
        }
    }
    
    functions
}