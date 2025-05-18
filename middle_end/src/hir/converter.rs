//! AST to HIR conversion
//!
//! This module handles the conversion from the AST representation to HIR.

use crate::hir::types::*;
use front_end::ast::{Statement, Expression};
use front_end::token::TokenType;
use front_end::types::Type;

/// Convert an AST statement to an HIR program
pub fn convert_ast_to_hir(stmt: Statement) -> HirProgram {
    let mut converter = HirConverter::new();
    let result = converter.convert_statement(stmt);
    converter.finalize(result)
}

/// Convert a list of AST statements to an HIR program
pub fn convert_statements_to_hir(statements: Vec<Statement>) -> HirProgram {
    let mut program = HirProgram::new();
    let mut converter = HirConverter::new();
    
    // Process each statement
    for stmt in statements {
        let hir_stmt = converter.convert_statement(stmt);
        program.add_statement(hir_stmt);
    }
    
    // Add type information to program
    for (name, typ) in converter.type_info.variables {
        program.type_info.variables.insert(name, typ);
    }
    
    for (name, ret_type) in converter.type_info.functions {
        program.type_info.functions.insert(name, ret_type);
    }
    
    program
}

/// Helper struct for the conversion process
struct HirConverter {
    type_info: TypeInfo,
}

impl HirConverter {
    /// Create a new HIR converter
    pub fn new() -> Self {
        Self {
            type_info: TypeInfo::default(),
        }
    }
    
    /// Finalize conversion and return the HIR program
    pub fn finalize(self, stmt: HirStatement) -> HirProgram {
        let mut program = HirProgram::new();
        program.add_statement(stmt);
        program.type_info = self.type_info;
        program
    }
    
    // Overloaded version without arguments (for multiple statements case)
    pub fn finalize_multi(self) -> HirProgram {
        let mut program = HirProgram::new();
        program.type_info = self.type_info;
        program
    }
    
    /// Convert an AST statement to an HIR statement
    pub fn convert_statement(&mut self, stmt: Statement) -> HirStatement {
        match stmt {
            Statement::Declaration { name, typ, initializer } => {
                // Convert permissions from front-end to HIR format
                let permissions: Vec<Permission> = typ.permissions
                    .iter()
                    .map(|p| Permission::from(p.clone()))
                    .collect();
                
                // Convert initializer if present
                let init_expr = initializer.map(|expr| self.convert_expression(expr));
                
                // Record type information
                let base_type = typ.base_type.clone();
                self.type_info.variables.insert(name.clone(), base_type.clone());
                
                HirStatement::Declaration(HirVariable {
                    name,
                    typ: base_type,
                    permissions,
                    initializer: init_expr,
                })
            },
            
            Statement::Assignment { target, value, target_type: _ } => {
                let hir_value = self.convert_expression(value);
                
                HirStatement::Assignment(HirAssignment {
                    target,
                    value: hir_value,
                })
            },
            
            Statement::Function { name, params, body, return_type, is_behavior: _ } => {
                // Convert parameters
                let parameters: Vec<HirParameter> = params
                    .into_iter()
                    .map(|(name, typ)| {
                        let permissions: Vec<Permission> = typ.permissions
                            .iter()
                            .map(|p| Permission::from(p.clone()))
                            .collect();
                            
                        // Record parameter type
                        self.type_info.variables.insert(name.clone(), typ.base_type.clone());
                        
                        HirParameter {
                            name,
                            typ: typ.base_type,
                            permissions,
                        }
                    })
                    .collect();
                
                // Convert function body
                let hir_body: Vec<HirStatement> = body
                    .into_iter()
                    .map(|stmt| self.convert_statement(stmt))
                    .collect();
                
                // Record function return type
                let return_typ = return_type.map(|t| t.base_type.clone());
                self.type_info.functions.insert(name.clone(), return_typ.clone());
                
                HirStatement::Function(HirFunction {
                    name,
                    parameters,
                    body: hir_body,
                    return_type: return_typ,
                })
            },
            
            Statement::Return(expr) => {
                HirStatement::Return(self.convert_expression(expr))
            },
            
            Statement::Print(expr) => {
                HirStatement::Print(self.convert_expression(expr))
            },
            
            Statement::Expression(expr) => {
                HirStatement::Expression(self.convert_expression(expr))
            },
            
            Statement::Block(statements) => {
                let hir_statements: Vec<HirStatement> = statements
                    .into_iter()
                    .map(|stmt| self.convert_statement(stmt))
                    .collect();
                
                HirStatement::Block(hir_statements)
            },
            
            // Any other types of statements we need to handle
            _ => {
                // For now, convert unhandled statement types to an empty block
                HirStatement::Block(vec![])
            }
        }
    }
    
    /// Convert an AST expression to an HIR expression
    pub fn convert_expression(&mut self, expr: Expression) -> HirExpression {
        match expr {
            Expression::Number(value) => {
                HirExpression::Integer(value)
            },
            
            Expression::Variable(name) => {
                // Look up the type if known, otherwise default to Int
                let typ = self.type_info.variables
                    .get(&name)
                    .cloned()
                    .unwrap_or(Type::Int);
                    
                HirExpression::Variable(name, typ)
            },
            
            Expression::Binary { left, operator, right } => {
                let left_expr = self.convert_expression(*left);
                let right_expr = self.convert_expression(*right);
                
                // Simplistic type determination - in a real compiler we'd do proper type checking
                let result_type = match operator {
                    TokenType::Plus | TokenType::Minus | TokenType::Star | TokenType::Slash => Type::Int,
                    TokenType::Greater | TokenType::GreaterEqual | 
                    TokenType::Less | TokenType::LessEqual | 
                    TokenType::EqualEqual | TokenType::BangEqual => Type::Bool,
                    _ => Type::Int,
                };
                
                HirExpression::Binary {
                    left: Box::new(left_expr),
                    operator,
                    right: Box::new(right_expr),
                    result_type,
                }
            },
            
            Expression::Call { function, arguments } => {
                let hir_arguments: Vec<HirExpression> = arguments
                    .into_iter()
                    .map(|arg| self.convert_expression(arg))
                    .collect();
                
                // Try to look up the return type, default to Int if unknown
                let result_type = self.type_info.functions
                    .get(&function)
                    .and_then(|t| t.clone())
                    .unwrap_or(Type::Int);
                
                HirExpression::Call {
                    function,
                    arguments: hir_arguments,
                    result_type,
                }
            },
            
            Expression::Peak(expr) => {
                HirExpression::Peak(Box::new(self.convert_expression(*expr)))
            },
            
            Expression::Clone(expr) => {
                HirExpression::Clone(Box::new(self.convert_expression(*expr)))
            },
        }
    }
}
