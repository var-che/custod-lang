#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Single-character tokens
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    Comma, Colon, Semicolon,
    
    // One or two character tokens
    Plus, PlusEqual,
    Minus, MinusEqual, Arrow,
    Star, StarEqual,
    Slash, SlashEqual,
    Equal, EqualEqual,
    Bang, BangEqual,
    Less, LessEqual,
    Greater, GreaterEqual,
    
    // Permission keywords
    Read, Write,
    Reads, Writes,
    
    // Permission operations
    Peak, Clone,  // Add these new token types
    
    // Literals
    Identifier(String),
    String(String),
    Number(i64),
    
    // Keywords
    If, Else, While, For,
    Fn, On, Actor, Return, Print,
    
    // Types
    TypeInt, TypeInt8, TypeInt16, TypeInt32, TypeInt64,
    TypeUInt, TypeUInt8, TypeUInt16, TypeUInt32, TypeUInt64,
    TypeFloat, TypeFloat32, TypeFloat64,
    TypeBool, TypeString, TypeUnit,
    
    // Special
    Error(String),
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

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,  
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: &str, line: usize, column: usize) -> Self {
        Self {
            token_type,
            lexeme: lexeme.to_string(),
            line,
            column,
            length: lexeme.chars().count(),
        }
    }
}