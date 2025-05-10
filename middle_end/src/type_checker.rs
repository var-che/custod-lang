use std::collections::HashMap;
use front_end::types::{Type, Permission, PermissionedType};
use front_end::token::PermissionType;
use crate::hir::{HirProgram, HirStatement, HirValue, HirVariable, HirAssignment};

pub struct TypePermissionChecker {
    type_env: HashMap<String, PermissionedType>
}

impl TypePermissionChecker {
    pub fn new() -> Self {
        Self {
            type_env: HashMap::new()
        }
    }

    pub fn check_program(&mut self, program: &HirProgram) -> Result<(), String> {
        for stmt in &program.statements {
            self.check_statement(stmt)?;
        }
        Ok(())
    }

    fn check_statement(&mut self, stmt: &HirStatement) -> Result<(), String> {
        match stmt {
            HirStatement::Declaration(var) => self.check_declaration(var),
            HirStatement::Assignment(assign) => self.check_assignment(assign),
            HirStatement::Print(value) => self.check_value_permissions(value),
            HirStatement::AtomicBlock(stmts) => {
                for stmt in stmts {
                    self.check_statement(stmt)?;
                }
                Ok(())
            },
            _ => Ok(())
        }
    }

    fn check_declaration(&mut self, var: &HirVariable) -> Result<(), String> {
        if let Some(init) = &var.initializer {
            match init {
                HirValue::Variable(source_name, _) => {
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
                        if let Some(source_type) = self.type_env.get(source_name.as_str()) {
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
        if let Some(source_type) = self.type_env.get(source_name) {
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

        self.type_env.insert(
            var.name.clone(),
            PermissionedType::new(var.typ.clone(), permissions)
        );
        Ok(())
    }

    fn check_assignment(&self, assign: &HirAssignment) -> Result<(), String> {
        // Check if target has write permission
        if let Some(target_type) = self.type_env.get(&assign.target) {
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
                if let Some(var_type) = self.type_env.get(name) {
                    // Allow reading if variable has read or reads permission
                    if !var_type.permissions.contains(&Permission::Read) &&
                       !var_type.permissions.contains(&Permission::Reads) {
                        return Err(format!(
                            "Cannot read from {} - missing read/reads permission",
                            name
                        ));
                    }
                } else {
                    return Err(format!("Variable {} not found", name));
                }
            }
            HirValue::Binary { left, right, .. } => {
                self.check_value_permissions(left)?;
                self.check_value_permissions(right)?;
            }
            _ => {}
        }
        Ok(())
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
        !self.permissions.contains(&Permission::Writes)
    }
}