//! Compiler diagnostic system
//!
//! This module provides a unified system for error reporting and suggestions
//! across all compiler phases.

use std::fmt;
use crate::hir::scope::ScopeError;
use crate::hir::validation::ValidationError;
use crate::hir::types::SourceLocation;

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
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format level prefix
        match self.level {
            DiagnosticLevel::Error => write!(f, "error: ")?,
            DiagnosticLevel::Warning => write!(f, "warning: ")?,
            DiagnosticLevel::Hint => write!(f, "hint: ")?,
            DiagnosticLevel::Note => write!(f, "note: ")?,
        }
        
        // Write the main message
        writeln!(f, "{}", self.message)?;
        
        // Write location if available
        if let Some(loc) = &self.location {
            writeln!(f, " --> {}:{}:{}", 
                    loc.file_id, 
                    loc.start.line, 
                    loc.start.column)?;
        }
        
        // Write detailed explanation if available
        if let Some(details) = &self.details {
            writeln!(f, "\n{}", details)?;
        }
        
        // Write suggestion if available
        if let Some(suggestion) = &self.suggestion {
            writeln!(f, "\nsuggestion: {}", suggestion)?;
        }
        
        // Write related notes
        for note in &self.notes {
            write!(f, "\n{}", note)?;
        }
        
        Ok(())
    }
}

/// A reporter that collects diagnostics
pub struct DiagnosticReporter {
    /// All diagnostics collected
    pub diagnostics: Vec<Diagnostic>,
    
    /// Count of errors
    pub error_count: usize,
    
    /// Count of warnings
    pub warning_count: usize,
}

impl DiagnosticReporter {
    /// Create a new reporter
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
        }
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
    
    /// Convert scope errors to diagnostics
    pub fn add_scope_errors(&mut self, errors: &[ScopeError]) {
        for error in errors {
            match error {
                ScopeError::AlreadyDefined { name, previous } => {
                    let mut diag = Diagnostic::error(format!("'{}' is already defined", name))
                        .with_suggestion(format!("Consider using a different name for this variable"));
                        
                    if let Some(loc) = previous {
                        let note = Diagnostic::note(format!("'{}' was previously defined here", name))
                            .with_location(loc.clone());
                        diag = diag.with_note(note);
                    }
                    
                    self.add(diag);
                },
                
                ScopeError::Shadowing { name, previous } => {
                    let mut diag = Diagnostic::warning(format!("'{}' shadows a previous definition", name))
                        .with_suggestion(format!("Consider renaming this variable to avoid confusion"));
                        
                    if let Some(loc) = previous {
                        let note = Diagnostic::note(format!("Previous definition is here", name))
                            .with_location(loc.clone());
                        diag = diag.with_note(note);
                    }
                    
                    self.add(diag);
                },
                
                ScopeError::NotFound { name } => {
                    self.add(Diagnostic::error(format!("Cannot find '{}' in this scope", name))
                        .with_suggestion(format!("Make sure '{}' is declared before use", name)));
                },
            }
        }
    }
    
    /// Convert validation errors to diagnostics
    pub fn add_validation_errors(&mut self, errors: &[ValidationError]) {
        for error in errors {
            match error {
                ValidationError::UndefinedVariable { name, context } => {
                    self.add(Diagnostic::error(format!("Undefined variable '{}' in {}", name, context))
                        .with_suggestion(format!("Declare '{}' before using it", name)));
                },
                
                ValidationError::TypeMismatch { expected, actual, context } => {
                    self.add(Diagnostic::error(format!("Type mismatch in {}", context))
                        .with_details(format!("Expected type '{}', found '{}'", expected, actual))
                        .with_suggestion("Make sure the types match"));
                },
                
                ValidationError::PermissionError { message } => {
                    self.add(Diagnostic::error(format!("Permission error: {}", message))
                        .with_suggestion("Check the permissions of the variables involved"));
                },
                
                ValidationError::Other(message) => {
                    self.add(Diagnostic::error(message.clone()));
                },
            }
        }
    }
    
    /// Check if any errors were reported
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }
}

/// Update the main name resolver to use the diagnostic system
pub fn convert_resolved_names_to_diagnostics(resolved: &crate::hir::name_resolver::ResolvedNames) -> DiagnosticReporter {
    let mut reporter = DiagnosticReporter::new();
    reporter.add_scope_errors(&resolved.errors);
    reporter
}

/// Update the validator to use the diagnostic system
pub fn convert_validation_result_to_diagnostics(result: &Result<(), Vec<ValidationError>>) -> DiagnosticReporter {
    let mut reporter = DiagnosticReporter::new();
    
    if let Err(errors) = result {
        reporter.add_validation_errors(errors);
    }
    
    reporter
}
