use crate::token::TokenType;
use crate::types::PermissionedType;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Number(i64),
    Variable(String),
    Binary {
        left: Box<Expression>,
        operator: TokenType,
        right: Box<Expression>,
    },
    Clone(Box<Expression>),
    Peak(Box<Expression>),
    Call {
        function: String,
        arguments: Vec<Expression>,
    },
}

impl Expression {
    pub fn new_binary(left: Expression, operator: TokenType, right: Expression) -> Self {
        // Debug print to check the binary expression creation
        println!("Creating binary expression with operator: {:?}", operator);
        
        Expression::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
    
    pub fn new_variable(name: String) -> Self {
        Expression::Variable(name)
    }
    
    pub fn new_number(value: i64) -> Self {
        Expression::Number(value)
    }
    
    pub fn new_call(function: String, arguments: Vec<Expression>) -> Self {
        Expression::Call {
            function,
            arguments,
        }
    }
    
    pub fn new_peak(expr: Expression) -> Self {
        Expression::Peak(Box::new(expr))
    }
    
    pub fn new_clone(expr: Expression) -> Self {
        Expression::Clone(Box::new(expr))
    }
    
    pub fn accept<T>(&self, visitor: &mut impl Visitor<T>) -> T {
        visitor.visit_expression(self)
    }
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
    Expression(Expression),
    Print(Expression),
    Block(Vec<Statement>),
    Return(Expression),
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
    
    pub fn new_declaration(name: String, typ: PermissionedType, initializer: Option<Expression>) -> Self {
        Statement::Declaration { name, typ, initializer }
    }
    
    pub fn new_assignment(target: String, value: Expression, target_type: PermissionedType) -> Self {
        Statement::Assignment { target, value, target_type }
    }

    pub fn new_expression(expr: Expression) -> Self {
        Statement::Expression(expr)
    }
    
    pub fn new_print(expr: Expression) -> Self {
        Statement::Print(expr)
    }
    
    pub fn new_block(statements: Vec<Statement>) -> Self {
        Statement::Block(statements)
    }
    
    pub fn new_return(expr: Expression) -> Self {
        Statement::Return(expr)
    }
    
    pub fn new_atomic_block(statements: Vec<Statement>) -> Self {
        Statement::AtomicBlock(statements)
    }
    
    pub fn accept<T>(&self, visitor: &mut impl Visitor<T>) -> T {
        visitor.visit_statement(self)
    }
}

pub struct FunctionBuilder {
    name: String,
    parameters: Vec<(String, PermissionedType)>,
    body: Vec<Statement>,
    return_type: Option<PermissionedType>,
    is_behavior: bool,
}

impl FunctionBuilder {
    pub fn new(name: String) -> Self {
        FunctionBuilder {
            name,
            parameters: Vec::new(),
            body: Vec::new(),
            return_type: None,
            is_behavior: false,
        }
    }
    
    pub fn with_parameter(mut self, name: String, typ: PermissionedType) -> Self {
        self.parameters.push((name, typ));
        self
    }
    
    pub fn with_return_type(mut self, typ: Option<PermissionedType>) -> Self {
        self.return_type = typ;
        self
    }
    
    pub fn as_behavior(mut self, is_behavior: bool) -> Self {
        self.is_behavior = is_behavior;
        self
    }
    
    pub fn with_body(mut self, statements: Vec<Statement>) -> Self {
        self.body = statements;
        self
    }
    
    pub fn build(self) -> Statement {
        Statement::Function {
            name: self.name,
            params: self.parameters,
            body: self.body,
            return_type: self.return_type,
            is_behavior: self.is_behavior,
        }
    }
}

pub trait Visitor<T> {
    fn visit_expression(&mut self, expr: &Expression) -> T;
    fn visit_statement(&mut self, stmt: &Statement) -> T;
    // Other visitor methods...
}

