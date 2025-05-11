use std::collections::HashMap;
use front_end::types::{Type, Permission, PermissionedType};
use front_end::token::{PermissionType, TokenType};  // Make sure TokenType includes PlusEquals
use crate::hir::{HirProgram, HirStatement, HirValue, HirVariable, HirAssignment, HirMethod};

#[derive(Clone)]
pub struct TypeEnvironment {
    variables: HashMap<String, PermissionedType>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

pub struct TypePermissionChecker {
    type_env: TypeEnvironment,
    functions: HashMap<String, HirMethod>,  // Add function tracking
}

impl TypePermissionChecker {
    pub fn new() -> Self {
        Self {
            type_env: TypeEnvironment::new(),
            functions: HashMap::new(),
        }
    }

    pub fn check_program(&mut self, program: &HirProgram) -> Result<(), String> {
        // First pass: register all functions
        for statement in &program.statements {
            if let HirStatement::Method(method) = statement {
                self.functions.insert(method.name.clone(), method.clone());
            }
        }

        // Second pass: check all statements
        for statement in &program.statements {
            match statement {
                HirStatement::Method(method) => {
                    // Create new scope for function
                    let mut function_env = TypeEnvironment::new();
                    
                    // Add parameters to function scope
                    for param in &method.params {
                        function_env.variables.insert(
                            param.name.clone(),
                            PermissionedType::new(
                                param.typ.clone(),
                                param.permissions.permissions.iter()
                                    .map(|p| Permission::from(LocalPermission::from(p.clone())))
                                    .collect()
                            )
                        );
                    }

                    // Save current environment
                    let old_env = std::mem::replace(&mut self.type_env, function_env);
                    
                    // Check function body with parameter scope
                    for stmt in &method.body {
                        self.check_statement(stmt)?;
                    }

                    // Restore outer scope
                    self.type_env = old_env;
                },
                _ => self.check_statement(statement)?,
            }
        }
        Ok(())
    }

    fn check_statement(&mut self, stmt: &HirStatement) -> Result<(), String> {
        match stmt {
            HirStatement::Declaration(var) => self.check_variable_declaration(var),
            HirStatement::Assignment(assign) => self.check_assignment(assign),
            HirStatement::Print(value) => self.check_value_permissions(value),
            HirStatement::AtomicBlock(stmts) => {
                for stmt in stmts {
                    self.check_statement(stmt)?;
                }
                Ok(())
            },
            _ => Ok(()),
        }
    }

    fn check_variable_declaration(&mut self, var: &HirVariable) -> Result<(), String> {
        if let Some(init) = &var.initializer {
            match init {
                HirValue::Variable(source_name, _) => {
                    if let Some(source_type) = self.type_env.variables.get(source_name) {
                        // For writes permission, allow write access to be granted
                        if var.permissions.permissions.contains(&PermissionType::Write) {
                            if !source_type.permissions.contains(&Permission::Writes) {
                                return Err(format!(
                                    "Cannot write to {} - missing writes permission",
                                    source_name
                                ));
                            }
                            // Implicitly grant read permission for write access
                            // This is because writing requires reading the value first
                            let mut permissions = var.permissions.permissions.clone();
                            if !permissions.contains(&PermissionType::Read) {
                                permissions.push(PermissionType::Read);
                            }
                            let converted_permissions: Vec<Permission> = permissions.iter()
                                .map(|p| match p {
                                    PermissionType::Read => Permission::Read,
                                    PermissionType::Write => Permission::Write,
                                    PermissionType::Reads => Permission::Reads,
                                    PermissionType::Writes => Permission::Writes,
                                })
                                .collect();
                            self.type_env.variables.insert(
                                var.name.clone(),
                                PermissionedType::new(var.typ.clone(), converted_permissions)
                            );
                        }
                    }

                    // Must use peak for read permission
                    if var.permissions.permissions.contains(&PermissionType::Read) {
                        return Err(format!(
                            "Must use 'peak' keyword for temporary read access: {} = peak {}",
                            var.name, source_name
                        ));
                    }
                    self.check_source_permissions(source_name)?;
                }
                HirValue::Peak(expr) => {
                    if let HirValue::Variable(ref source_name, _) = **expr {
                        if let Some(source_type) = self.type_env.variables.get(source_name.as_str()) {
                            // For peak operations, we need either read or reads permission
                            if !source_type.permissions.contains(&Permission::Read) &&
                               !source_type.permissions.contains(&Permission::Reads) {
                                return Err(format!(
                                    "Cannot peak from {} - missing read/reads permission",
                                    source_name
                                ));
                            }
                        } else {
                            return Err(format!("Variable {} not found", source_name));
                        }
                    }
                }
                _ => self.check_value_permissions(init)?,
            }
        }

        // Store variable permissions
        self.store_permissions(var)?;
        Ok(())
    }

    fn check_source_permissions(&self, source_name: &str) -> Result<(), String> {
        if let Some(source_type) = self.type_env.variables.get(source_name) {
            if source_type.has_exclusive_access() {
                return Err(format!(
                    "Cannot read from {} - variable has exclusive read write access",
                    source_name
                ));
            }
        } else {
            return Err(format!("Variable {} not found", source_name));
        }
        Ok(())
    }

    fn store_permissions(&mut self, var: &HirVariable) -> Result<(), String> {
        let permissions: Vec<Permission> = var.permissions.permissions.iter()
            .map(|p| match p {
                PermissionType::Read => Permission::Read,
                PermissionType::Write => Permission::Write,
                PermissionType::Reads => Permission::Reads,
                PermissionType::Writes => Permission::Writes,
            })
            .collect();

        self.type_env.variables.insert(
            var.name.clone(),
            PermissionedType::new(var.typ.clone(), permissions)
        );
        Ok(())
    }

    fn check_assignment(&self, assign: &HirAssignment) -> Result<(), String> {
        // Check if target has write permission
        if let Some(target_type) = self.type_env.variables.get(&assign.target) {
            if !target_type.permissions.contains(&Permission::Write) {
                return Err(format!(
                    "Cannot write to {} - missing write permission",
                    assign.target
                ));
            }

            // Also check read permissions for variables used in the value
            self.check_value_permissions(&assign.value)?;
        } else {
            return Err(format!("Variable {} not found", assign.target));
        }
        Ok(())
    }

    fn check_value_permissions(&self, value: &HirValue) -> Result<(), String> {
        match value {
            HirValue::Variable(name, _) => {
                if let Some(var_type) = self.type_env.variables.get(name) {
                    // Allow reading if variable has read or reads permission
                    Ok(if !var_type.permissions.contains(&Permission::Read) &&
                       !var_type.permissions.contains(&Permission::Reads) {
                        return Err(format!(
                            "Cannot read from {} - missing read/reads permission",
                            name
                        ));
                    })
                } else {
                    return Err(format!("Variable {} not found", name));
                }
            }
            HirValue::Binary { left, right, .. } => {
                self.check_value_permissions(left)?;
                self.check_value_permissions(right)?;
                Ok(())
            }
            HirValue::Call { function, arguments, .. } => {
                // Lookup function
                let method = self.functions.get(function)
                    .ok_or_else(|| format!("Function {} not found", function))?;
                
                // Check argument count matches
                if arguments.len() != method.params.len() {
                    return Err(format!(
                        "Function {} expects {} arguments but got {}", 
                        function, method.params.len(), arguments.len()
                    ));
                }

                // Check argument permissions
                for (arg, param) in arguments.iter().zip(&method.params) {
                    self.check_value_permissions(arg)?;
                    // Additional permission checks could be added here
                }
                Ok(())
            },
            _ => Ok(()),
        }
    }

    fn check_method(&self, method: &HirMethod) -> Result<(), String> {
        // Add method parameter types to a new scope
        let mut scope_env = self.type_env.clone();
        for param in &method.params {
            // Convert param type to PermissionedType
            let param_type = PermissionedType::new(
                param.typ.clone(),
                param.permissions.permissions.iter()
                    .map(|p| match p {
                        PermissionType::Read => Permission::Read,
                        PermissionType::Write => Permission::Write,
                        PermissionType::Reads => Permission::Reads,
                        PermissionType::Writes => Permission::Writes,
                    })
                    .collect()
            );
            scope_env.variables.insert(param.name.clone(), param_type);
        }

        // Check method body with parameter scope
        for statement in &method.body {
            match statement {
                HirStatement::Return(value) => {
                    self.check_value_permissions(value)?;
                },
                // ... handle other statement types ...
                _ => {}
            }
        }
        Ok(())
    }

    fn check_binary_operation(&self, stmt: &HirStatement) -> Result<(), String> {
        if let HirStatement::Assignment(assign) = stmt {
            if let HirValue::Binary { 
                left: ref left, 
                operator, 
                .. 
            } = &assign.value {
                if let HirValue::Variable(name, _) = &**left {
                    if operator == &TokenType::Plus {
                        if let Some(perms) = self.type_env.variables.get(name) {
                            if !perms.permissions.contains(&Permission::Read) && 
                               !perms.permissions.contains(&Permission::Reads) {
                                return Err(format!(
                                    "Cannot use += on '{}' - write-only reference cannot read its own value.\n\
                                     Suggestions:\n\
                                     1. Use direct assignment instead: {} = <new_value>\n\
                                     2. Request read permission: read write {} = counter",
                                    name, name, name
                                ));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

// Local wrapper type for Permission
#[derive(Clone)]
struct LocalPermission(Permission);

impl From<PermissionType> for LocalPermission {
    fn from(p: PermissionType) -> Self {
        LocalPermission(match p {
            PermissionType::Read => Permission::Read,
            PermissionType::Write => Permission::Write,
            PermissionType::Reads => Permission::Reads,
            PermissionType::Writes => Permission::Writes,
        })
    }
}

impl From<LocalPermission> for Permission {
    fn from(p: LocalPermission) -> Self {
        p.0
    }
}

trait ExclusiveAccess {
    fn has_exclusive_access(&self) -> bool;
}

impl ExclusiveAccess for PermissionedType {
    fn has_exclusive_access(&self) -> bool {
        self.permissions.contains(&Permission::Read) && 
        self.permissions.contains(&Permission::Write) &&
        !self.permissions.contains(&Permission::Reads) &&
        !self.permissions.contains(&Permission::Writes)  // Added Writes check
    }
}