//! HIR to MIR conversion
//!
//! This module provides the functionality to convert HIR to MIR.

use crate::hir::types::{HirProgram, HirStatement, HirExpression};
use front_end::token::TokenType; // Import TokenType which might be used as the binary operator
use crate::mir::types::*;
use std::collections::HashMap;

/// Convert a HIR program to a MIR program
pub fn convert_hir_to_mir(hir: &HirProgram) -> MirProgram {
    let mut converter = HirToMirConverter::new();
    converter.convert_program(hir)
}

/// Converter for transforming HIR to MIR
struct HirToMirConverter {
    /// The MIR program being built
    mir: MirProgram,
    
    /// Maps HIR variable names to MIR variable IDs
    var_map: HashMap<String, VarId>,
    
    /// Current function being converted
    current_function: Option<MirFunction>,
    
    /// Current block being filled
    current_block: Option<BasicBlock>,
}

impl HirToMirConverter {
    /// Create a new HIR to MIR converter
    pub fn new() -> Self {
        Self {
            mir: MirProgram::new(),
            var_map: HashMap::new(),
            current_function: None,
            current_block: None,
        }
    }
    
    /// Convert a HIR program to a MIR program
    pub fn convert_program(&mut self, hir: &HirProgram) -> MirProgram {
        // First collect all global variables
        for stmt in &hir.statements {
            if let HirStatement::Declaration(var) = stmt {
                // Create a MIR variable for the global
                let var_id = self.mir.new_var_id();
                let mir_var = MirVariable {
                    id: var_id,
                    name: var.name.clone(),
                    typ: var.typ.clone(),
                };
                
                // Add to globals and variable mapping
                self.mir.globals.insert(var.name.clone(), mir_var);
                self.var_map.insert(var.name.clone(), var_id);
                
                // If there's an initializer, we'll handle it in a special init function
                if var.initializer.is_some() {
                    // TODO: Handle global initializers
                }
            }
        }
        
        // Then convert all functions
        for stmt in &hir.statements {
            if let HirStatement::Function(func) = stmt {
                let mir_func = self.convert_function(func);
                self.mir.functions.insert(func.name.clone(), mir_func);
            }
        }
        
        self.mir.clone()
    }
    
    /// Convert a HIR function to a MIR function
    fn convert_function(&mut self, func: &crate::hir::types::HirFunction) -> MirFunction {
        // Create a new MIR function
        let mut mir_func = MirFunction {
            name: func.name.clone(),
            parameters: Vec::new(),
            return_type: func.return_type.clone(),
            blocks: Vec::new(),
            entry_block: BlockId(0), // Will be set correctly below
            variables: HashMap::new(),
        };
        
        // Set as current function
        self.current_function = Some(mir_func);
        
        // Create entry block
        let entry_id = self.mir.new_block_id();
        let entry_block = BasicBlock {
            id: entry_id,
            instructions: Vec::new(),
        };
        
        // Set as current block
        self.current_block = Some(entry_block);
        
        // Set entry block ID
        if let Some(ref mut func) = self.current_function {
            func.entry_block = entry_id;
        }
        
        // Convert parameters
        for param in &func.parameters {
            let var_id = self.mir.new_var_id();
            
            // Create MIR variable
            let mir_var = MirVariable {
                id: var_id,
                name: param.name.clone(),
                typ: param.typ.clone(),
            };
            
            // Add to function variables and parameters
            if let Some(ref mut func) = self.current_function {
                func.variables.insert(var_id, mir_var.clone());
                func.parameters.push((var_id, param.typ.clone()));
            }
            
            // Update variable mapping
            self.var_map.insert(param.name.clone(), var_id);
        }
        
        // Convert function body
        for stmt in &func.body {
            self.convert_statement(stmt);
        }
        
        // Make sure the function returns if it doesn't already
        if let Some(ref mut block) = self.current_block {
            if block.instructions.is_empty() || !matches!(block.instructions.last(), Some(Instruction::Return(_))) {
                block.instructions.push(Instruction::Return(None));
            }
        }
        
        // Finalize function
        let mut func = self.current_function.take().unwrap();
        if let Some(block) = self.current_block.take() {
            func.blocks.push(block);
        }
        
        func
    }
    
    /// Convert a HIR statement to MIR instructions
    fn convert_statement(&mut self, stmt: &HirStatement) {
        match stmt {
            HirStatement::Declaration(var) => {
                // Create a MIR variable
                let var_id = self.mir.new_var_id();
                let mir_var = MirVariable {
                    id: var_id,
                    name: var.name.clone(),
                    typ: var.typ.clone(),
                };
                
                // Add to function variables
                if let Some(ref mut func) = self.current_function {
                    func.variables.insert(var_id, mir_var);
                }
                
                // Update variable mapping
                self.var_map.insert(var.name.clone(), var_id);
                
                // If there's an initializer, convert it
                if let Some(ref init) = var.initializer {
                    let operand = self.convert_expression(init);
                    self.add_instruction(Instruction::Assign {
                        target: var_id,
                        source: operand,
                    });
                }
            },
            
            HirStatement::Assignment(assign) => {
                // Get the target variable ID
                if let Some(&var_id) = self.var_map.get(&assign.target) {
                    // Convert the value expression
                    let operand = self.convert_expression(&assign.value);
                    
                    // Add assignment instruction
                    self.add_instruction(Instruction::Assign {
                        target: var_id,
                        source: operand,
                    });
                }
            },
            
            HirStatement::Return(expr_opt) => {
                // Convert the return expression if any
                let operand = expr_opt.as_ref().map(|expr| self.convert_expression(expr));
                
                // Add return instruction
                self.add_instruction(Instruction::Return(operand));
            },
            
            // Handle other statement types as needed
            _ => {
                // Add a no-op for now
                self.add_instruction(Instruction::Nop);
            }
        }
    }
    
    /// Convert a HIR expression to a MIR operand
    fn convert_expression(&mut self, expr: &HirExpression) -> Operand {
        match expr {
            HirExpression::Integer(value, _) => {
                // Simple integer constant
                Operand::Constant(Constant::Integer(*value))
            },
            
            HirExpression::Boolean(value) => {
                // Simple boolean constant
                Operand::Constant(Constant::Boolean(*value))
            },
            
            HirExpression::String(value) => {
                // Simple string constant
                Operand::Constant(Constant::String(value.clone()))
            },
            
            HirExpression::Variable(name, _, _) => {
                // Look up the variable ID
                if let Some(&var_id) = self.var_map.get(name) {
                    Operand::Variable(var_id)
                } else {
                    // Unknown variable, this shouldn't happen if HIR is valid
                    panic!("Unknown variable: {}", name);
                }
            },
            
            HirExpression::Binary { left, operator, right, result_type, .. } => {
                // Convert the operands
                let left_operand = self.convert_expression(left);
                let right_operand = self.convert_expression(right);
                
                // Create a temporary variable for the result
                let result_id = self.mir.new_var_id();
                let result_var = MirVariable {
                    id: result_id,
                    name: format!("temp_{}", result_id.0),
                    typ: result_type.clone(), // Use the result_type directly from the expression
                };
                
                // Add the variable to the function
                if let Some(ref mut func) = self.current_function {
                    func.variables.insert(result_id, result_var);
                }
                
                // Convert the operator using TokenType instead of BinaryOperator
                let mir_op = match operator {
                    TokenType::Plus => BinaryOperation::Add,
                    TokenType::Minus => BinaryOperation::Subtract,
                    TokenType::Star => BinaryOperation::Multiply,
                    TokenType::Slash => BinaryOperation::Divide,
                    TokenType::EqualEqual => BinaryOperation::Equal,
                    TokenType::BangEqual => BinaryOperation::NotEqual,
                    TokenType::Less => BinaryOperation::LessThan,
                    TokenType::LessEqual => BinaryOperation::LessThanEqual,
                    TokenType::Greater => BinaryOperation::GreaterThan,
                    TokenType::GreaterEqual => BinaryOperation::GreaterThanEqual,
                    // Any remaining operators
                    _ => {
                        println!("Warning: Unsupported binary operator encountered in MIR conversion");
                        BinaryOperation::Add // Default fallback
                    }
                };
                
                // Add the binary operation instruction
                self.add_instruction(Instruction::BinaryOp {
                    target: result_id,
                    left: left_operand,
                    op: mir_op,
                    right: right_operand,
                });
                
                // Return a reference to the result variable
                Operand::Variable(result_id)
            },
            
            // Handle other expression types as needed
            _ => {
                // Default to a dummy constant for now
                Operand::Constant(Constant::Integer(0))
            }
        }
    }
    
    /// Add an instruction to the current block
    fn add_instruction(&mut self, instruction: Instruction) {
        if let Some(ref mut block) = self.current_block {
            block.instructions.push(instruction);
        }
    }
}

// Add this helper method to HirExpression
impl HirExpression {
    /// Get the type of this expression
    fn get_type(&self) -> front_end::types::Type {
        match self {
            HirExpression::Integer(_, _) => front_end::types::Type::Int,
            HirExpression::Boolean(_) => front_end::types::Type::Bool,
            HirExpression::String(_) => front_end::types::Type::String,
            HirExpression::Variable(_, typ, _) => typ.clone(),
            HirExpression::Binary { result_type, .. } => result_type.clone(),
            // Add other expression types as needed
            _ => front_end::types::Type::Int, // Default for now
        }
    }
}

