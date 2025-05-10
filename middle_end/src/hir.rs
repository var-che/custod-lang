use front_end::ast::{Statement, Expression};
use front_end::types::Type;
use front_end::token::{PermissionType, TokenType};
use front_end::types::Permission;
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
    Peak(Box<HirValue>),  // Add Peak variant
}

#[derive(Debug, PartialEq, Clone)]
pub enum HirStatement {
    Declaration(HirVariable),
    Method(HirMethod),         // Changed from Function/Handler
    Assignment(HirAssignment),
    Actor(HirActor),
    Print(HirValue),
    AtomicBlock(Vec<HirStatement>),
    ActorCall {
        actor: String,
        behavior: String,
        arguments: Vec<HirValue>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct HirVariable {
    pub name: String,
    pub typ: Type,
    pub permissions: PermissionInfo,
    pub initializer: Option<HirValue>
}

#[derive(Debug, PartialEq, Clone)]
pub struct PermissionInfo {
    pub permissions: Vec<PermissionType>,  
    pub is_isolated: bool,
    pub is_sendable: bool,
}

impl PermissionInfo {
    pub fn new(permissions: Vec<PermissionType>) -> Self {
        // Check for invalid permission combinations
        let mut info = Self {
            permissions,
            is_isolated: true,
            is_sendable: true,
        };

        // Update isolation based on permissions
        if info.permissions.contains(&PermissionType::Reads) ||
           info.permissions.contains(&PermissionType::Writes) {
            info.is_isolated = false;
        }

        info
    }

    #[allow(dead_code)]
    fn has_permission(&self, permission: &PermissionType) -> bool {
        self.permissions.contains(permission)
    }

    fn has_read_access(&self) -> bool {
        self.permissions.iter().any(|p| matches!(p, PermissionType::Read | PermissionType::Reads))
    }

    #[allow(dead_code)]
    fn has_write_access(&self) -> bool {
        self.permissions.iter().any(|p| matches!(p, PermissionType::Write | PermissionType::Writes))
    }

    fn has_exclusive_permissions(&self) -> bool {
        // read,write means exclusive access - no one else can access
        self.permissions.contains(&PermissionType::Read) && 
        self.permissions.contains(&PermissionType::Write)
    }

    fn is_reads_write(&self) -> bool {
        // reads,write means others can read but only owner can write
        self.permissions.contains(&PermissionType::Reads) && 
        self.permissions.contains(&PermissionType::Write)
    }

    fn can_be_consumed(&self) -> bool {
        self.is_reads_write()
    }

    fn check_permission_combination(&self) -> Result<(), String> {
        if self.is_reads_write() {
            if self.permissions.contains(&PermissionType::Read) {
                return Err("Cannot combine 'reads write' with 'read'".to_string());
            }
            Ok(())
        } else if self.has_exclusive_permissions() {
            if self.permissions.contains(&PermissionType::Reads) || 
               self.permissions.contains(&PermissionType::Writes) {
                return Err("Cannot combine 'read write' with other permissions".to_string());
            }
            Ok(())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct HirProgram {
    pub statements: Vec<HirStatement>,
    pub type_info: TypeEnvironment,
}

pub fn convert_to_hir(ast: Statement) -> HirProgram {
    let mut statements = Vec::new();
    let mut type_info = TypeEnvironment {
        variables: HashMap::new(),
    };

    match ast {
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
    }

    HirProgram {
        statements,
        type_info,
    }
}

fn convert_expression(expr: Expression) -> HirValue {
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

// Fix convert_variable to handle both tuple and Statement forms
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

// Add helper enum for variable declaration conversion
enum VariableDecl {
    Tuple((String, Vec<TokenType>)),
    Statement(Statement),
}

impl From<(String, Vec<TokenType>)> for VariableDecl {
    fn from(tuple: (String, Vec<TokenType>)) -> Self {  // Fixed parenthesis placement
        VariableDecl::Tuple(tuple)
    }
}

impl From<Statement> for VariableDecl {
    fn from(stmt: Statement) -> Self {
        VariableDecl::Statement(stmt)
    }
}

// Fix convert_statement to use the correct convert_variable form
fn convert_statement(stmt: Statement) -> HirStatement {
    match stmt {
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

#[derive(Debug, PartialEq, Clone)]
pub struct TypeEnvironment {
    variables: HashMap<String, Type>
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

pub struct PermissionChecker {
    // Track variable permissions
    pub(crate) permissions: HashMap<String, PermissionInfo>,
}

// Fix PermissionChecker's actor check
impl PermissionChecker {
    pub fn check_program(program: &HirProgram) -> Result<(), String> {
        let mut checker = PermissionChecker {
            permissions: HashMap::new(),
        };
        
        for statement in &program.statements {
            match statement {
                HirStatement::Actor(actor) => {
                    // Check actor state permissions
                    for var in &actor.state {
                        var.permissions.check_permission_combination()?;
                    }
                    
                    // Check method permissions
                    for method in &actor.methods {
                        for perm in &method.used_permissions {
                            perm.check_permission_combination()?;
                        }
                    }

                    // Check behavior permissions
                    for behavior in &actor.behaviors {
                        for stmt in &behavior.body {
                            checker.check_statement(stmt)?;
                        }
                    }
                }
                HirStatement::Declaration(var) => {
                    // Check variable permissions
                    var.permissions.check_permission_combination()?;
                    
                    // Check initializer
                    if let Some(init) = &var.initializer {
                        match init {
                            HirValue::Variable(name, _) => {
                                if let Some(source_perms) = checker.permissions.get(name) {
                                    // If trying to get write permission from a reads,write variable
                                    if source_perms.is_reads_write() && 
                                       var.permissions.permissions.contains(&PermissionType::Write) {
                                        return Err(format!(
                                            "Cannot get write permission to '{}' - source has reads,write permission",
                                            name
                                        ));
                                    }
                                }
                            }
                            _ => {}
                        }
                        checker.check_permissions(init)?;
                    }
                    
                    // Record permissions
                    checker.permissions.insert(var.name.clone(), var.permissions.clone());
                }
                HirStatement::Assignment(assignment) => {
                    // Check if we have write permission to the target
                    if let Some(target_perms) = checker.permissions.get(&assignment.target) {
                        if !target_perms.has_write_access() {
                            return Err(format!(
                                "Cannot write to '{}' - need write permission",
                                assignment.target
                            ));
                        }
                        // For reads,write permission, only the owner can write
                        if target_perms.is_reads_write() {
                            // Assignment is allowed because we're the owner
                        } else {
                            // Check the value being assigned
                            checker.check_permissions(&assignment.value)?;
                        }
                    } else {
                        return Err(format!("Variable '{}' not found", assignment.target));
                    }
                }
                HirStatement::Print(value) => {
                    // Check if we have read permission for the value being printed
                    checker.check_permissions(value)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn check_statement(&self, stmt: &HirStatement) -> Result<(), String> {
        match stmt {
            HirStatement::AtomicBlock(stmts) => {
                for stmt in stmts {
                    self.check_statement(stmt)?;
                }
                Ok(())
            }
            HirStatement::Assignment(assign) => self.check_assignment(assign),
            HirStatement::Declaration(var) => self.check_declaration(var),
            HirStatement::Method(_) => Ok(()),
            HirStatement::Actor(_) => Ok(()),
            HirStatement::Print(value) => {
                // Check read permissions for printed values
                self.check_value_permissions(value)
            }
            HirStatement::ActorCall { .. } => Ok(()),
            _ => Ok(()),
        }
    }

    fn check_permissions(&self, value: &HirValue) -> Result<(), String> {
        match value {
            HirValue::Variable(name, _) => {
                if let Some(perms) = self.permissions.get(name) {
                    if perms.has_exclusive_permissions() {
                        // If variable has read,write permissions, it's exclusive
                        Err(format!("'{}' has exclusive read and write permissions", name))
                    } else if perms.is_reads_write() {
                        // reads,write allows others to read
                        Ok(())
                    } else if !perms.has_read_access() {
                        Err(format!("Cannot read from '{}' - need read permission", name))
                    } else {
                        Ok(())
                    }
                } else {
                    Err(format!("Variable '{}' not found", name))
                }
            }
            HirValue::Peak(expr) => {
                // For peak operations, we need either read or reads permission
                self.check_permissions(expr)
            },
            HirValue::Consume(inner) => {
                // Check if we can consume the value
                match **inner {
                    HirValue::Variable(ref name, _) => {
                        if let Some(perms) = self.permissions.get(name) {
                            if perms.can_be_consumed() {
                                Ok(())
                            } else {
                                Err(format!("Cannot consume '{}' - not consumable", name))
                            }
                        } else {
                            Err(format!("Variable '{}' not found", name))
                        }
                    }
                    _ => Err("Can only consume variables".to_string()),
                }
            }
            HirValue::Binary { left, right, .. } => {
                self.check_permissions(left)?;
                self.check_permissions(right)
            }
            _ => Ok(())
        }
    }

    fn check_value_permissions(&self, value: &HirValue) -> Result<(), String> {
        self.check_permissions(value)
    }

    fn check_declaration(&self, var: &HirVariable) -> Result<(), String> {
        var.permissions.check_permission_combination()?;
        if let Some(init) = &var.initializer {
            self.check_permissions(init)?;
        }
        Ok(())
    }

    fn check_assignment(&self, assign: &HirAssignment) -> Result<(), String> {
        // Check if we have write permission to the target
        if let Some(target_perms) = self.permissions.get(&assign.target) {
            if !target_perms.has_write_access() {
                return Err(format!(
                    "Cannot write to '{}' - need write permission",
                    assign.target
                ));
            }
            self.check_permissions(&assign.value)?;
        } else {
            return Err(format!("Variable '{}' not found", assign.target));
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct HirActor {
    pub name: String,
    pub state: Vec<HirVariable>,
    pub methods: Vec<HirMethod>,     // Regular functions (fn)
    pub behaviors: Vec<HirBehavior>, // Async behaviors (on)
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
    pub used_permissions: Vec<PermissionInfo>
}

#[derive(Debug, PartialEq, Clone)]
pub struct HirBehavior {
    pub name: String,
    pub params: Vec<HirVariable>,
    pub body: Vec<HirStatement>,
    pub atomic_blocks: Vec<Vec<HirStatement>>,  // Track atomic blocks
}

#[derive(Debug, PartialEq, Clone)]
pub struct HirAssignment {
    pub target: String,
    pub value: HirValue,
    pub permissions_used: Vec<PermissionType>
}

#[derive(Default)]
pub struct HirConverter {
    type_environment: TypeEnvironment,
}

impl HirConverter {
    fn get_type(&self, name: &str) -> Type {
        self.type_environment.variables.get(name)
            .cloned()
            .unwrap_or(Type::I64)
    }

    fn convert_expression(&mut self, expr: Expression) -> HirValue {
        match expr {
            Expression::Number(n) => HirValue::Number(n, Type::I64),
            Expression::Variable(name) => HirValue::Variable(name.clone(), self.get_type(&name)),
            Expression::Clone(expr) => {
                        let inner = self.convert_expression(*expr);
                        HirValue::Clone(Box::new(inner))
                    },
            Expression::Binary { left, operator, right } => todo!(),
Expression::Peak(expression) => todo!(),
        }
    }
}


