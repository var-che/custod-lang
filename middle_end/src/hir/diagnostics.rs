//! Compiler diagnostic system
//!
//! This module provides a unified system for error reporting and suggestions
//! across all compiler phases.

use std::fmt;
use crate::hir::scope::ScopeError;
// Change this to use SourceLocation from scope instead of types
use crate::hir::scope::SourceLocation;

/// Severity level of a diagnostic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    /// Error - prevents compilation from succeeding
    Error,
    /// Warning - allows compilation but indicates potential issues
    Warning,
    /// Hint - suggestions for improvement
    Hint,
    /// Note - additional information
    Note,
}

/// A diagnostic message with source information and suggestions
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Severity level
    pub level: DiagnosticLevel,
    
    /// Primary message
    pub message: String,
    
    /// Optional detailed explanation
    pub details: Option<String>,
    
    /// Source location
    pub location: Option<SourceLocation>,
    
    /// Optional suggestion for fixing the issue
    pub suggestion: Option<String>,
    
    /// Related diagnostic messages
    pub notes: Vec<Diagnostic>,
    
    /// Source code context (line with error highlighted)
    pub context: Option<String>,
}

impl Diagnostic {
    /// Create a new error diagnostic
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message: message.into(),
            details: None,
            location: None,
            suggestion: None,
            notes: Vec::new(),
            context: None, // Add this field
        }
    }
    
    /// Create a new warning diagnostic
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            message: message.into(),
            details: None,
            location: None,
            suggestion: None,
            notes: Vec::new(),
            context: None, // Add this field
        }
    }
    
    /// Create a new note diagnostic
    pub fn note(message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Note,
            message: message.into(),
            details: None,
            location: None,
            suggestion: None,
            notes: Vec::new(),
            context: None, // Add this field
        }
    }
    
    /// Add a source location to this diagnostic
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }
    
    /// Add detailed explanation
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
    
    /// Add a suggestion for fixing the issue
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
    
    /// Add a related note
    pub fn with_note(mut self, note: Diagnostic) -> Self {
        self.notes.push(note);
        self
    }
    
    /// Add source code context to the diagnostic
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format level prefix and message more concisely
        match self.level {
            DiagnosticLevel::Error => write!(f, "error: ")?,
            DiagnosticLevel::Warning => write!(f, "warning: ")?,
            DiagnosticLevel::Hint => write!(f, "hint: ")?,
            DiagnosticLevel::Note => write!(f, "note: ")?,
        }
        writeln!(f, "{}", self.message)?;
        
        // Write location and source context if available
        if let Some(loc) = &self.location {
            writeln!(f, " --> {}:{}:{}", loc.file, loc.line, loc.column)?;
            
            if let Some(context) = &self.context {
                write!(f, "{}", context)?;
            }
        }
        
        // Write details if available (optional)
        if let Some(details) = &self.details {
            writeln!(f, "{}", details)?;
        }
        
        // Write suggestion in a more concise format
        if let Some(suggestion) = &self.suggestion {
            writeln!(f, "suggestion: {}", suggestion)?;
        }
        
        // Write related notes (if any)
        for note in &self.notes {
            write!(f, "{}", note)?;
        }
        
        Ok(())
    }
}

/// A reporter that collects diagnostics
#[derive(Debug)]
pub struct DiagnosticReporter {
    /// All diagnostics collected
    pub diagnostics: Vec<Diagnostic>,
    
    /// Count of errors
    pub error_count: usize,
    
    /// Count of warnings
    pub warning_count: usize,
    
    /// Source code for context in error messages
    pub source_code: Option<String>,
}

impl DiagnosticReporter {
    /// Create a new reporter
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
            source_code: None,
        }
    }
    
    /// Create a new reporter with source code information
    pub fn with_source(source: &str) -> Self {
        let mut reporter = Self::new();
        reporter.source_code = Some(source.to_string());
        reporter
    }
    
    /// Create a new reporter from scope errors
    pub fn from_scope_errors(scope_errors: Vec<ScopeError>) -> Self {
        let mut reporter = Self::new();
        reporter.add_scope_errors(&scope_errors);
        reporter
    }
    
    /// Create from scope errors with source code
    pub fn from_scope_errors_with_source(scope_errors: Vec<ScopeError>, source: String) -> Self {
        let mut reporter = Self::new();
        reporter.source_code = Some(source);
        reporter.add_scope_errors(&scope_errors);
        reporter
    }
    
    /// Add a diagnostic
    pub fn add(&mut self, diagnostic: Diagnostic) {
        match diagnostic.level {
            DiagnosticLevel::Error => self.error_count += 1,
            DiagnosticLevel::Warning => self.warning_count += 1,
            _ => {}
        }
        
        self.diagnostics.push(diagnostic);
    }
    
    /// Report all diagnostics
    pub fn report(&self) -> String {
        let mut output = String::new();
        
        for diagnostic in &self.diagnostics {
            output.push_str(&format!("{}\n\n", diagnostic));
        }
        
        output.push_str(&format!("{} error(s), {} warning(s) emitted\n",
            self.error_count, self.warning_count));
            
        output
    }
    
    /// Add scope errors to diagnostics with improved formatting
    pub fn add_scope_errors(&mut self, errors: &[ScopeError]) {
        for error in errors {
            match error {
                ScopeError::NotFound { name, location } => {
                    // Get location information
                    let loc = location.clone().unwrap_or_else(|| 
                        SourceLocation { line: 1, column: 1, file: "input".to_string() }
                    );
                    
                    // Create a more concise error message
                    let mut diag = Diagnostic::error(format!("Cannot find '{}' in this scope", name));
                    
                    // Add location
                    diag = diag.with_location(loc.clone());
                    
                    // Extract code context
                    if let Some(context) = self.extract_code_context(loc.line, loc.column) {
                        diag = diag.with_context(context);
                    }
                    
                    // Add suggestion
                    diag = diag.with_suggestion(format!("Make sure '{}' is declared before use", name));
                    
                    self.add(diag);
                },
                ScopeError::AlreadyDefined { name, previous } => {
                    let location = previous.clone().unwrap_or_else(|| 
                        SourceLocation { line: 1, column: 1, file: "unknown".to_string() }
                    );
                    
                    let mut diag = Diagnostic::error(format!("Variable '{}' is already defined", name))
                        .with_suggestion(format!("Consider using a different name, such as '{}_2'", name))
                        .with_location(location.clone());
                        
                    if let Some(context) = self.extract_code_context(location.line, location.column) {
                        diag = diag.with_context(context);
                    }
                    
                    self.add(diag);
                },
                ScopeError::Shadowing { name, previous } => {
                    let location = previous.clone().unwrap_or_else(|| 
                        SourceLocation { line: 1, column: 1, file: "unknown".to_string() }
                    );
                    
                    let mut diag = Diagnostic::warning(format!("Variable '{}' shadows a previous definition", name))
                        .with_suggestion(format!("Consider renaming to avoid confusion"))
                        .with_location(location.clone());
                        
                    if let Some(context) = self.extract_code_context(location.line, location.column) {
                        diag = diag.with_context(context);
                    }
                    
                    self.add(diag);
                },
            }
        }
    }
    
    /// Improve error display with source code context
    pub fn add_scope_errors_with_source(&mut self, errors: &[ScopeError], source: &str) {
        self.source_code = Some(source.to_string());
        self.add_scope_errors(errors);
    }
    
    /// Extract source code context for a given line and column - improved formatting
    fn extract_code_context(&self, line: usize, column: usize) -> Option<String> {
        if let Some(source) = &self.source_code {
            let lines: Vec<&str> = source.lines().collect();
            
            // Find the actual line index (adjusting for blank lines at start)
            let mut actual_line = line;
            if actual_line >= lines.len() {
                // If line is out of range, look for the nearest valid line
                for i in (0..lines.len()).rev() {
                    if !lines[i].trim().is_empty() {
                        actual_line = i + 1;  // Convert to 1-based line number
                        break;
                    }
                }
            }
            
            // Ensure we're within bounds and adjust for 0-based indexing
            let index = actual_line.saturating_sub(1);
            if index < lines.len() {
                let line_content = lines[index].trim_start(); // Remove leading whitespace
                
                // Format the line with just the number and content - more concise
                let mut result = format!("{} | {}\n", actual_line, line_content);
                
                // Calculate tilde position & length
                let adjusted_col = column.saturating_sub(lines[index].len() - line_content.len());
                let token_len = extract_token_length(line_content, adjusted_col.saturating_sub(1));
                
                // Just show the tilde underline with no extra padding
                result.push_str(&format!("    {}{}\n", 
                    " ".repeat(adjusted_col.saturating_sub(1)), 
                    "~".repeat(token_len)
                ));
                
                return Some(result);
            }
        }
        None
    }
    
    /// Check if any errors were reported
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }
}

/// Helper function to calculate token length
fn extract_token_length(text: &str, pos: usize) -> usize {
    if pos >= text.len() {
        return 1;
    }
    
    // Try to find the token length
    let mut end = pos;
    while end < text.len() && is_token_char(text.chars().nth(end).unwrap_or(' ')) {
        end += 1;
    }
    
    // Ensure we return at least 1 for length
    (end - pos).max(1)
}

/// Helper function to check if a character is part of a token/identifier
fn is_token_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
