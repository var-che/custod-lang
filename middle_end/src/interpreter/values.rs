//! Value representation and operations for the interpreter

/// Values that can be manipulated by the interpreter
#[derive(Debug, Clone, PartialEq)]
pub enum InterpreterValue {
    /// Integer value
    Integer(i64),
    
    /// Boolean value
    Boolean(bool),
    
    /// Unit/void value
    Unit,
}

impl InterpreterValue {
    /// Try to convert to integer
    pub fn as_integer(&self) -> Result<i64, String> {
        match self {
            InterpreterValue::Integer(i) => Ok(*i),
            _ => Err("Expected integer value".into()),
        }
    }
    
    /// Try to convert to boolean
    pub fn as_boolean(&self) -> Result<bool, String> {
        match self {
            InterpreterValue::Boolean(b) => Ok(*b),
            InterpreterValue::Integer(i) => Ok(*i != 0),
            _ => Err("Cannot convert to boolean".into()),
        }
    }
    
    /// Add two values
    pub fn add(&self, other: &Self) -> Result<Self, String> {
        match (self, other) {
            (InterpreterValue::Integer(a), InterpreterValue::Integer(b)) => {
                a.checked_add(*b)
                 .map(InterpreterValue::Integer)
                 .ok_or_else(|| "Integer overflow during addition".to_string())
            },
            _ => Err("Cannot add these value types".into()),
        }
    }
    
    /// Convert from an i64
    pub fn from_i64(val: i64) -> Self {
        InterpreterValue::Integer(val)
    }
    
    /// Convert to i64
    pub fn to_i64(&self) -> Result<i64, String> {
        self.as_integer()
    }
}

impl std::fmt::Display for InterpreterValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InterpreterValue::Integer(i) => write!(f, "{}", i),
            InterpreterValue::Boolean(b) => write!(f, "{}", b),
            InterpreterValue::Unit => write!(f, "()"),
        }
    }
}