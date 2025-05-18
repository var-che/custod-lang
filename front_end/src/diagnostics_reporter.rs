use crate::symbol_table::{ResolutionError};
use crate::source_manager::SourceManager;

pub struct DiagnosticReporter {
    pub source_manager: SourceManager,
}

impl DiagnosticReporter {
    pub fn new(source_manager: SourceManager) -> Self {
        Self { source_manager }
    }
    
    pub fn report_error(&self, error: &ResolutionError) -> String {
        match error {
            ResolutionError::DuplicateSymbol { name, first, second } => {
                let mut output = format!("error[E0001]: duplicate definition of `{}`\n", name);
                
                // First definition - use accurate line/column from token
                let first_loc = format!("{}:{}", first.start_line, first.start_column);
                output.push_str(&format!("--> {}\n", first_loc));
                
                // Get the snippet from source manager with proper position
                let first_snippet = self.source_manager.get_snippet(first);
                output.push_str(&format!("{}\n", first_snippet));
                output.push_str(" | first definition here\n\n");
                
                // Second definition
                let second_loc = format!("{}:{}", second.start_line, second.start_column);
                output.push_str(&format!("--> {}\n", second_loc));
                
                let second_snippet = self.source_manager.get_snippet(second);
                output.push_str(&format!("{}\n", second_snippet));
                output.push_str(" | redefinition here\n\n");
                
                output.push_str("note: each variable must be defined only once per scope");
                
                output
            },
            ResolutionError::UndefinedSymbol { name, span } => {
                let mut output = format!("error[E0002]: undefined variable `{}`\n", name);
                
                let loc = format!("{}:{}", span.start_line, span.start_column);
                output.push_str(&format!("--> {}\n", loc));
                
                let snippet = self.source_manager.get_snippet(span);
                output.push_str(&format!("{}\n", snippet));
                output.push_str(" | variable not found in this scope\n\n");
                
                output.push_str("help: consider declaring the variable before using it");
                
                output
            },
            ResolutionError::ImmutableAssignment { name, span, declaration_span } => {
                let mut output = format!("error[E0003]: cannot assign to immutable variable `{}`\n", name);
                
                // Show where the immutable assignment happened
                let loc = format!("{}:{}", span.start_line, span.start_column);
                output.push_str(&format!("--> {}\n", loc));
                
                let snippet = self.source_manager.get_snippet(span);
                output.push_str(&format!("{}\n", snippet));
                output.push_str(" | cannot assign to immutable variable\n\n");
                
                // If we have the declaration span, show it too
                if let Some(decl_span) = declaration_span {
                    let decl_loc = format!("{}:{}", decl_span.start_line, decl_span.start_column);
                    output.push_str(&format!("--> {}\n", decl_loc));
                    
                    let decl_snippet = self.source_manager.get_snippet(decl_span);
                    output.push_str(&format!("{}\n", decl_snippet));
                    output.push_str(" | variable declared here without write permission\n\n");
                }
                
                output.push_str("help: add 'write' or 'writes' permission to make the variable mutable");
                
                output
            },
            ResolutionError::PermissionViolation { name, required, provided, span, declaration_span } => {
                let mut output = format!("error[E0004]: permission violation for variable `{}`\n", name);
                
                // Show where the violation happened
                let loc = format!("{}:{}", span.start_line, span.start_column);
                output.push_str(&format!("--> {}\n", loc));
                
                let snippet = self.source_manager.get_snippet(span);
                output.push_str(&format!("{}\n", snippet));
                output.push_str(&format!(" | requires permission '{}' but found '{}'\n\n", required, provided));
                
                // If we have the declaration span, show it too
                if let Some(decl_span) = declaration_span {
                    let decl_loc = format!("{}:{}", decl_span.start_line, decl_span.start_column);
                    output.push_str(&format!("--> {}\n", decl_loc));
                    
                    let decl_snippet = self.source_manager.get_snippet(decl_span);
                    output.push_str(&format!("{}\n", decl_snippet));
                    output.push_str(&format!(" | variable declared with '{}' permission\n\n", provided));
                }
                
                output.push_str(&format!("help: update the variable declaration to include '{}' permission", required));
                
                output
            },
            ResolutionError::ReadAccessViolation { name, span, declaration_span, target_permission } => {
                let mut output = format!("error[E0005]: cannot directly assign reads variable `{}` to {} variable\n", name, target_permission);
                
                // Show where the violation happened
                let loc = format!("{}:{}", span.start_line, span.start_column);
                output.push_str(&format!("--> {}\n", loc));
                
                let snippet = self.source_manager.get_snippet(span);
                output.push_str(&format!("{}\n", snippet));
                output.push_str(" | cannot directly assign reads variable without clone or peak\n\n");
                
                // If we have the declaration span, show it too
                if let Some(decl_span) = declaration_span {
                    let decl_loc = format!("{}:{}", decl_span.start_line, decl_span.start_column);
                    output.push_str(&format!("--> {}\n", decl_loc));
                    
                    let decl_snippet = self.source_manager.get_snippet(decl_span);
                    output.push_str(&format!("{}\n", decl_snippet));
                    output.push_str(" | variable declared with 'reads' permission\n\n");
                }
                
                // Always provide both solutions regardless of target permission
                output.push_str("help: you have two options to fix this issue:\n\n");
                output.push_str("Option 1: use 'clone' to create a deep copy of the variable\n");
                output.push_str(&format!("  reads {} = clone {}\n\n", "c", name)); // Using "c" as example variable name
                
                output.push_str("Option 2: use 'peak' with 'read' permission to create a read-only reference\n");
                output.push_str(&format!("  read {} = peak {}\n", "c", name)); // Using "c" as example
                
                output
            },
            ResolutionError::TypeMismatch { expected, found, span, context } => {
                let mut output = format!("error[E0006]: type mismatch {}\n", context);
                
                let loc = format!("{}:{}", span.start_line, span.start_column);
                output.push_str(&format!("--> {}\n", loc));
                
                let snippet = self.source_manager.get_snippet(span);
                output.push_str(&format!("{}\n", snippet));
                
                // Show the expected vs. found types
                output.push_str(&format!("   | expected type `{}`, found `{}`\n\n", expected, found));
                
                // Add helpful suggestion
                if context.contains("return") {
                    output.push_str("help: ensure the expression's type matches the function's return type");
                } else {
                    output.push_str("help: ensure the types match with what is expected");
                }
                
                output
            },
        }
    }
}