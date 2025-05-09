use crate::token::TokenType;
use crate::types::PermissionedType;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Number(i64),
    Variable(String),
    Binary {
        left: Box<Expression>,
        operator: TokenType,
        right: Box<Expression>,
    },
    Clone(Box<Expression>),  // Add this variant
}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub name: String,
    pub typ: PermissionedType,
    pub initializer: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub typ: PermissionedType,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Declaration {
        name: String,
        typ: PermissionedType,
        initializer: Option<Expression>,
    },
    Assignment {
        target: String,
        value: Expression,
        target_type: PermissionedType,
    },
    Print(Expression),
    Block(Vec<Statement>),  // Add this variant
    Actor {
        name: String,
        state: Vec<Statement>,
        methods: Vec<Statement>,
        behaviors: Vec<Statement>,
    },
    Function {
        name: String,
        params: Vec<(String, PermissionedType)>,
        body: Vec<Statement>,
        return_type: Option<PermissionedType>,
        is_behavior: bool,
    },
    AtomicBlock(Vec<Statement>),
}

// Helper functions for permission checking at parse time
impl Statement {
    pub fn check_permissions(&self) -> Result<(), String> {
        match self {
            Statement::Declaration { typ, .. } => {
                typ.check_validity()
            },
            Statement::Assignment { target_type, .. } => {
                target_type.check_write_permission()
            },
            Statement::Function { params, return_type, .. } => {
                for (_, typ) in params {
                    typ.check_validity()?;
                }
                if let Some(ret) = return_type {
                    ret.check_validity()?;
                }
                Ok(())
            },
            // Other cases...
            _ => Ok(())
        }
    }
}

