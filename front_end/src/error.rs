use crate::symbol_table::{ResolutionError, Span};
use std::fmt;

/// Error type for parsing errors
#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
    pub error_code: Option<String>,
}

impl ParseError {
    pub fn new(span: Span, message: String) -> Self {
        Self {
            span,
            message,
            error_code: None,
        }
    }
    
    pub fn with_code(mut self, code: &str) -> Self {
        self.error_code = Some(code.to_string());
        self
    }
    
    pub fn unexpected_token(span: Span, message: String) -> Self {
        Self::new(span, message).with_code("E0001")
    }
    
    pub fn invalid_expression(span: Span, message: String) -> Self {
        Self::new(span, message).with_code("E0002")
    }
    
    pub fn syntax_error(span: Span, message: String) -> Self {
        Self::new(span, message).with_code("E0003")
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(code) = &self.error_code {
            write!(f, "error[{}]: {}", code, self.message)?;
        } else {
            write!(f, "error: {}", self.message)?;
        }
        
        if let Some(file) = &self.span.source_file {
            write!(f, " at {}:{}:{}", file, self.span.start_line, self.span.start_column)
        } else {
            write!(f, " at line {}:{}", self.span.start_line, self.span.start_column)
        }
    }
}

/// Common error type for compiler errors
#[derive(Debug, Clone)]
pub enum CompileError {
    Parse(ParseError),
    Resolution(ResolutionError), // This is fine, as ResolutionError now derives Clone
    TypeError(String, Span),
    IoError(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompileError::Parse(err) => write!(f, "{}", err),
            CompileError::Resolution(err) => write!(f, "{}", err),
            CompileError::TypeError(msg, span) => {
                write!(f, "type error: {}", msg)?;
                if let Some(file) = &span.source_file {
                    write!(f, " at {}:{}:{}", file, span.start_line, span.start_column)
                } else {
                    write!(f, " at line {}:{}", span.start_line, span.start_column)
                }
            },
            CompileError::IoError(msg) => write!(f, "io error: {}", msg),
        }
    }
}
