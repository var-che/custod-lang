use crate::mir::{MirFunction, MirInstruction, MirValue};
use std::collections::HashMap;

#[derive(Debug)] 
pub struct Interpreter {
    variables: HashMap<String, i64>,
    temporaries: HashMap<usize, i64>,
    references: HashMap<String, String>,  // Track references (target -> source)
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            temporaries: HashMap::new(),
            references: HashMap::new(),
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<i64> {
        // If this is a reference, get the source variable's value
        if let Some(source) = self.references.get(name) {
            self.variables.get(source).copied()
        } else {
            self.variables.get(name).copied()
        }
    }

    fn store_reference(&mut self, target: String, source: String) {
        self.references.insert(target, source);
    }

    pub fn execute(&mut self, mir: &MirFunction) -> Result<i64, String> {
        let mut last_value = 0;

        for instruction in &mir.instructions {
            match instruction {
                MirInstruction::CreateReference { target, source } => {
                    // Store the reference relationship
                    self.store_reference(target.clone(), source.clone());
                    // Get the initial value
                    if let Some(val) = self.get_variable(source) {
                        last_value = val;
                    }
                },
                MirInstruction::Store { target, value } => {
                    let val = self.evaluate_value(value)?;
                    self.variables.insert(target.clone(), val);
                    last_value = val;
                },
                MirInstruction::Load { target, value } => {
                    let val = self.evaluate_value(value)?;
                    self.temporaries.insert(*target, val);
                    last_value = val;
                },
                MirInstruction::Add { target, left, right } => {
                    let l = self.evaluate_value(left)?;
                    let r = self.evaluate_value(right)?;
                    let result = l + r;
                    self.temporaries.insert(*target, result);
                    last_value = result;
                },
                MirInstruction::Print { value } => {
                    let val = self.evaluate_value(value)?;
                    println!("{}", val);
                    last_value = val;
                },
                MirInstruction::ReadBarrier { .. } | MirInstruction::WriteBarrier { .. } => {
                    // Barriers are no-ops in the interpreter
                }
            }
        }

        Ok(last_value)
    }

    fn evaluate_value(&self, value: &MirValue) -> Result<i64, String> {
        match value {
            MirValue::Number(n) => Ok(*n),
            MirValue::Variable(name) => {
                // Check references first, then variables
                if let Some(source) = self.references.get(name) {
                    self.variables.get(source)
                        .copied()
                        .ok_or_else(|| format!("Source variable {} not found", source))
                } else {
                    self.variables.get(name)
                        .copied()
                        .ok_or_else(|| format!("Variable {} not found", name))
                }
            },
            MirValue::Temporary(t) => {
                self.temporaries.get(t)
                    .copied()
                    .ok_or_else(|| format!("Temporary {} not found", t))
            }
        }
    }
}