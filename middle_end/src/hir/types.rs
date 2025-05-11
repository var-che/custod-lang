//! HIR Type Definitions
//!
//! This module contains the core data structures that represent the HIR.

use front_end::token::{PermissionType, TokenType};
use front_end::types::Type;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum HirValue {
    Number(i64, Type),
    Variable(String, Type),
    Binary {
        left: Box<HirValue>,
        operator: TokenType,
        right: Box<HirValue>,
        result_type: Type,
    },
    Clone(Box<HirValue>),
    Consume(Box<HirValue>),
    Peak(Box<HirValue>),
    Call {
        function: String,
        arguments: Vec<HirValue>,
        result_type: Type,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum HirStatement {
    Declaration(HirVariable),
    Method(HirMethod),
    Assignment(HirAssignment),
    Actor(HirActor),
    Print(HirValue),
    AtomicBlock(Vec<HirStatement>),
    ActorCall {
        actor: String,
        behavior: String,
        arguments: Vec<HirValue>,
    },
    Return(HirValue),
    // Add this variant to handle expression statements
    Expression(HirValue),
}

#[derive(Debug, PartialEq, Clone)]
pub struct HirVariable {
    pub name: String,
    pub typ: Type,
    pub permissions: super::permissions::PermissionInfo,
    pub initializer: Option<HirValue>
}

#[derive(Debug, PartialEq, Clone)]
pub struct HirProgram {
    pub statements: Vec<HirStatement>,
    pub type_info: TypeEnvironment,
}

#[derive(Debug, PartialEq, Clone)]
pub struct HirActor {
    pub name: String,
    pub state: Vec<HirVariable>,
    pub methods: Vec<HirMethod>,
    pub behaviors: Vec<HirBehavior>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MethodKind {
    Regular,    // fn keyword
    Behavior,   // on keyword
}

#[derive(Debug, PartialEq, Clone)]
pub struct HirMethod {
    pub name: String,
    pub kind: MethodKind,
    pub params: Vec<HirVariable>,
    pub body: Vec<HirStatement>,
    pub return_type: Option<Type>,
    pub used_permissions: Vec<super::permissions::PermissionInfo>
}

#[derive(Debug, PartialEq, Clone)]
pub struct HirBehavior {
    pub name: String,
    pub params: Vec<HirVariable>,
    pub body: Vec<HirStatement>,
    pub atomic_blocks: Vec<Vec<HirStatement>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct HirAssignment {
    pub target: String,
    pub value: HirValue,
    pub permissions_used: Vec<PermissionType>
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeEnvironment {
    pub variables: HashMap<String, Type>
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}