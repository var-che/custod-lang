#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Keywords
    Actor,     // New: 'actor' keyword
    On,        // New: 'on' keyword for behaviors
    Fn,        // New: 'fn' keyword for methods
    Atomic,    // New: 'atomic' block
    Reads,
    Write,
    Read,
    Writes,
    
    // Symbols
    LeftBrace,    // {
    RightBrace,   // }
    LeftParen,    // (
    RightParen,   // )
    Arrow,        // ->
    Comma,        // ,
    
    // Operators
    Plus,
    Equal,
    PlusEquals,   // Change this to PlusEquals to match usage
    
    // Literals
    Identifier(String),
    Number(i64),
    
    // Other
    Eof,
}

use crate::types::Permission;

#[derive(Debug, Clone, PartialEq)]
pub enum PermissionType {
    Read,
    Write,
    Reads,
    Writes,  // Added this variant
}

impl From<Permission> for PermissionType {
    fn from(p: Permission) -> Self {
        match p {
            Permission::Read => PermissionType::Read,
            Permission::Write => PermissionType::Write,
            Permission::Reads => PermissionType::Reads,
            Permission::Writes => PermissionType::Writes,  // Handle the new variant
        }
    }
}