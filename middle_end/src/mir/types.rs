//! Core MIR Type Definitions
//!
//! This module contains the fundamental data structures that represent the
//! Medium-level Intermediate Representation (MIR).

use std::fmt;

/// A value in the MIR representation
#[derive(Debug, Clone, PartialEq)]
pub enum MirValue {
    /// A literal number
    Number(i64),
    /// A reference to a temporary register
    Temporary(usize),
    /// A reference to a named variable
    Variable(String),
}

/// Permission types for metadata
#[derive(Debug, PartialEq, Clone)]
pub enum PermissionType {
    Read,
    Write,
}

/// An instruction in the MIR representation
#[derive(Debug, PartialEq, Clone)]
pub enum MirInstruction {
    // Memory operations
    /// Load a value into a temporary register
    Load { target: usize, value: MirValue },
    
    /// Store a value into a named location
    Store { 
        target: String, 
        value: MirValue,
    },
    
    // Arithmetic operations
    /// Add two values and store the result in a temporary
    Add { target: usize, left: MirValue, right: MirValue },
    
    // I/O operations
    /// Print a value to standard output
    Print { value: MirValue },
    
    // Memory barriers for permissions
    /// Create a read barrier for a variable
    ReadBarrier { reference: String },
    /// Create a write barrier for a variable
    WriteBarrier { reference: String },

    // Reference handling
    /// Create a reference between variables
    CreateReference { target: String, source: String },
    /// Share write permissions between variables
    ShareWrite { source: String, target: String },
    /// Create a peak view between variables
    CreatePeakView { source: String, target: String },
    
    // Scope management
    /// Enter a new scope
    EnterScope,
    /// Exit the current scope
    ExitScope,
    
    // Function operations
    /// Call a function
    Call { function: String },
    /// Return a value from a function
    Return { value: MirValue },
}

/// A MIR function contains a sequence of MIR instructions
#[derive(Debug, PartialEq, Clone)]
pub struct MirFunction {
    /// The instructions that make up the function
    pub instructions: Vec<MirInstruction>,
}

impl MirFunction {
    /// Create a new, empty MIR function
    pub fn new() -> Self {
        MirFunction {
            instructions: Vec::new()
        }
    }

    /// Add an instruction to the function
    pub fn push(&mut self, instruction: MirInstruction) {
        self.instructions.push(instruction);
    }

    /// Get the number of instructions in the function
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    /// Return true if the function has no instructions
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }
}

impl fmt::Display for MirInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load { target, value } => write!(f, "t{} = {}", target, value),
            Self::Store { target, value } => {
                write!(f, "{} = {}", target, value)
            }
            Self::Add { target, left, right } => write!(f, "t{} = {} + {}", target, left, right),
            Self::Print { value } => write!(f, "print {}", value),
            Self::ReadBarrier { reference } => write!(f, "read_barrier {}", reference),
            Self::WriteBarrier { reference } => write!(f, "write_barrier {}", reference),
            Self::CreateReference { target, source } => write!(f, "{} -> {}", target, source),
            Self::ShareWrite { source, target } => write!(f, "share_write {} -> {}", source, target),
            Self::CreatePeakView { source, target } => write!(f, "{} = peak {}", target, source),
            Self::EnterScope => write!(f, "enter_scope"),
            Self::ExitScope => write!(f, "exit_scope"),
            Self::Call { function } => write!(f, "call {}", function),
            Self::Return { value } => write!(f, "return {}", value),
        }
    }
}

impl fmt::Display for MirValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Temporary(t) => write!(f, "t{}", t),
            Self::Variable(name) => write!(f, "{}", name),
        }
    }
}

impl fmt::Display for MirFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "MIR Function:")?;
        for (i, inst) in self.instructions.iter().enumerate() {
            writeln!(f, "  {}: {}", i, inst)?;
        }
        Ok(())
    }
}