pub struct TypeChecker {
    type_env: HashMap<String, PermissionedType>
}

impl TypeChecker {
    pub fn check_assignment(&self, target: &str, value: &Expression) -> Result<(), String> {
        let target_type = self.type_env.get(target)
            .ok_or_else(|| format!("Variable {} not found", target))?;

        // Check if target has write permission
        if !target_type.permissions.contains(&Permission::Write) {
            return Err(format!("Cannot write to {}: no write permission", target));
        }

        // Check value permissions
        match value {
            Expression::Variable(name) => {
                let source_type = self.type_env.get(name)
                    .ok_or_else(|| format!("Variable {} not found", name))?;
                
                // Check permission compatibility
                if !source_type.check_compatibility(target_type) {
                    return Err(format!("Incompatible permissions"));
                }
            }
            // Other cases...
        }
        Ok(())
    }
}