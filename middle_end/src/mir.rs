use crate::hir::{HirProgram, HirStatement, HirValue, HirMethod};
use front_end::token::PermissionType;  
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum MirValue {
    Number(i64),
    Temporary(usize),
    Variable(String),    
}

#[derive(Debug, PartialEq, Clone)]
pub enum MirInstruction {
    // Load value into temporary
    Load { target: usize, value: MirValue },
    
    // Store value into named location
    Store { target: String, value: MirValue },
    
    // Arithmetic operations
    Add { 
        target: usize, 
        left: MirValue, 
        right: MirValue 
    },
    
    // Print value
    Print { value: MirValue },
    
    // Memory barriers for permissions
    ReadBarrier { reference: String },
    WriteBarrier { reference: String },

    // Create reference between variables
    CreateReference { target: String, source: String },

    // Share write tracking
    ShareWrite { source: String, target: String },

    // Create peak view between variables
    CreatePeakView { source: String, target: String },
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

    // First collect all function definitions
    let mut functions = HashMap::new();
    for stmt in &program.statements {
        if let HirStatement::Method(method) = stmt {
            functions.insert(method.name.clone(), method.clone());
        }
    }

    // Helper function to generate function call code
    fn generate_function_call(
        mir: &mut MirFunction,
        function_name: &str,
        arguments: &[HirValue],
        functions: &HashMap<String, HirMethod>,
        temp_counter: &mut usize,
    ) -> MirValue {
        // Find function
        if let Some(function) = functions.get(function_name) {
            // Create temporaries for arguments
            let mut arg_values = Vec::new();
            for arg in arguments {
                let arg_temp = *temp_counter;
                *temp_counter += 1;
                let arg_value = convert_hir_value(arg);
                mir.instructions.push(MirInstruction::Load {
                    target: arg_temp,
                    value: arg_value,
                });
                arg_values.push(MirValue::Temporary(arg_temp));
            }

            // Store arguments in parameter variables
            for (param, arg_value) in function.params.iter().zip(arg_values) {
                mir.instructions.push(MirInstruction::Store {
                    target: param.name.clone(),
                    value: arg_value,
                });
            }

            // Process function body statements
            for stmt in &function.body {
                match stmt {
                    HirStatement::Declaration(var) => {
                        if let Some(initializer) = &var.initializer {
                            match initializer {
                                HirValue::Call { function: nested_fn, arguments: nested_args, .. } => {
                                    // Generate code for nested function call
                                    let result = generate_function_call(
                                        mir, 
                                        nested_fn, 
                                        nested_args, 
                                        functions, 
                                        temp_counter
                                    );
                                    
                                    // Store result in variable
                                    mir.instructions.push(MirInstruction::Store {
                                        target: var.name.clone(),
                                        value: result,
                                    });
                                },
                                _ => {
                                    // Handle regular initializers
                                    let val = process_hir_value(mir, initializer, temp_counter);
                                    mir.instructions.push(MirInstruction::Store {
                                        target: var.name.clone(),
                                        value: val,
                                    });
                                }
                            }
                        }
                    },
                    HirStatement::Return(value) => {
                        match value {
                            HirValue::Call { function: nested_fn, arguments: nested_args, .. } => {
                                // For nested function calls in return statement
                                return generate_function_call(mir, nested_fn, nested_args, functions, temp_counter);
                            },
                            HirValue::Binary { left, right, operator, .. } => {
                                // Process binary operation in return
                                let left_val = process_hir_value(mir, left, temp_counter);
                                let right_val = process_hir_value(mir, right, temp_counter);
                                
                                let result_temp = *temp_counter;
                                *temp_counter += 1;
                                
                                mir.instructions.push(MirInstruction::Add {
                                    target: result_temp,
                                    left: left_val,
                                    right: right_val,
                                });
                                
                                return MirValue::Temporary(result_temp);
                            },
                            _ => {
                                // For simple values
                                return process_hir_value(mir, value, temp_counter);
                            }
                        }
                    },
                    _ => {} // Other statement types
                }
            }
            
            // Default return if no return statement found
            MirValue::Number(0)
        } else {
            panic!("Function {} not found", function_name)
        }
    }

    // Helper function to process HIR values into MIR values
    fn process_hir_value(
        mir: &mut MirFunction, 
        value: &HirValue, 
        temp_counter: &mut usize
    ) -> MirValue {
        match value {
            HirValue::Binary { left, right, .. } => {
                // Process binary operation
                let left_val = process_hir_value(mir, left, temp_counter);
                let right_val = process_hir_value(mir, right, temp_counter);
                
                let result_temp = *temp_counter;
                *temp_counter += 1;
                
                mir.instructions.push(MirInstruction::Add {
                    target: result_temp,
                    left: left_val,
                    right: right_val,
                });
                
                MirValue::Temporary(result_temp)
            },
            HirValue::Variable(name, _) => {
                // For variables, load into a temporary
                let temp = *temp_counter;
                *temp_counter += 1;
                
                mir.instructions.push(MirInstruction::Load {
                    target: temp,
                    value: MirValue::Variable(name.clone()),
                });
                
                MirValue::Temporary(temp)
            },
            _ => convert_hir_value(value)
        }
    }

    // Main statement processing
    for statement in &program.statements {
        match statement {
            HirStatement::Declaration(var) => {
                if let Some(init) = &var.initializer {
                    match init {
                        HirValue::Call { function, arguments, .. } => {
                            mir.instructions.push(MirInstruction::WriteBarrier {
                                reference: var.name.clone()
                            });
                            let result = generate_function_call(
                                &mut mir,
                                function,
                                arguments,
                                &functions,
                                &mut temp_counter
                            );
                            mir.instructions.push(MirInstruction::Store {
                                target: var.name.clone(),
                                value: result,
                            });
                        },
                        HirValue::Peak(expr) => {
                            if let HirValue::Variable(source_name, _) = &**expr {
                                mir.instructions.push(MirInstruction::CreatePeakView {
                                    source: source_name.clone(),
                                    target: var.name.clone(),
                                });
                            }
                        },
                        _ => {
                            // For write permission with writes source, create shared write relationship
                            if var.permissions.permissions.contains(&PermissionType::Write) {
                                if let HirValue::Variable(source_name, _) = init {
                                    mir.instructions.push(MirInstruction::ShareWrite {
                                        source: source_name.clone(),
                                        target: var.name.clone(),
                                    });
                                }
                            }

                            // Store the initial value
                            mir.instructions.push(MirInstruction::Store {
                                target: var.name.clone(),
                                value: convert_hir_value(init),
                            });
                        }
                    }
                }
            },
            HirStatement::Assignment(assign) => {
                // For assignment, update all shared writes
                mir.instructions.push(MirInstruction::WriteBarrier {
                    reference: assign.target.clone()
                });
                mir.instructions.push(MirInstruction::Store {
                    target: assign.target.clone(),
                    value: convert_hir_value(&assign.value),
                });
            },
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
        HirValue::Peak(expr) => convert_hir_value(expr),
        HirValue::Binary { left, operator: _, right, .. } => {
            // Instead of evaluating immediately, create a temporary
            let left_val = convert_hir_value(left);
            let right_val = convert_hir_value(right);
            
            // Return the left value for now (temporary fix)
            // Later we'll properly handle binary operations in the interpreter
            left_val
        },
        HirValue::Call { arguments, .. } => {
            // For now, evaluate the arguments and return their sum
            let first_arg = arguments.first()
                .map(convert_hir_value)
                .unwrap_or(MirValue::Number(0));
            
            first_arg
        },
        _ => panic!("Unsupported HIR value type"),
    }
}