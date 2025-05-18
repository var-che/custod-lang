//! HIR type definitions
//!
//! This module defines the types that make up the HIR structure.

use front_end::token::TokenType;
use front_end::types::Type;
use std::collections::HashMap;

/// A complete HIR program
#[derive(Debug, Clone)]
pub struct HirProgram {
    /// Top-level statements in the program
    pub statements: Vec<HirStatement>,
    
    /// Type information collected during conversion
    pub type_info: TypeInfo,
}

impl HirProgram {
    /// Create a new empty HIR program
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
            type_info: TypeInfo::default(),
        }
    }
    
    /// Add a statement to the program
    pub fn add_statement(&mut self, stmt: HirStatement) {
        self.statements.push(stmt);
    }
}

/// Type information for the program
#[derive(Debug, Clone, Default)]
pub struct TypeInfo {
    /// Maps variable names to their types
    pub variables: HashMap<String, Type>,
    
    /// Maps function names to their return types
    pub functions: HashMap<String, Option<Type>>,
}

/// A statement in the HIR
#[derive(Debug, Clone)]
pub enum HirStatement {
    /// Variable declaration
    Declaration(HirVariable),
    
    /// Assignment statement
    Assignment(HirAssignment),
    
    /// Function declaration
    Function(HirFunction),
    
    /// Return statement
    Return(HirExpression),
    
    /// Print statement
    Print(HirExpression),
    
    /// Expression statement
    Expression(HirExpression),
    
    /// Block of statements
    Block(Vec<HirStatement>),
}

/// A variable declaration in HIR
#[derive(Debug, Clone)]
pub struct HirVariable {
    /// Variable name
    pub name: String,
    
    /// Variable type
    pub typ: Type,
    
    /// Variable permissions
    pub permissions: Vec<Permission>,
    
    /// Initial value (if any)
    pub initializer: Option<HirExpression>,
}

/// An assignment in HIR
#[derive(Debug, Clone)]
pub struct HirAssignment {
    /// Target variable name
    pub target: String,
    
    /// Value being assigned
    pub value: HirExpression,
}

/// A function declaration in HIR
#[derive(Debug, Clone)]
pub struct HirFunction {
    /// Function name
    pub name: String,
    
    /// Function parameters
    pub parameters: Vec<HirParameter>,
    
    /// Function body
    pub body: Vec<HirStatement>,
    
    /// Return type (if specified)
    pub return_type: Option<Type>,
}

/// A function parameter in HIR
#[derive(Debug, Clone)]
pub struct HirParameter {
    /// Parameter name
    pub name: String,
    
    /// Parameter type
    pub typ: Type,
    
    /// Parameter permissions
    pub permissions: Vec<Permission>,
}

/// An expression in HIR
#[derive(Debug, Clone)]
pub enum HirExpression {
    /// Literal value
    Integer(i64),
    
    /// Variable reference
    Variable(String, Type),
    
    /// Binary operation
    Binary {
        left: Box<HirExpression>,
        operator: TokenType,
        right: Box<HirExpression>,
        result_type: Type,
    },
    
    /// Function call
    Call {
        function: String,
        arguments: Vec<HirExpression>,
        result_type: Type,
    },
    
    /// Peak operation (safely borrow a value)
    Peak(Box<HirExpression>),
    
    /// Clone operation (make a copy of a value)
    Clone(Box<HirExpression>),
}

/// Permission for variables and parameters
#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    Read,
    Write,
    Reads,
    Writes,
}

/// Convert a front-end permission to HIR permission
impl From<front_end::types::Permission> for Permission {
    fn from(perm: front_end::types::Permission) -> Self {
        match perm {
            front_end::types::Permission::Read => Permission::Read,
            front_end::types::Permission::Write => Permission::Write,
            front_end::types::Permission::Reads => Permission::Reads,
            front_end::types::Permission::Writes => Permission::Writes,
        }
    }
}