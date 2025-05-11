//! AST to HIR Conversion
//!
//! Handles the transformation from AST nodes to HIR representation,
//! including type and permission handling.

use front_end::ast::{Statement, Expression};
use front_end::token::{TokenType, PermissionType};
use front_end::types::Type;
use std::collections::HashMap;
use crate::hir::{
    types::*, 
    permissions::PermissionInfo
};

pub fn convert_to_hir(ast: Statement) -> HirProgram {
    let mut statements = Vec::new();
    let mut type_info = TypeEnvironment {
        variables: HashMap::new(),
    };

    match ast {
        Statement::Expression(expr) => {
            statements.push(HirStatement::Expression(convert_expression(expr)));
        },
        Statement::Declaration { name, typ, initializer } => {
            let hir_var = HirVariable {
                name,
                typ: typ.base_type,
                permissions: PermissionInfo::new(
                    typ.permissions.into_iter()
                        .map(Into::into)
                        .collect()
                ),
                initializer: initializer.map(|expr| convert_expression(expr)),
            };

            type_info.variables.insert(hir_var.name.clone(), hir_var.typ.clone());
            statements.push(HirStatement::Declaration(hir_var));
        },
        Statement::Assignment { target, value, target_type } => {
                let hir_assignment = HirAssignment {
                    target,
                    value: convert_expression(value),
                    permissions_used: vec![PermissionType::Write],
                };
                statements.push(HirStatement::Assignment(hir_assignment));
            },
        Statement::Print(expr) => {
                statements.push(HirStatement::Print(convert_expression(expr)));
            },
        Statement::Actor { .. } => {
                statements.push(HirStatement::Actor(convert_actor(ast)));
            },
        Statement::Function { .. } => {
                statements.push(HirStatement::Method(convert_method(ast)));
            },
        Statement::AtomicBlock(block_stmts) => {
                statements.push(HirStatement::AtomicBlock(
                    block_stmts.into_iter()
                        .map(|s| convert_statement(s))
                        .collect()
                ));
            },
        Statement::Block(block_statements) => {
            // Convert each statement in the block
            for stmt in block_statements {
                let hir = convert_to_hir(stmt);
                statements.extend(hir.statements);
                type_info.variables.extend(hir.type_info.variables);
            }
        },
        Statement::Return(expr) => {
            // Convert return statement with expression
            statements.push(HirStatement::Return(convert_expression(expr)));
        },
    }

    HirProgram {
        statements,
        type_info,
    }
}

pub fn convert_expression(expr: Expression) -> HirValue {
    match expr {
        Expression::Number(n) => HirValue::Number(n, Type::I64),
        Expression::Binary { left, operator, right } => HirValue::Binary {
            left: Box::new(convert_expression(*left)),
            operator,
            right: Box::new(convert_expression(*right)),
            result_type: Type::I64,
        },
        Expression::Variable(name) => HirValue::Variable(name, Type::I64),
        Expression::Clone(expr) => {
            let inner = convert_expression(*expr);
            HirValue::Clone(Box::new(inner))
        },
        Expression::Peak(expr) => {
            let inner = convert_expression(*expr);
            HirValue::Peak(Box::new(inner))
        },
        Expression::Call { function, arguments } => {
            HirValue::Call {
                function: function.clone(),
                arguments: arguments.into_iter()
                    .map(|arg| convert_expression(arg))
                    .collect(),
                result_type: Type::I64,
            }
        }
    }
}

fn convert_statements(statements: Vec<Statement>) -> HirProgram {
    let mut program = HirProgram {
        statements: Vec::new(),
        type_info: TypeEnvironment::default(),
    };

    for stmt in statements {
        let hir = convert_to_hir(stmt);
        program.statements.extend(hir.statements);
        program.type_info.variables.extend(hir.type_info.variables);
    }

    program
}

fn convert_actor(ast: Statement) -> HirActor {
    match ast {
        Statement::Actor { name, state, methods, behaviors } => {
            HirActor {
                name,
                state: state.into_iter()
                    .map(|v| convert_variable(v))
                    .collect(),
                methods: methods.into_iter()
                    .map(|m| convert_method(m))
                    .collect(),
                behaviors: behaviors.into_iter()
                    .map(|b| convert_behavior(b))
                    .collect(),
            }
        },
        _ => panic!("Expected actor declaration"),
    }
}

fn convert_method(method: Statement) -> HirMethod {
    match method {
        Statement::Function { name, params, body, return_type, is_behavior } => {
            let converted_params = params.into_iter()
                .map(|(name, typ)| HirVariable {
                    name,
                    typ: typ.base_type,
                    permissions: PermissionInfo::new(
                        typ.permissions.into_iter()
                            .map(PermissionType::from)
                            .collect()
                    ),
                    initializer: None,
                })
                .collect();

            HirMethod {
                name,
                kind: if is_behavior { MethodKind::Behavior } else { MethodKind::Regular },
                params: converted_params,
                body: body.into_iter()
                    .map(|s| convert_statement(s))
                    .collect(),
                return_type: return_type.map(|t| t.base_type),
                used_permissions: Vec::new(),
            }
        },
        _ => panic!("Expected method declaration"),
    }
}

fn convert_behavior(behavior: Statement) -> HirBehavior {
    match behavior {
        Statement::Function { name, params, body, is_behavior, .. } => {
            if !is_behavior {
                panic!("Expected behavior but got regular method");
            }

            let converted_params = params.into_iter()
                .map(|(name, typ)| HirVariable {
                    name,
                    typ: typ.base_type,
                    permissions: PermissionInfo::new(
                        typ.permissions.into_iter()
                            .map(PermissionType::from)
                            .collect()
                    ),
                    initializer: None,
                })
                .collect();

            let converted_body: Vec<HirStatement> = body.into_iter()
                .map(|s| convert_statement(s))
                .collect();

            let atomic_blocks = extract_atomic_blocks_from_statements(&converted_body);

            HirBehavior {
                name,
                params: converted_params,
                body: converted_body,
                atomic_blocks,
            }
        },
        _ => panic!("Expected behavior declaration"),
    }
}

// Helper function to extract atomic blocks from HirStatements
fn extract_atomic_blocks_from_statements(statements: &[HirStatement]) -> Vec<Vec<HirStatement>> {
    statements.iter()
        .filter_map(|stmt| {
            if let HirStatement::AtomicBlock(block) = stmt {
                Some(block.clone())
            } else {
                None
            }
        })
        .collect()
}

// Helper function for variable declaration conversion
fn convert_variable(var: impl Into<VariableDecl>) -> HirVariable {
    match var.into() {
        VariableDecl::Tuple((name, permissions)) => {
            HirVariable {
                name,
                typ: Type::I64,
                permissions: PermissionInfo::new(
                    permissions.into_iter()
                        .filter_map(|t| match t {
                            TokenType::Read => Some(PermissionType::Read),
                            TokenType::Write => Some(PermissionType::Write),
                            TokenType::Reads => Some(PermissionType::Reads),
                            TokenType::Writes => Some(PermissionType::Writes),
                            _ => None
                        })
                        .collect()
                ),
                initializer: None,
            }
        },
        VariableDecl::Statement(Statement::Declaration { name, typ, initializer }) => {
            HirVariable {
                name,
                typ: typ.base_type,
                permissions: PermissionInfo::new(
                    typ.permissions.into_iter()
                        .map(PermissionType::from)
                        .collect()
                ),
                initializer: initializer.map(|expr| convert_expression(expr)),
            }
        },
        _ => panic!("Expected variable declaration"),
    }
}

// Helper enum for variable declaration conversion
pub(crate) enum VariableDecl {
    Tuple((String, Vec<TokenType>)),
    Statement(Statement),
}

impl From<(String, Vec<TokenType>)> for VariableDecl {
    fn from(tuple: (String, Vec<TokenType>)) -> Self {
        VariableDecl::Tuple(tuple)
    }
}

impl From<Statement> for VariableDecl {
    fn from(stmt: Statement) -> Self {
        VariableDecl::Statement(stmt)
    }
}

fn convert_statement(stmt: Statement) -> HirStatement {
    match stmt {
        Statement::Expression(expr) => 
            HirStatement::Expression(convert_expression(expr)),
        Statement::Declaration { .. } => 
            HirStatement::Declaration(convert_variable(stmt)),
        Statement::Assignment { target, value, target_type } => 
            HirStatement::Assignment(HirAssignment {
                target,
                value: convert_expression(value),
                permissions_used: vec![PermissionType::Write],
            }),
        Statement::Print(expr) => 
            HirStatement::Print(convert_expression(expr)),
        Statement::AtomicBlock(stmts) =>
            HirStatement::AtomicBlock(
                stmts.into_iter()
                    .map(|s| convert_statement(s))
                    .collect()
            ),
        Statement::Function { .. } =>
            HirStatement::Method(convert_method(stmt)),
        Statement::Return(expr) =>
            HirStatement::Return(convert_expression(expr)),
        _ => panic!("Unexpected statement type"),
    }
}

// Add helper function for atomic blocks
fn extract_atomic_blocks(statements: &[Statement]) -> Vec<Vec<HirStatement>> {
    let mut atomic_blocks = Vec::new();
    
    for stmt in statements {
        if let Statement::AtomicBlock(block_stmts) = stmt {
            atomic_blocks.push(
                block_stmts.iter()
                    .cloned()
                    .map(|s| convert_statement(s))
                    .collect()
            );
        }
    }
    
    atomic_blocks
}

#[derive(Default)]
pub struct HirConverter {
    type_environment: TypeEnvironment,
}

impl HirConverter {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn get_type(&self, name: &str) -> Type {
        self.type_environment.variables.get(name)
            .cloned()
            .unwrap_or(Type::I64)
    }

    pub fn convert_expression(&mut self, expr: Expression) -> HirValue {
        match expr {
            Expression::Number(n) => HirValue::Number(n, Type::I64),
            Expression::Variable(name) => HirValue::Variable(name.clone(), self.get_type(&name)),
            Expression::Clone(expr) => {
                let inner = self.convert_expression(*expr);
                HirValue::Clone(Box::new(inner))
            },
            Expression::Binary { left, operator, right } => HirValue::Binary {
                left: Box::new(self.convert_expression(*left)),
                operator,
                right: Box::new(self.convert_expression(*right)),
                result_type: Type::I64,
            },
            Expression::Peak(expr) => {
                let inner = self.convert_expression(*expr);
                HirValue::Peak(Box::new(inner))
            },
            Expression::Call { function, arguments } => {
                HirValue::Call {
                    function: function.clone(),
                    arguments: arguments.into_iter()
                        .map(|arg| self.convert_expression(arg))
                        .collect(),
                    result_type: Type::I64,
                }
            }
        }
    }
}