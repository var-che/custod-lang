//! HIR desugaring
//!
//! This module implements transformations that simplify complex HIR constructs
//! into simpler, more primitive operations.

use crate::hir::types::*;

/// Desugar a HIR program
pub fn desugar_program(program: &mut HirProgram) {
    let mut desugarer = Desugarer::new();
    
    // Process each statement in the program
    for i in 0..program.statements.len() {
        if let Some(statement) = program.statements.get(i).cloned() {
            let desugared = desugarer.desugar_statement(&statement);
            
            // Replace with desugared version if it changed
            if !desugarer.is_same_statement(&statement, &desugared) {
                program.statements[i] = desugared;
            }
        }
    }
}

/// Helper struct for desugaring operations
struct Desugarer;

impl Desugarer {
    /// Create a new desugarer
    pub fn new() -> Self {
        Self
    }
    
    /// Check if two statements are the same (to avoid unnecessary replacements)
    fn is_same_statement(&self, original: &HirStatement, desugared: &HirStatement) -> bool {
        // This is a simplistic check - in a real compiler we'd have a more
        // comprehensive equality check or add a PartialEq implementation
        match (original, desugared) {
            (HirStatement::Declaration(_), HirStatement::Declaration(_)) => true,
            (HirStatement::Assignment(_), HirStatement::Assignment(_)) => true,
            (HirStatement::Expression(_), HirStatement::Expression(_)) => true,
            (HirStatement::Return(_), HirStatement::Return(_)) => true,
            (HirStatement::Print(_), HirStatement::Print(_)) => true,
            (HirStatement::Function(_), HirStatement::Function(_)) => true,
            (HirStatement::Block(a), HirStatement::Block(b)) => a.len() == b.len(),
            _ => false,
        }
    }
    
    /// Desugar a statement
    pub fn desugar_statement(&mut self, stmt: &HirStatement) -> HirStatement {
        match stmt {
            HirStatement::Declaration(var) => {
                // For declarations with initializers, we could split them into
                // declaration and assignment for simplicity
                let initializer = var.initializer.as_ref().map(|expr| self.desugar_expression(expr));
                
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
                    value: self.desugar_expression(&assign.value),
                })
            },
            
            HirStatement::Expression(expr) => {
                HirStatement::Expression(self.desugar_expression(expr))
            },

            HirStatement::Print(expr) => {
                HirStatement::Print(self.desugar_expression(expr))
            },
            
            HirStatement::Block(statements) => {
                let desugared_stmts: Vec<HirStatement> = statements
                    .iter()
                    .map(|s| self.desugar_statement(s))
                    .collect();
                
                HirStatement::Block(desugared_stmts)
            },
            
            HirStatement::Function(func) => {
                // Desugar function body statements
                let desugared_body: Vec<HirStatement> = func.body
                    .iter()
                    .map(|s| self.desugar_statement(s))
                    .collect();
                
                HirStatement::Function(HirFunction {
                    name: func.name.clone(),
                    parameters: func.parameters.clone(),
                    body: desugared_body,
                    return_type: func.return_type.clone(),
                })
            },
            
            HirStatement::Return(expr_opt) => {
                // Process Option<HirExpression> correctly
                let desugared = expr_opt.as_ref().map(|expr| self.desugar_expression(expr));
                HirStatement::Return(desugared)
            },
            
            // Handle any other statement types
            _ => stmt.clone(),
        }
    }
    
    /// Desugar an expression
    pub fn desugar_expression(&mut self, expr: &HirExpression) -> HirExpression {
        match expr {
            HirExpression::Integer(val, loc) => {
                HirExpression::Integer(*val, loc.clone())
            },
            
            HirExpression::Boolean(val) => {
                HirExpression::Boolean(*val)
            },
            
            HirExpression::String(val) => {
                HirExpression::String(val.clone())
            },
            
            HirExpression::Variable(name, typ, loc) => {
                HirExpression::Variable(name.clone(), typ.clone(), loc.clone())
            },
            
            HirExpression::Binary { left, operator, right, result_type } => {
                // Desugar nested binary expressions
                let desugared_left = Box::new(self.desugar_expression(left));
                let desugared_right = Box::new(self.desugar_expression(right));
                
                HirExpression::Binary {
                    left: desugared_left,
                    operator: operator.clone(),
                    right: desugared_right,
                    result_type: result_type.clone(),
                }
            },
            
            HirExpression::Call { function, arguments, result_type } => {
                // Desugar function call arguments
                let desugared_args: Vec<HirExpression> = arguments
                    .iter()
                    .map(|a| self.desugar_expression(a))
                    .collect();
                
                HirExpression::Call {
                    function: function.clone(),
                    arguments: desugared_args,
                    result_type: result_type.clone(),
                }
            },
            
            HirExpression::Conditional { condition, then_expr, else_expr, result_type } => {
                HirExpression::Conditional {
                    condition: Box::new(self.desugar_expression(condition)),
                    then_expr: Box::new(self.desugar_expression(then_expr)),
                    else_expr: Box::new(self.desugar_expression(else_expr)),
                    result_type: result_type.clone(),
                }
            },
            
            HirExpression::Cast { expr, target_type } => {
                HirExpression::Cast {
                    expr: Box::new(self.desugar_expression(expr)),
                    target_type: target_type.clone(),
                }
            },
            
            HirExpression::Peak(expr) => {
                HirExpression::Peak(Box::new(self.desugar_expression(expr)))
            },
            
            HirExpression::Clone(expr) => {
                HirExpression::Clone(Box::new(self.desugar_expression(expr)))
            },
        }
    }
}
