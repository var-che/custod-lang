#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Keywords
    Read,
    Write,
    Reads,
    Writes,

    // Operators
    Plus,
    Equal,
    PlusEqual,

    // Literals
    Number(i64),
    Identifier(String),

    // Other
    Eof,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PermissionType {
    Read,
    Write,
    Reads,
    Writes,
}