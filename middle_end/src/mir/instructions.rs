//! MIR Instruction Analysis and Optimization
//!
//! This module provides utilities for analyzing and optimizing MIR instructions.

use crate::mir::types::{MirInstruction, MirValue, MirFunction};
use std::collections::HashSet;

/// Find all variables referenced in a MIR function
pub fn find_referenced_variables(function: &MirFunction) -> HashSet<String> {
    let mut variables = HashSet::new();
    
    for instruction in &function.instructions {
        match instruction {
            MirInstruction::Load { value: MirValue::Variable(name), .. } => {
                variables.insert(name.clone());
            },
            MirInstruction::Store { target, .. } => {
                variables.insert(target.clone());
            },
            MirInstruction::ReadBarrier { reference } |
            MirInstruction::WriteBarrier { reference } => {
                variables.insert(reference.clone());
            },
            MirInstruction::CreateReference { target, source } |
            MirInstruction::ShareWrite { target, source } |
            MirInstruction::CreatePeakView { target, source } => {
                variables.insert(target.clone());
                variables.insert(source.clone());
            },
            MirInstruction::Print { value: MirValue::Variable(name) } => {
                variables.insert(name.clone());
            },
            _ => {}
        }
    }
    
    variables
}

/// Check if a variable is read in the given MIR function
pub fn is_variable_read(function: &MirFunction, variable: &str) -> bool {
    for instruction in &function.instructions {
        match instruction {
            MirInstruction::Load { value: MirValue::Variable(name), .. } if name == variable => {
                return true;
            },
            MirInstruction::ReadBarrier { reference } if reference == variable => {
                return true;
            },
            MirInstruction::Print { value: MirValue::Variable(name) } if name == variable => {
                return true;
            },
            _ => {}
        }
    }
    
    false
}

/// Check if a variable is written in the given MIR function
pub fn is_variable_written(function: &MirFunction, variable: &str) -> bool {
    for instruction in &function.instructions {
        match instruction {
            MirInstruction::Store { target, .. } if target == variable => {
                return true;
            },
            MirInstruction::WriteBarrier { reference } if reference == variable => {
                return true;
            },
            _ => {}
        }
    }
    
    false
}