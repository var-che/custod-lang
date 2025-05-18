use std::collections::{HashMap, HashSet};
use crate::types::{Type, Permission, PermissionedType};
use crate::ast::{Statement, Expression};

/// Represents a region of source code with start and end positions
#[derive(Debug, Clone)]
pub struct Span {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub source_file: Option<String>, // Optional source file path
}

impl Span {
    pub fn new(start_line: usize, start_column: usize, end_line: usize, end_column: usize) -> Self {
        Self {
            start_line,
            start_column,
            end_line,
            end_column,
            source_file: None,
        }
    }
    
    pub fn with_file(mut self, file: &str) -> Self {
        self.source_file = Some(file.to_string());
        self
    }
    
    /// Create a single-point span (for when we only have a position, not a range)
    pub fn point(line: usize, column: usize) -> Self {
        Self::new(line, column, line, column)
    }
    
    /// Combine two spans into one that encompasses both
    pub fn combine(&self, other: &Span) -> Self {
        let start_line = self.start_line.min(other.start_line);
        let start_column = if self.start_line < other.start_line {
            self.start_column
        } else if self.start_line > other.start_line {
            other.start_column
        } else {
            self.start_column.min(other.start_column)
        };
        
        let end_line = self.end_line.max(other.end_line);
        let end_column = if self.end_line > other.end_line {
            self.end_column
        } else if self.end_line < other.end_line {
            other.end_column
        } else {
            self.end_column.max(other.end_column)
        };
        
        Self {
            start_line,
            start_column,
            end_line,
            end_column,
            source_file: self.source_file.clone(),
        }
    }
}

// For backward compatibility, keep the original Location struct but have it use Span
#[derive(Debug, Clone)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub span: Option<Span>, // Add an optional span field
}

impl From<&Span> for Location {
    fn from(span: &Span) -> Self {
        Self {
            line: span.start_line,
            column: span.start_column,
            span: Some(span.clone()),
        }
    }
}

impl Location {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
            span: Some(Span::point(line, column)),
        }
    }
    
    pub fn with_span(span: Span) -> Self {
        Self {
            line: span.start_line,
            column: span.start_column,
            span: Some(span),
        }
    }
}

/// Symbol represents a variable or function in the code
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub typ: PermissionedType,
    pub kind: SymbolKind,
    pub span: Span,  // Use Span instead of Location
}

/// Different kinds of symbols
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable,
    Parameter,
    Function,
}

/// A scope represents a lexical block with its own variable declarations
#[derive(Debug)]
struct Scope {
    symbols: HashMap<String, Symbol>,
    parent: Option<usize>, // Index of parent scope in SymbolTable's scopes vec
}

/// Error type for symbol resolution
#[derive(Debug)]
pub enum ResolutionError {
    DuplicateSymbol{name: String, first: Span, second: Span},
    UndefinedSymbol{name: String, span: Span},
    ImmutableAssignment{name: String, span: Span, declaration_span: Option<Span>},
    PermissionViolation{name: String, required: String, provided: String, span: Span, declaration_span: Option<Span>},
}

impl std::fmt::Display for ResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ResolutionError::DuplicateSymbol{name, first, second} => {
                write!(f, "Error: Variable '{}' already defined", name)?;
                if let Some(file) = &first.source_file {
                    write!(f, " in {}:{}:{}", file, first.start_line, first.start_column)?;
                } else {
                    write!(f, " at line {}:{}", first.start_line, first.start_column)?;
                }
                write!(f, ", but redeclared")?;
                if let Some(file) = &second.source_file {
                    write!(f, " in {}:{}:{}", file, second.start_line, second.start_column)
                } else {
                    write!(f, " at line {}:{}", second.start_line, second.start_column)
                }
            },
            ResolutionError::UndefinedSymbol{name, span} => {
                write!(f, "Error: Variable '{}' not defined in this scope", name)?;
                if let Some(file) = &span.source_file {
                    write!(f, " ({}:{}:{})", file, span.start_line, span.start_column)
                } else {
                    write!(f, " (line {}:{})", span.start_line, span.start_column)
                }
            },
            ResolutionError::ImmutableAssignment{name, span, declaration_span} => {
                write!(f, "Error: Cannot assign to immutable variable '{}'", name)?;
                if let Some(file) = &span.source_file {
                    write!(f, " at {}:{}:{}", file, span.start_line, span.start_column)?;
                } else {
                    write!(f, " at line {}:{}", span.start_line, span.start_column)?;
                }
                
                if let Some(decl_span) = declaration_span {
                    write!(f, "\nNote: '{}' was declared as immutable", name)?;
                    if let Some(file) = &decl_span.source_file {
                        write!(f, " at {}:{}:{}", file, decl_span.start_line, decl_span.start_column)
                    } else {
                        write!(f, " at line {}:{}", decl_span.start_line, decl_span.start_column)
                    }
                } else {
                    Ok(())
                }
            },
            ResolutionError::PermissionViolation{name, required, provided, span, declaration_span} => {
                write!(f, "Error: Variable '{}' requires permission '{}' but has '{}'", 
                      name, required, provided)?;
                if let Some(file) = &span.source_file {
                    write!(f, " at {}:{}:{}", file, span.start_line, span.start_column)?;
                } else {
                    write!(f, " at line {}:{}", span.start_line, span.start_column)?;
                }
                
                if let Some(decl_span) = declaration_span {
                    write!(f, "\nNote: '{}' was declared with permission '{}'", name, provided)?;
                    if let Some(file) = &decl_span.source_file {
                        write!(f, " at {}:{}:{}", file, decl_span.start_line, decl_span.start_column)
                    } else {
                        write!(f, " at line {}:{}", decl_span.start_line, decl_span.start_column)
                    }
                } else {
                    Ok(())
                }
            },
        }
    }
}

/// The Symbol Table manages variable scopes and provides methods for resolving variables
pub struct SymbolTable {
    scopes: Vec<Scope>,
    current_scope: usize,
    errors: Vec<ResolutionError>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut scopes = Vec::new();
        
        // Start with global scope
        scopes.push(Scope {
            symbols: HashMap::new(),
            parent: None,
        });
        
        Self {
            scopes,
            current_scope: 0,
            errors: Vec::new(),
        }
    }
    
    pub fn begin_scope(&mut self) -> usize {
        let parent = self.current_scope;
        let new_scope_idx = self.scopes.len();
        
        self.scopes.push(Scope {
            symbols: HashMap::new(),
            parent: Some(parent),
        });
        
        self.current_scope = new_scope_idx;
        new_scope_idx
    }
    
    pub fn end_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current_scope].parent {
            self.current_scope = parent;
        }
    }
    
    pub fn define(&mut self, symbol: Symbol) {
        // Check for duplicate in current scope
        if let Some(existing) = self.scopes[self.current_scope].symbols.get(&symbol.name) {
            self.errors.push(ResolutionError::DuplicateSymbol{
                name: symbol.name.clone(),
                first: existing.span.clone(),
                second: symbol.span.clone(),
            });
            return;
        }
        
        // Add to current scope
        self.scopes[self.current_scope].symbols.insert(
            symbol.name.clone(), symbol
        );
    }
    
    pub fn resolve(&mut self, name: &str, span: Span) -> Option<&Symbol> {
        let mut scope_idx = self.current_scope;
        
        loop {
            if let Some(symbol) = self.scopes[scope_idx].symbols.get(name) {
                return Some(symbol);
            }
            
            // Move to parent scope if it exists
            match self.scopes[scope_idx].parent {
                Some(parent_idx) => scope_idx = parent_idx,
                None => break, // We've reached the global scope
            }
        }
        
        // Symbol not found - add an error
        self.errors.push(ResolutionError::UndefinedSymbol {
            name: name.to_string(),
            span,
        });
        None
    }
    
    pub fn check_assignment(&mut self, name: &str, span: Span) -> Result<(), ResolutionError> {
        match self.resolve(name, span.clone()) {
            Some(symbol) => {
                // Check if variable has write permission
                if symbol.typ.permissions.contains(&Permission::Write) ||
                   symbol.typ.permissions.contains(&Permission::Writes) {
                    Ok(())
                } else {
                    Err(ResolutionError::ImmutableAssignment{
                        name: name.to_string(),
                        span,
                        declaration_span: Some(symbol.span.clone()),
                    })
                }
            },
            None => Err(ResolutionError::UndefinedSymbol{
                name: name.to_string(),
                span,
            }),
        }
    }
    
    pub fn process_statement(&mut self, stmt: &Statement, token_locations: &HashMap<usize, Location>) {
        match stmt {
            Statement::Declaration{name, typ, initializer} => {
                let location = token_locations.get(&self.current_scope)
                    .cloned().unwrap_or(Location{line: 0, column: 0, span: None});
                
                self.define(Symbol {
                    name: name.clone(),
                    typ: typ.clone(),
                    kind: SymbolKind::Variable,
                    span: location.span.unwrap_or_else(|| Span::point(0, 0)),
                });
                
                // Process initializer if present
                if let Some(expr) = initializer {
                    self.process_expression(expr, token_locations);
                }
            },
            Statement::Assignment{target, value, ..} => {
                // Check if variable exists and is writable
                let location = token_locations.get(&self.current_scope)
                    .cloned().unwrap_or(Location{line: 0, column: 0, span: None});
                
                let _ = self.check_assignment(target, location.span.unwrap_or_else(|| Span::point(0, 0)));
                self.process_expression(value, token_locations);
            },
            // Handle other statement types...
            _ => {}
        }
    }
    
    pub fn process_expression(&mut self, expr: &Expression, token_locations: &HashMap<usize, Location>) {
        match expr {
            Expression::Variable(name) => {
                let location = token_locations.get(&self.current_scope)
                    .cloned().unwrap_or(Location{line: 0, column: 0, span: None});
                
                let _ = self.resolve(name, location.span.unwrap_or_else(|| Span::point(0, 0)));
            },
            Expression::Binary{left, right, ..} => {
                self.process_expression(left, token_locations);
                self.process_expression(right, token_locations);
            },
            // Handle other expression types...
            _ => {}
        }
    }
    
    pub fn get_errors(&self) -> &[ResolutionError] {
        &self.errors
    }
    
    pub fn add_error(&mut self, error: ResolutionError) {
        self.errors.push(error);
    }
}