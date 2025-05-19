//! Constant folding for HIR
//!
//! This module implements compile-time evaluation of constant expressions.

use crate::hir::types::*;
use front_end::token::TokenType;

/// Fold constants in a HIR program
pub fn fold_constants(program: &mut HirProgram) {
    let mut folder = ConstantFolder::new();
    
    // Process each statement in the program
    for i in 0..program.statements.len() {
        if let Some(statement) = program.statements.get(i).cloned() {
            let folded = folder.fold_statement(&statement);
            program.statements[i] = folded;
        }
    }
}

/// Visitor for constant folding
struct ConstantFolder;

impl ConstantFolder {
    /// Create a new constant folder
    fn new() -> Self {
        Self
    }
    
    /// Fold constants in a statement
    fn fold_statement(&mut self, stmt: &HirStatement) -> HirStatement {
        match stmt {
            HirStatement::Declaration(var) => {
                let initializer = var.initializer.as_ref().map(|expr| self.fold_expression(expr));
                
                HirStatement::Declaration(HirVariable {
                    name: var.name.clone(),
                    typ: var.typ.clone(),
                    permissions: var.permissions.clone(),
                    initializer,
                    location: var.location.clone(),
                })
            },
            
            HirStatement::Assignment(assign) => {
                HirStatement::Assignment(HirAssignment {
                    target: assign.target.clone(),
                    value: self.fold_expression(&assign.value),
                })
            },
            
            HirStatement::Expression(expr) => {
                HirStatement::Expression(self.fold_expression(expr))
            },
            
            HirStatement::Return(expr_opt) => {
                HirStatement::Return(expr_opt.as_ref().map(|expr| self.fold_expression(expr)))
            },
            
            HirStatement::Print(expr) => {
                HirStatement::Print(self.fold_expression(expr))
            },
            
            HirStatement::Function(func) => {
                // Fold expressions in the function body
                let body = func.body.iter()
                    .map(|stmt| self.fold_statement(stmt))
                    .collect();
                
                HirStatement::Function(HirFunction {
                    name: func.name.clone(),
                    parameters: func.parameters.clone(),
                    body,
                    return_type: func.return_type.clone(),
                })
            },
            
            HirStatement::Block(statements) => {
                let folded = statements.iter()
                    .map(|stmt| self.fold_statement(stmt))
                    .collect();
                
                HirStatement::Block(folded)
            },
            
            // Fold expressions in control flow statements
            HirStatement::If { condition, then_branch, else_branch } => {
                HirStatement::If {
                    condition: self.fold_expression(condition),
                    then_branch: Box::new(self.fold_statement(then_branch)),
                    else_branch: else_branch.as_ref().map(|stmt| Box::new(self.fold_statement(stmt))),
                }
            },
            
            HirStatement::While { condition, body } => {
                HirStatement::While {
                    condition: self.fold_expression(condition),
                    body: Box::new(self.fold_statement(body)),
                }
            },
        }
    }
    
    /// Fold constants in an expression
    fn fold_expression(&mut self, expr: &HirExpression) -> HirExpression {
        match expr {
            HirExpression::Binary { left, operator, right, result_type } => {
                let folded_left = self.fold_expression(left);
                let folded_right = self.fold_expression(right);
                
                // Try to evaluate constant binary expressions
                match (&folded_left, operator, &folded_right) {
                    (HirExpression::Integer(lhs, _), TokenType::Plus, HirExpression::Integer(rhs, _)) => {
                        HirExpression::Integer(lhs + rhs, None)
                    },
                    (HirExpression::Integer(lhs, _), TokenType::Minus, HirExpression::Integer(rhs, _)) => {
                        HirExpression::Integer(lhs - rhs, None)
                    },
                    (HirExpression::Integer(lhs, _), TokenType::Star, HirExpression::Integer(rhs, _)) => {
                        HirExpression::Integer(lhs * rhs, None)
                    },
                    (HirExpression::Integer(lhs, _), TokenType::Slash, HirExpression::Integer(rhs, _)) if *rhs != 0 => {
                        HirExpression::Integer(lhs / rhs, None)
                    },
                    // Can't fold, return a new binary expression with folded operands
                    _ => HirExpression::Binary {
                        left: Box::new(folded_left),
                        operator: operator.clone(),
                        right: Box::new(folded_right),
                        result_type: result_type.clone(),
                    }
                }
            },
            
            HirExpression::Conditional { condition, then_expr, else_expr, result_type } => {
                let folded_condition = self.fold_expression(condition);
                
                // If condition is a constant boolean, select the appropriate branch
                match folded_condition {
                    HirExpression::Boolean(true) => self.fold_expression(then_expr),
                    HirExpression::Boolean(false) => self.fold_expression(else_expr),
                    _ => HirExpression::Conditional {
                        condition: Box::new(folded_condition),
                        then_expr: Box::new(self.fold_expression(then_expr)),
                        else_expr: Box::new(self.fold_expression(else_expr)),
                        result_type: result_type.clone(),
                    }
                }
            },
            
            // Other expression types just need their subexpressions folded
            HirExpression::Call { function, arguments, result_type } => {
                let folded_args = arguments.iter()
                    .map(|arg| self.fold_expression(arg))
                    .collect();
                
                HirExpression::Call {
                    function: function.clone(),
                    arguments: folded_args,
                    result_type: result_type.clone(),
                }
            },
            
            HirExpression::Cast { expr, target_type } => {
                HirExpression::Cast {
                    expr: Box::new(self.fold_expression(expr)),
                    target_type: target_type.clone(),
                }
            },
            
            HirExpression::Peak(expr) => {
                HirExpression::Peak(Box::new(self.fold_expression(expr)))
            },
            
            HirExpression::Clone(expr) => {
                HirExpression::Clone(Box::new(self.fold_expression(expr)))
            },
            
            // Leaf nodes (literals and variables) remain the same
            _ => expr.clone(),
        }
    }
}
