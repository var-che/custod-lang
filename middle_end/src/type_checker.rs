use std::collections::HashMap;
use front_end::types::{Type, Permission, PermissionedType};
use crate::hir::{HirProgram, HirStatement, HirValue, HirVariable};

#[derive(Debug)]
pub enum TypeError {
    InvalidPermissionCombination(String),
    PermissionViolation(String),
    TypeMismatch(String),
    UndefinedVariable(String),
}

pub struct TypePermissionChecker {
    type_env: HashMap<String, PermissionedType>
}

impl TypePermissionChecker {
    pub fn new() -> Self {
        Self {
            type_env: HashMap::new()
        }
    }

    // This runs during compilation
    pub fn check_program(&mut self, program: &HirProgram) -> Result<(), TypeError> {
        for stmt in &program.statements {
            self.check_statement(stmt)?;
        }
        Ok(())
    }

    fn check_statement(&mut self, stmt: &HirStatement) -> Result<(), TypeError> {
        match stmt {
            HirStatement::Declaration(var) => self.check_declaration(var),
            HirStatement::Assignment(assign) => self.check_assignment(assign),
            HirStatement::Actor(actor) => self.check_actor(actor),
            _ => Ok(())
        }
    }

    fn check_declaration(&mut self, var: &HirVariable) -> Result<(), TypeError> {
        // Check permission combination validity
        let ptype = PermissionedType::new(
            var.typ.clone(),
            var.permissions.permissions.iter().map(|p| match p {
                PermissionType::Read => Permission::Read,
                PermissionType::Write => Permission::Write,
                PermissionType::Reads => Permission::Reads,
                PermissionType::Writes => Permission::Writes,
            }).collect()
        );

        // Validate permission combinations at compile time
        if ptype.has_invalid_combination() {
            return Err(TypeError::InvalidPermissionCombination(
                format!("Invalid permission combination for variable {}", var.name)
            ));
        }

        // Check initializer permissions
        if let Some(init) = &var.initializer {
            self.check_initialization(&var.name, init, &ptype)?;
        }

        // Add to type environment
        self.type_env.insert(var.name.clone(), ptype);
        Ok(())
    }

    fn check_assignment(&self, assign: &HirAssignment) -> Result<(), TypeError> {
        let target_type = self.type_env.get(&assign.target)
            .ok_or_else(|| TypeError::UndefinedVariable(
                format!("Variable {} not defined", assign.target)
            ))?;

        // Check write permission at compile time
        if !target_type.has_write_permission() {
            return Err(TypeError::PermissionViolation(
                format!("Cannot write to {} - no write permission", assign.target)
            ));
        }

        // Check value permissions
        self.check_value_permissions(&assign.value, target_type)
    }

    fn check_value_permissions(
        &self,
        value: &HirValue,
        expected_type: &PermissionedType
    ) -> Result<(), TypeError> {
        match value {
            HirValue::Variable(name, _) => {
                let source_type = self.type_env.get(name)
                    .ok_or_else(|| TypeError::UndefinedVariable(
                        format!("Variable {} not defined", name)
                    ))?;

                // Check permission compatibility at compile time
                if !source_type.is_compatible_with(expected_type) {
                    return Err(TypeError::PermissionViolation(
                        format!("Incompatible permissions between {} and target", name)
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }
}