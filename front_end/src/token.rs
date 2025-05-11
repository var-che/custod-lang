#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Single-character tokens
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star, Colon,
    Bang, // '!'
    Actor, On, Atomic,

    If, Else,

    Fn, Return, Print, 

    TypeI64, 
    Number(i64),  // Make sure this takes an i64 value
    Identifier(String),  // Add this variant to hold identifier names

    Error(String),  // Make error take a String for the message

    Read, Write, Reads, Writes, Peak, Clone,

    // Two-character tokens
    Equal, EqualEqual, // '=', '=='
    BangEqual, // '!='
    Less, LessEqual, // '<', '<='
    Greater, GreaterEqual, // '>', '>='
    PlusEqual, MinusEqual, StarEqual, SlashEqual, // '+=', '-=', '*=', '/='
    Arrow,
    Eof, // '->'
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
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: &str) -> Self {
        Self {
            token_type,
            lexeme: lexeme.to_string(),
        }
    }
}