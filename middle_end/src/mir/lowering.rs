//! HIR to MIR Lowering
//!
//! This module handles the conversion from HIR to MIR representation,
//! translating higher-level constructs into simpler instructions.

use crate::hir::{HirProgram, HirStatement, HirValue, HirMethod};
use crate::mir::types::{MirFunction, MirInstruction, MirValue};
use crate::mir::values::convert_hir_value;
use front_end::token::PermissionType;
use std::collections::HashMap;

/// Convert an entire HIR program to MIR representation
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

    // Process all top-level statements
    for statement in &program.statements {
        match statement {
            HirStatement::Declaration(var) => {
                lower_declaration(&mut mir, var, &functions, &mut temp_counter);
            },
            HirStatement::Assignment(assign) => {
                lower_assignment(&mut mir, assign, &mut temp_counter);
            },
            HirStatement::Print(value) => {
                lower_print(&mut mir, value);
            },
            _ => {} // Other statement types handled elsewhere
        }
    }
    
    mir
}

/// Lower an HIR declaration to MIR instructions
fn lower_declaration(
    mir: &mut MirFunction,
    var: &crate::hir::HirVariable,
    functions: &HashMap<String, HirMethod>,
    temp_counter: &mut usize,
) {
    // Make sure to always add a WriteBarrier before storing to a variable
    mir.push(MirInstruction::WriteBarrier {
        reference: var.name.clone()
    });

    if let Some(init) = &var.initializer {
        match init {
            HirValue::Call { function, arguments, .. } => {
                // The WriteBarrier is already added above
                let result = generate_function_call(
                    mir,
                    function,
                    arguments,
                    functions,
                    temp_counter
                );
                mir.push(MirInstruction::Store {
                    target: var.name.clone(),
                    value: result,
                });
            },
            HirValue::Peak(expr) => {
                if let HirValue::Variable(source_name, _) = &**expr {
                    mir.push(MirInstruction::CreatePeakView {
                        source: source_name.clone(),
                        target: var.name.clone(),
                    });
                }
            },
            HirValue::Binary { left, right, .. } => {
                // Process binary operation and store result
                let left_val = process_hir_value(mir, left, temp_counter);
                let right_val = process_hir_value(mir, right, temp_counter);
                
                let result_temp = *temp_counter;
                *temp_counter += 1;
                
                mir.push(MirInstruction::Add {
                    target: result_temp,
                    left: left_val,
                    right: right_val,
                });
                
                mir.push(MirInstruction::Store {
                    target: var.name.clone(),
                    value: MirValue::Temporary(result_temp),
                });
            },
            _ => {
                // For write permission with writes source, create shared write relationship
                if var.permissions.permissions.contains(&PermissionType::Write) {
                    if let HirValue::Variable(source_name, _) = init {
                        mir.push(MirInstruction::ShareWrite {
                            source: source_name.clone(),
                            target: var.name.clone(),
                        });
                    }
                }

                // Store the initial value
                mir.push(MirInstruction::Store {
                    target: var.name.clone(),
                    value: convert_hir_value(init),
                });
            }
        }
    }
}

/// Lower an HIR assignment to MIR instructions
fn lower_assignment(
    mir: &mut MirFunction, 
    assign: &crate::hir::HirAssignment, 
    temp_counter: &mut usize
) {
    // For assignment, update all shared writes
    mir.push(MirInstruction::WriteBarrier {
        reference: assign.target.clone()
    });
    
    // Process the assignment value - this is the key change
    match &assign.value {
        HirValue::Binary { left, right, .. } => {
            // Evaluate left side
            let left_temp = *temp_counter;
            *temp_counter += 1;
            if let HirValue::Variable(name, _) = &**left {
                mir.push(MirInstruction::Load {
                    target: left_temp,
                    value: MirValue::Variable(name.clone()),
                });
            } else {
                mir.push(MirInstruction::Load {
                    target: left_temp,
                    value: convert_hir_value(left),
                });
            }
            
            // Evaluate right side
            let right_temp = *temp_counter;
            *temp_counter += 1;
            mir.push(MirInstruction::Load {
                target: right_temp,
                value: convert_hir_value(right),
            });
            
            // Compute result
            let result_temp = *temp_counter;
            *temp_counter += 1;
            mir.push(MirInstruction::Add {
                target: result_temp,
                left: MirValue::Temporary(left_temp),
                right: MirValue::Temporary(right_temp),
            });
            
            // Store result
            mir.push(MirInstruction::Store {
                target: assign.target.clone(),
                value: MirValue::Temporary(result_temp),
            });
        },
        _ => {
            // Default case for non-binary expressions
            mir.push(MirInstruction::Store {
                target: assign.target.clone(),
                value: convert_hir_value(&assign.value),
            });
        }
    }
}

/// Lower an HIR print statement to MIR instructions
fn lower_print(mir: &mut MirFunction, value: &HirValue) {
    if let HirValue::Variable(name, _) = value {
        mir.push(MirInstruction::ReadBarrier {
            reference: name.clone()
        });
        mir.push(MirInstruction::Print {
            value: MirValue::Variable(name.clone())
        });
    }
}

/// Generate MIR code for a function call
pub fn generate_function_call(
    mir: &mut MirFunction,
    function_name: &str,
    arguments: &[HirValue],
    functions: &HashMap<String, HirMethod>,
    temp_counter: &mut usize,
) -> MirValue {
    // Find function
    if let Some(function) = functions.get(function_name) {
        // Enter new scope for function
        mir.push(MirInstruction::EnterScope);
        
        // Create temporaries for arguments
        let mut arg_values = Vec::new();
        for arg in arguments {
            let arg_temp = *temp_counter;
            *temp_counter += 1;
            let arg_value = convert_hir_value(arg);
            mir.push(MirInstruction::Load {
                target: arg_temp,
                value: arg_value,
            });
            arg_values.push(MirValue::Temporary(arg_temp));
        }

        // Store arguments in parameter variables
        for (param, arg_value) in function.params.iter().zip(arg_values) {
            mir.push(MirInstruction::Store {
                target: param.name.clone(),
                value: arg_value,
            });
        }

        // Process function body statements
        process_function_body(mir, function, functions, temp_counter)
    } else {
        panic!("Function {} not found", function_name)
    }
}

/// Process a function body to generate MIR instructions
fn process_function_body(
    mir: &mut MirFunction,
    function: &HirMethod,
    functions: &HashMap<String, HirMethod>,
    temp_counter: &mut usize,
) -> MirValue {
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
                            mir.push(MirInstruction::Store {
                                target: var.name.clone(),
                                value: result,
                            });
                        },
                        _ => {
                            // Handle regular initializers
                            let val = process_hir_value(mir, initializer, temp_counter);
                            mir.push(MirInstruction::Store {
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
                        let result = generate_function_call(mir, nested_fn, nested_args, functions, temp_counter);
                        mir.push(MirInstruction::ExitScope);
                        return result;
                    },
                    HirValue::Binary { left, right, .. } => {
                        // Process binary operation in return
                        let left_val = process_hir_value(mir, left, temp_counter);
                        let right_val = process_hir_value(mir, right, temp_counter);
                        
                        let result_temp = *temp_counter;
                        *temp_counter += 1;
                        
                        mir.push(MirInstruction::Add {
                            target: result_temp,
                            left: left_val,
                            right: right_val,
                        });
                        
                        mir.push(MirInstruction::ExitScope);
                        return MirValue::Temporary(result_temp);
                    },
                    _ => {
                        // For simple values
                        let result = process_hir_value(mir, value, temp_counter);
                        mir.push(MirInstruction::ExitScope);
                        return result;
                    }
                }
            },
            _ => {} // Other statement types
        }
    }
    
    // Prepare default return value if no explicit return was found
    let result_temp = *temp_counter;
    *temp_counter += 1;
    
    // Try to extract return statement
    if let Some(HirStatement::Return(value)) = function.body.last() {
        let return_value = process_hir_value(mir, value, temp_counter);
        
        mir.push(MirInstruction::Load {
            target: result_temp,
            value: return_value,
        });
    } else {
        // Default return of 0 if no return statement
        mir.push(MirInstruction::Load {
            target: result_temp,
            value: MirValue::Number(0),
        });
    }
    
    // Exit scope before returning
    mir.push(MirInstruction::ExitScope);
    
    MirValue::Temporary(result_temp)
}

/// Process an HIR value to generate MIR value with necessary instructions
pub fn process_hir_value(
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
            
            mir.push(MirInstruction::Add {
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
            
            mir.push(MirInstruction::Load {
                target: temp,
                value: MirValue::Variable(name.clone()),
            });
            
            MirValue::Temporary(temp)
        },
        _ => convert_hir_value(value)
    }
}