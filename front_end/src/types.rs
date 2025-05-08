#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    I64,
    // Add more types as needed
}

#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    Read,
    Write,
    Reads,
    Writes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PermissionedType {
    pub base_type: Type,
    pub permissions: Vec<Permission>,
}

impl PermissionedType {
    pub fn new(base_type: Type, permissions: Vec<Permission>) -> Self {
        Self {
            base_type,
            permissions,
        }
    }

    pub fn check_validity(&self) -> Result<(), String> {
        // Check invalid combinations
        if self.permissions.contains(&Permission::Read) && 
           self.permissions.contains(&Permission::Reads) {
            return Err("Cannot combine read and reads".to_string());
        }
        // Add more validation rules
        Ok(())
    }

    pub fn check_write_permission(&self) -> Result<(), String> {
        if !self.permissions.contains(&Permission::Write) {
            return Err("Write permission required".to_string());
        }
        Ok(())
    }

    pub fn check_compatibility(&self, other: &PermissionedType) -> bool {
        match (&self.permissions[..], &other.permissions[..]) {
            // reads write -> read is allowed
            ([Permission::Reads, Permission::Write], [Permission::Read]) => true,
            
            // reads write -> reads is allowed
            ([Permission::Reads, Permission::Write], [Permission::Reads]) => true,
            
            // read write is exclusive
            ([Permission::Read, Permission::Write], _) => false,
            
            // Other cases...
            _ => false
        }
    }
}