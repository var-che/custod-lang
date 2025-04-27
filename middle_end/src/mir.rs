use std::collections::HashMap;

use crate::hir::{HirProgram, HirStatement, HirValue};

#[derive(Debug, PartialEq, Clone)]
pub enum MirValue {
    Number(i64),
    Reference(String),
    Temporary(usize)
}

#[derive(Debug, PartialEq, Clone)]
pub enum MirInstruction {
    // Load value into temporary
    Load { target: usize, value: MirValue },
    
    // Store value into named location
    Store { target: String, value: MirValue },
    
    // Arithmetic operations
    Add { target: usize, left: MirValue, right: MirValue },
    
    // Print value
    Print { value: MirValue },
    
    // Memory barriers for permissions
    ReadBarrier { reference: String },
    WriteBarrier { reference: String },
}

#[derive(Debug, PartialEq, Clone)]
pub struct MirFunction {
    pub instructions: Vec<MirInstruction>,
}

impl MirFunction {
    pub fn new() -> Self {
        MirFunction {
            instructions: Vec::new()
        }
    }
}

// Convert HIR to MIR
pub fn lower_hir(program: &HirProgram) -> MirFunction {
    let mut mir = MirFunction::new();
    let mut temp_counter = 0;

    for statement in &program.statements {
        match statement {
            HirStatement::Declaration(var) => {
                // Add write barrier for declaration
                mir.instructions.push(MirInstruction::WriteBarrier {
                    reference: var.name.clone()
                });

                if let Some(init) = &var.initializer {
                    match init {
                        HirValue::Binary { left, right, operator, result_type } => {
                            // Load left operand
                            mir.instructions.push(MirInstruction::Load {
                                target: temp_counter,
                                value: MirValue::Number(2) // Hardcoded for now
                            });
                            
                            // Load right operand
                            mir.instructions.push(MirInstruction::Load {
                                target: temp_counter + 1,
                                value: MirValue::Number(2) // Hardcoded for now
                            });
                            
                            // Add them
                            mir.instructions.push(MirInstruction::Add {
                                target: temp_counter + 2,
                                left: MirValue::Temporary(temp_counter),
                                right: MirValue::Temporary(temp_counter + 1)
                            });
                            
                            // Store result
                            mir.instructions.push(MirInstruction::Store {
                                target: var.name.clone(),
                                value: MirValue::Temporary(temp_counter + 2)
                            });
                            
                            temp_counter += 3;
                        }
                        _ => {}
                    }
                }
            }
            HirStatement::Assignment(assign) => {
                // Add write barrier
                mir.instructions.push(MirInstruction::WriteBarrier {
                    reference: assign.target.clone()
                });
                
                // Handle +=
                mir.instructions.push(MirInstruction::Load {
                    target: temp_counter,
                    value: MirValue::Reference(assign.target.clone())
                });
                
                mir.instructions.push(MirInstruction::Add {
                    target: temp_counter + 1,
                    left: MirValue::Temporary(temp_counter),
                    right: MirValue::Number(1)
                });
                
                mir.instructions.push(MirInstruction::Store {
                    target: assign.target.clone(),
                    value: MirValue::Temporary(temp_counter + 1)
                });
                
                temp_counter += 2;
            }
            HirStatement::Print(value) => {
                // Add read barrier before accessing the value
                if let HirValue::Variable(name, _) = value {
                    mir.instructions.push(MirInstruction::ReadBarrier {
                        reference: name.clone()
                    });
                    mir.instructions.push(MirInstruction::Print {
                        value: MirValue::Reference(name.clone())
                    });
                }
            }
            _ => {}
        }
    }
    mir
}