use crate::hir::{HirProgram, HirStatement, HirValue};

#[derive(Debug, Clone, PartialEq)]
pub enum MirValue {
    Number(i64),
    Temporary(usize),
    Variable(String),    // For named variables like "counter"
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

                // Add initial store for the variable
                if let Some(init) = &var.initializer {
                    match init {
                        HirValue::Number(n, _) => {
                            // Store the number directly
                            mir.instructions.push(MirInstruction::Store {
                                target: var.name.clone(),
                                value: MirValue::Number(*n),
                            });
                        },
                        HirValue::Clone(expr) => {
                            // Load the source value
                            let source_temp = temp_counter;
                            mir.instructions.push(MirInstruction::Load {
                                target: source_temp,
                                value: convert_hir_value(expr),
                            });
                            
                            // Store it in the target
                            mir.instructions.push(MirInstruction::Store {
                                target: var.name.clone(),
                                value: MirValue::Temporary(source_temp),
                            });
                            
                            temp_counter += 1;
                        },
                        HirValue::Binary { left, right, operator, result_type } => {
                            // Load left operand
                            mir.instructions.push(MirInstruction::Load {
                                target: temp_counter,
                                value: convert_hir_value(left),
                            });
                            
                            // Load right operand
                            mir.instructions.push(MirInstruction::Load {
                                target: temp_counter + 1,
                                value: convert_hir_value(right),
                            });
                            
                            // Add them
                            mir.instructions.push(MirInstruction::Add {
                                target: temp_counter + 2,
                                left: MirValue::Temporary(temp_counter),
                                right: MirValue::Temporary(temp_counter + 1),
                            });
                            
                            // Store result
                            mir.instructions.push(MirInstruction::Store {
                                target: var.name.clone(),
                                value: MirValue::Temporary(temp_counter + 2),
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

                match &assign.value {
                    HirValue::Binary { left, right, .. } => {
                        // Load the left operand (counter)
                        mir.instructions.push(MirInstruction::Load {
                            target: temp_counter,
                            value: convert_hir_value(left),
                        });

                        // Add right value (should be 13, not 1)
                        mir.instructions.push(MirInstruction::Add {
                            target: temp_counter + 1,
                            left: MirValue::Temporary(temp_counter),
                            right: convert_hir_value(right),  // This will preserve the actual number
                        });

                        // Store the result back
                        mir.instructions.push(MirInstruction::Store {
                            target: assign.target.clone(),
                            value: MirValue::Temporary(temp_counter + 1),
                        });

                        temp_counter += 2;
                    },
                    _ => {}
                }
            }
            HirStatement::Print(value) => {
                // Add read barrier before accessing the value
                if let HirValue::Variable(name, _) = value {
                    mir.instructions.push(MirInstruction::ReadBarrier {
                        reference: name.clone()
                    });
                    mir.instructions.push(MirInstruction::Print {
                        value: MirValue::Variable(name.clone())
                    });
                }
            }
            _ => {}
        }
    }
    mir
}

// Helper function to convert HIR values to MIR values
fn convert_hir_value(value: &HirValue) -> MirValue {
    match value {
        HirValue::Number(n, _) => MirValue::Number(*n),
        HirValue::Variable(name, _) => MirValue::Variable(name.clone()),
        HirValue::Clone(expr) => convert_hir_value(expr),
        _ => panic!("Unsupported HIR value type"),
    }
}