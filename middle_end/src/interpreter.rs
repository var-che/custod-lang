use crate::mir::{MirFunction, MirInstruction, MirValue};
use std::collections::HashMap;

#[derive(Debug)] 
pub struct Interpreter {
    variables: HashMap<String, i64>,
    temporaries: HashMap<usize, i64>,
    shared_writes: HashMap<String, Vec<String>>,  // Track shared write aliases
    peak_views: HashMap<String, String>,  // Track peak view relationships
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            temporaries: HashMap::new(),
            shared_writes: HashMap::new(),
            peak_views: HashMap::new(),
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<i64> {
        // First check if this is a peak view
        if let Some(source) = self.peak_views.get(name) {
            // Return the source's current value
            return self.variables.get(source).copied();
        }
        // Otherwise return direct value
        self.variables.get(name).copied()
    }

    pub fn execute(&mut self, mir: &MirFunction) -> Result<i64, String> {
        let mut last_value = 0;

        for instruction in &mir.instructions {
            match instruction {
                MirInstruction::Store { target, value } => {
                    let val = self.evaluate_value(value)?;
                    
                    // Update the target
                    self.variables.insert(target.clone(), val);
                    
                    // Update all variables that share write access with the target
                    if let Some(aliases) = self.shared_writes.get(target) {
                        for alias in aliases.clone() {  // Clone to avoid borrowing issues
                            if alias != *target {  // Skip self-reference
                                self.variables.insert(alias, val);
                            }
                        }
                    }
                    
                    last_value = val;
                },
                MirInstruction::ShareWrite { source, target } => {
                    // Add bidirectional write sharing
                    self.shared_writes
                        .entry(source.clone())
                        .or_insert_with(Vec::new)
                        .push(target.clone());
                    self.shared_writes
                        .entry(target.clone())
                        .or_insert_with(Vec::new)
                        .push(source.clone());
                },
                MirInstruction::Load { target, value } => {
                    let val = self.evaluate_value(value)?;
                    self.temporaries.insert(*target, val);
                    last_value = val;
                },
                MirInstruction::Add { target, left, right } => {
                    let l = self.evaluate_value(left)?;
                    let r = self.evaluate_value(right)?;
                    // Convert values to the same type before addition
                    let result = l.checked_add(r).ok_or_else(|| "Integer overflow during addition".to_string())?;
                    self.temporaries.insert(target.clone(), result);
                    last_value = result;
                },
                MirInstruction::Print { value } => {
                    let val = self.evaluate_value(value)?;
                    println!("{}", val);
                    last_value = val;
                },
                MirInstruction::CreateReference { target, source } => {
                    // Add target to source's write aliases
                    self.shared_writes
                        .entry(source.clone())
                        .or_insert_with(Vec::new)
                        .push(target.clone());
                    // Add source to target's write aliases
                    self.shared_writes
                        .entry(target.clone())
                        .or_insert_with(Vec::new)
                        .push(source.clone());
                },
                MirInstruction::CreatePeakView { source, target } => {
                    // Store the view relationship
                    self.peak_views.insert(target.clone(), source.clone());
                    // We shouldn't initialize the peak view with an actual value
                    // since it should always reflect the source's current value
                },
                MirInstruction::ReadBarrier { .. } | MirInstruction::WriteBarrier { .. } => {
                    // Barriers are no-ops in the interpreter
                }
            }
        }

        Ok(last_value)
    }

    pub fn print_variables(&self) {
        println!("\nVariables:");
        for (name, value) in &self.variables {
            println!("{} = {}", name, value);
        }
        println!("\nShared writes:");
        for (source, aliases) in &self.shared_writes {
            println!("{} shares writes with: {:?}", source, aliases);
        }
    }

    fn evaluate_value(&self, value: &MirValue) -> Result<i64, String> {
        match value {
            MirValue::Number(n) => Ok(*n),
            MirValue::Variable(name) => {
                // First check if this is a peak view
                if let Some(source) = self.peak_views.get(name) {
                    return self.variables.get(source)
                        .copied()
                        .ok_or_else(|| format!("Source variable {} not found", source));
                }
                // Otherwise get direct value
                self.variables.get(name)
                    .copied()
                    .ok_or_else(|| format!("Variable {} not found", name))
            },
            MirValue::Temporary(t) => {
                self.temporaries.get(t)
                    .copied()
                    .ok_or_else(|| format!("Temporary {} not found", t))
            },
        }
    }
}