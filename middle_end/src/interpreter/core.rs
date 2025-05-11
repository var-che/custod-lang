//! Core interpreter functionality

use crate::mir::{MirFunction, MirInstruction, MirValue};
use std::collections::{HashMap, VecDeque};
use super::values::InterpreterValue;
use super::memory::{MemoryManager, VariableScope};
use super::functions::FunctionManager;

/// MIR code interpreter
#[derive(Debug)]
pub struct Interpreter {
    /// Memory management for variables and scopes
    pub memory: MemoryManager,
    
    /// Function registry and execution
    functions: FunctionManager,
    
    /// Temporaries used in computation (always local to current execution)
    temporaries: HashMap<usize, i64>,
}

impl Interpreter {
    /// Create a new interpreter instance
    pub fn new() -> Self {
        Self {
            memory: MemoryManager::new(),
            functions: FunctionManager::new(),
            temporaries: HashMap::new(),
        }
    }

    /// Get a variable's value from any scope
    pub fn get_variable(&self, name: &str) -> Option<i64> {
        self.memory.get_variable(name)
    }
    
    /// Execute the MIR program
    pub fn execute(&mut self, mir: &MirFunction) -> Result<i64, String> {
        self.execute_instructions(&mir.instructions)
    }
    
    /// Execute a list of MIR instructions
    pub fn execute_instructions(&mut self, instructions: &[MirInstruction]) -> Result<i64, String> {
        let mut last_value = 0;

        for instruction in instructions {
            match instruction {
                MirInstruction::Store { target, value } => {
                    last_value = self.handle_store(target, value)?;
                },
                MirInstruction::EnterScope => {
                    self.memory.enter_scope();
                },
                MirInstruction::ExitScope => {
                    self.memory.exit_scope();
                },
                MirInstruction::Load { target, value } => {
                    let val = self.evaluate_value(value)?;
                    self.temporaries.insert(*target, val);
                    last_value = val;
                },
                MirInstruction::Add { target, left, right } => {
                    last_value = self.handle_add(*target, left, right)?;
                },
                MirInstruction::Print { value } => {
                    let val = self.evaluate_value(value)?;
                    println!("{}", val);
                    last_value = val;
                },
                MirInstruction::CreateReference { target, source } => {
                    self.memory.create_reference(target, source)?;
                },
                MirInstruction::CreatePeakView { source, target } => {
                    self.memory.create_peak_view(source, target)?;
                },
                MirInstruction::ShareWrite { source, target } => {
                    self.memory.share_write(source, target)?;
                },
                MirInstruction::Call { function } => {
                    self.functions.handle_function_declaration(function);
                },
                MirInstruction::Return { value } => {
                    last_value = self.evaluate_value(value)?;
                },
                MirInstruction::ReadBarrier { .. } | MirInstruction::WriteBarrier { .. } => {
                    // Barriers are no-ops in the interpreter
                }
            }
        }

        Ok(last_value)
    }
    
    /// Handle store instruction
    fn handle_store(&mut self, target: &str, value: &MirValue) -> Result<i64, String> {
        let val = self.evaluate_value(value)?;
        self.memory.set_variable(target, val);
        Ok(val)
    }
    
    /// Handle add instruction 
    fn handle_add(&mut self, target: usize, left: &MirValue, right: &MirValue) -> Result<i64, String> {
        let l = self.evaluate_value(left)?;
        let r = self.evaluate_value(right)?;
        
        let result = l.checked_add(r)
            .ok_or_else(|| "Integer overflow during addition".to_string())?;
            
        self.temporaries.insert(target, result);
        Ok(result)
    }
    
    /// Evaluate a MIR value to a concrete value
    pub fn evaluate_value(&self, value: &MirValue) -> Result<i64, String> {
        match value {
            MirValue::Number(n) => Ok(*n),
            MirValue::Variable(name) => {
                self.memory.get_variable(name)
                    .ok_or_else(|| format!("Variable {} not found", name))
            },
            MirValue::Temporary(t) => {
                self.temporaries.get(t)
                    .copied()
                    .ok_or_else(|| format!("Temporary {} not found", t))
            },
        }
    }
    
    /// Print debugging information about variables
    pub fn print_variables(&self) {
        self.memory.print_debug_info();
    }

    /// Register a MIR function for later execution
    pub fn register_function(&mut self, name: String, function: MirFunction) {
        self.functions.register_function(name, function);
    }
}