#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,    // Platform default integer (replaces I64)
    Int8,   // 8-bit signed integer
    Int16,  // 16-bit signed integer
    Int32,  // 32-bit signed integer
    Int64,  // 64-bit signed integer (was I64)
    UInt,   // Platform default unsigned integer
    UInt8,  // 8-bit unsigned integer
    UInt16, // 16-bit unsigned integer
    UInt32, // 32-bit unsigned integer
    UInt64, // 64-bit unsigned integer
    Float,  // Platform default floating point (32 or 64 bit)
    Float32,// 32-bit floating point
    Float64,// 64-bit floating point
    Bool,   // Boolean type
    String, // String type
    Unit,   // Unit type (for functions that return nothing)
}

impl Type {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "int" => Ok(Type::Int),
            "int8" => Ok(Type::Int8),
            "int16" => Ok(Type::Int16),
            "int32" => Ok(Type::Int32),
            "int64" => Ok(Type::Int64),
            "uint" => Ok(Type::UInt),
            "uint8" => Ok(Type::UInt8),
            "uint16" => Ok(Type::UInt16),
            "uint32" => Ok(Type::UInt32),
            "uint64" => Ok(Type::UInt64),
            "float" => Ok(Type::Float),
            "float32" => Ok(Type::Float32),
            "float64" => Ok(Type::Float64),
            "bool" => Ok(Type::Bool),
            "string" => Ok(Type::String),
            "unit" => Ok(Type::Unit),
            _ => Err(format!("Unknown type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]  // Added Clone
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