//! MIR pretty printer
//!
//! This module provides functionality for pretty-printing MIR for debugging.

use crate::mir::types::*;
use std::fmt::Write;

/// Pretty-print a MIR program
pub fn pretty_print_program(program: &MirProgram) -> String {
    let mut output = String::new();
    
    // Print global variables
    if !program.globals.is_empty() {
        writeln!(&mut output, "// Global Variables").unwrap();
        for (name, var) in &program.globals {
            writeln!(&mut output, "var {}: {:?} [{}]", name, var.typ, var.id.0).unwrap();
        }
        writeln!(&mut output).unwrap();
    }
    
    // Print functions
    for (_, func) in &program.functions {
        pretty_print_function(func, &mut output);
        writeln!(&mut output).unwrap();
    }
    
    output
}

/// Pretty-print a MIR function
pub fn pretty_print_function(func: &MirFunction, output: &mut String) {
    // Print function signature
    write!(output, "fn {}(", func.name).unwrap();
    
    // Print parameters
    for (i, (var_id, typ)) in func.parameters.iter().enumerate() {
        if i > 0 {
            write!(output, ", ").unwrap();
        }
        
        let var_name = func.variables.get(var_id)
            .map(|v| v.name.as_str())
            .unwrap_or("unknown");
            
        write!(output, "{}: {:?} [{}]", var_name, typ, var_id.0).unwrap();
    }
    
    // Print return type
    if let Some(ret_type) = &func.return_type {
        writeln!(output, ") -> {:?} {{", ret_type).unwrap();
    } else {
        writeln!(output, ") {{").unwrap();
    }
    
    // Print local variables that aren't parameters
    let param_ids: std::collections::HashSet<_> = func.parameters.iter()
        .map(|(id, _)| *id)
        .collect();
        
    let locals: Vec<_> = func.variables.values()
        .filter(|var| !param_ids.contains(&var.id))
        .collect();
        
    if !locals.is_empty() {
        writeln!(output, "    // Local variables").unwrap();
        for var in locals {
            writeln!(output, "    var {}: {:?} [{}]", var.name, var.typ, var.id.0).unwrap();
        }
        writeln!(output).unwrap();
    }
    
    // Print blocks
    for block in &func.blocks {
        pretty_print_block(block, output, func);
    }
    
    writeln!(output, "}}").unwrap();
}

/// Pretty-print a basic block
fn pretty_print_block(block: &BasicBlock, output: &mut String, func: &MirFunction) {
    // Print block header
    writeln!(output, "    block {}:", block.id.0).unwrap();
    
    // Print instructions
    for instr in &block.instructions {
        writeln!(output, "        {}", pretty_print_instruction(instr, func)).unwrap();
    }
    
    writeln!(output).unwrap();
}

/// Pretty-print an instruction
fn pretty_print_instruction(instr: &Instruction, func: &MirFunction) -> String {
    match instr {
        Instruction::Assign { target, source } => {
            let target_name = get_var_name(*target, func);
            format!("{} = {}", target_name, pretty_print_operand(source, func))
        },
        
        Instruction::BinaryOp { target, left, op, right } => {
            let target_name = get_var_name(*target, func);
            let op_str = match op {
                BinaryOperation::Add => "+",
                BinaryOperation::Subtract => "-",
                BinaryOperation::Multiply => "*",
                BinaryOperation::Divide => "/",
                BinaryOperation::Remainder => "%",
                BinaryOperation::Equal => "==",
                BinaryOperation::NotEqual => "!=",
                BinaryOperation::LessThan => "<",
                BinaryOperation::LessThanEqual => "<=",
                BinaryOperation::GreaterThan => ">",
                BinaryOperation::GreaterThanEqual => ">=",
                BinaryOperation::And => "&&",
                BinaryOperation::Or => "||",
            };
            
            format!(
                "{} = {} {} {}", 
                target_name,
                pretty_print_operand(left, func),
                op_str,
                pretty_print_operand(right, func)
            )
        },
        
        Instruction::Call { target, function, arguments } => {
            let mut result = String::new();
            
            // Add target assignment if present
            if let Some(target_id) = target {
                let target_name = get_var_name(*target_id, func);
                write!(&mut result, "{} = ", target_name).unwrap();
            }
            
            // Add function call
            write!(&mut result, "call {}(", function).unwrap();
            
            // Add arguments
            for (i, arg) in arguments.iter().enumerate() {
                if i > 0 {
                    write!(&mut result, ", ").unwrap();
                }
                write!(&mut result, "{}", pretty_print_operand(arg, func)).unwrap();
            }
            
            write!(&mut result, ")").unwrap();
            result
        },
        
        Instruction::Return(operand) => {
            if let Some(op) = operand {
                format!("return {}", pretty_print_operand(op, func))
            } else {
                "return".to_string()
            }
        },
        
        Instruction::Jump(block_id) => {
            format!("jump block{}", block_id.0)
        },
        
        Instruction::Branch { condition, true_block, false_block } => {
            format!(
                "branch {} ? block{} : block{}", 
                pretty_print_operand(condition, func),
                true_block.0,
                false_block.0
            )
        },
        
        Instruction::Nop => {
            "nop".to_string()
        },
    }
}

/// Pretty-print an operand
fn pretty_print_operand(operand: &Operand, func: &MirFunction) -> String {
    match operand {
        Operand::Variable(var_id) => {
            get_var_name(*var_id, func)
        },
        
        Operand::Constant(constant) => {
            match constant {
                Constant::Integer(value) => value.to_string(),
                Constant::Boolean(value) => value.to_string(),
                Constant::String(value) => format!("\"{}\"", value),
            }
        },
    }
}

/// Get the name of a variable
fn get_var_name(var_id: VarId, func: &MirFunction) -> String {
    if let Some(var) = func.variables.get(&var_id) {
        format!("{}[{}]", var.name, var_id.0)
    } else {
        format!("var_{}", var_id.0)
    }
}
