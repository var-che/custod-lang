use crate::ast::{Expression, Statement};
use crate::symbol_table::{Span, ResolutionError, SymbolTable};
use crate::types::{Type, PermissionedType};
use crate::token::TokenType;

// Change the TypeChecker to use an immutable reference to SymbolTable
pub struct TypeChecker<'a> {
    symbol_table: &'a SymbolTable, // Changed from &'a mut SymbolTable
}

impl<'a> TypeChecker<'a> {
    // Update constructor to take an immutable reference
    pub fn new(symbol_table: &'a SymbolTable) -> Self {
        Self { symbol_table }
    }

    // Since we're not modifying the symbol table, we can make this immutable too
    pub fn check_function(&self, function: &Statement, span: Span) -> Vec<ResolutionError> {
        let mut errors = Vec::new();
        
        if let Statement::Function { name, return_type, body, .. } = function {
            // If there's an explicit return type, check all returns match it
            if let Some(return_type) = return_type {
                let expected_type = &return_type.base_type;
                
                // Check all return statements in the body
                for stmt in body {
                    if let Statement::Return(expr) = stmt {
                        let expr_type = self.infer_expression_type(expr);
                        
                        if &expr_type != expected_type {
                            errors.push(ResolutionError::TypeMismatch {
                                expected: format!("{:?}", expected_type),
                                found: format!("{:?}", expr_type),
                                span: span.clone(),
                                context: format!("in return value of function '{}'", name)
                            });
                        }
                    }
                }
            }
        }
        
        errors
    }
    
    pub fn infer_expression_type(&self, expr: &Expression) -> Type {
        match expr {
            Expression::Number(_) => Type::Int,
            Expression::Variable(name) => {
                // Since we're using an immutable reference, we need to handle this differently
                // We can't use resolve since it modifies the symbol table
                // Instead, let's use a simple type inference based on the expression
                
                // For variables, default to Int for now
                // In a more advanced implementation, we would track variable types separately
                Type::Int
            },
            
            Expression::Binary { operator, .. } => {
                // Arithmetic operators yield Int
                match operator {
                    TokenType::Plus | TokenType::Minus | TokenType::Star | TokenType::Slash => Type::Int,
                    
                    // Comparison operators yield Bool
                    TokenType::Greater | TokenType::GreaterEqual | TokenType::Less | 
                    TokenType::LessEqual | TokenType::EqualEqual | TokenType::BangEqual => Type::Bool,
                    
                    // Default to Int for other operators
                    _ => Type::Int,
                }
            },
            
            Expression::Call { .. } => Type::Int, // For now, all function calls default to Int
            
            // Operators that maintain the type of their operand
            Expression::Clone(expr) => self.infer_expression_type(expr),
            Expression::Peak(expr) => self.infer_expression_type(expr),
        }
    }
}
