use front_end::ast::{Statement, Expression};
use front_end::token::{PermissionType, TokenType};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    I64,
}

#[derive(Debug, PartialEq, Clone)]
pub enum HirValue {
    Number(i64, Type),
    Binary {
        left: Box<HirValue>,
        operator: TokenType,
        right: Box<HirValue>,
        result_type: Type,
    },
    Variable(String, Type),
    Consume(Box<HirValue>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum HirStatement {
    Declaration(HirVariable),
    Function(HirFunction),
    Handler(HirHandler),
    Assignment(HirAssignment),
    Actor(HirActor),
    Print(HirValue),  // Add Print variant that takes a value to print
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
        Self {
            permissions,
            is_isolated: true,
            is_sendable: true,
        }
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
        Statement::Declaration(var_decl) => {
            // we are converting AST variable declaration to HIR
            let hir_val = HirVariable {
                name: var_decl.name,
                typ: Type::I64,
                permissions: PermissionInfo::new(vec![var_decl.permission]),
                initializer: match var_decl.initializer {
                    Expression::Number(n) => Some(HirValue::Number(n, Type::I64)),
                    Expression::Binary { left, operator, right } => Some(HirValue::Binary {
                        left: Box::new(match *left {
                            Expression::Number(n) => HirValue::Number(n, Type::I64),
                            Expression::Identifier(name) => HirValue::Variable(name, Type::I64),
                            _ => panic!("Invalid left operand"),
                        }),
                        operator,  // Using TokenType directly
                        right: Box::new(match *right {
                            Expression::Number(n) => HirValue::Number(n, Type::I64),
                            Expression::Identifier(name) => HirValue::Variable(name, Type::I64),
                            _ => panic!("Invalid right operand"),
                        }),
                        result_type: Type::I64,
                    }),
                    Expression::Identifier(name) => Some(HirValue::Variable(name, Type::I64)),
                    _ => None,
                },
            };

            // we are trackign type information
            type_info.variables.insert(hir_val.name.clone(), hir_val.typ.clone());

            statements.push(HirStatement::Declaration(hir_val))
        }
        Statement::Expression(expr) => {
            // Handle expression statements if needed
            // Currently just ignoring expressions
        }
        
    }
    HirProgram {
        statements,
        type_info,

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
    permissions: HashMap<String, PermissionInfo>,
}

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
                    
                    // Check function permissions
                    for func in &actor.functions {
                        for perm in &func.used_permissions {
                            perm.check_permission_combination()?;
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
}

#[derive(Debug, PartialEq, Clone)]
pub struct HirActor {
    name: String,
    state: Vec<HirVariable>,      // Actor's state (like counter)
    functions: Vec<HirFunction>,  // Internal functions
    handlers: Vec<HirHandler>,    // Message handlers (on)
}

#[derive(Debug, PartialEq, Clone)]
pub struct HirFunction {
    pub name: String,
    pub body: Vec<HirStatement>,
    pub used_permissions: Vec<PermissionInfo>  // Tracks which actor state variables are used
}

#[derive(Debug, PartialEq, Clone)]
pub struct HirHandler {
    name: String,
    params: Vec<HirVariable>,
    body: Vec<HirStatement>
}

#[derive(Debug, PartialEq, Clone)]
pub struct HirAssignment {
    pub target: String,
    pub value: HirValue,
    pub permissions_used: Vec<PermissionType>
}