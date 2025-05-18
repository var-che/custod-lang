use crate::symbol_table::{SymbolTable, Symbol, SymbolKind, Span, ResolutionError};
use crate::types::{Type, Permission, PermissionedType};
use crate::source_manager::SourceManager;
use crate::diagnostics_reporter::DiagnosticReporter;
use crate::parser::Parser;

#[test]
fn test_duplicate_variable_error() {
    let source = "
    reads x: Int = 5
    reads x: Int = 10 
    ";
    
    let mut source_manager = SourceManager::new();
    source_manager.set_default_source(source);
    
    let mut parser = Parser::from_source(source);
    let _ = parser.parse_statements(); // Will generate errors in the symbol table
    
    let errors = parser.get_symbol_table().get_errors();
    assert!(!errors.is_empty(), "Should have caught a duplicate symbol error");
    
    // Test error formatting
    let reporter = DiagnosticReporter::new(source_manager);
    for error in errors {
        let formatted = reporter.report_error(error);
        println!("{}", formatted); // Print for visual inspection
        assert!(formatted.contains("duplicate definition of `x`"), 
                "Error message should mention duplicate definition of variable x");
    }
}

#[test]
fn test_undefined_variable_error() {
    let source = "
    reads y = z 
    ";
    
    let mut source_manager = SourceManager::new();
    source_manager.set_default_source(source);
    
    let mut parser = Parser::from_source(source);
    let _ = parser.parse_statements();
    
    let errors = parser.get_symbol_table().get_errors();
    assert!(!errors.is_empty(), "Should have caught an undefined symbol error");
    
    // Test error formatting
    let reporter = DiagnosticReporter::new(source_manager);
    for error in errors {
        let formatted = reporter.report_error(error);
        println!("{}", formatted);
        assert!(formatted.contains("undefined variable `z`"), 
               "Error message should mention undefined variable");
    }
}

#[test]
fn test_immutable_assignment_error() {
    let source = "
    reads x: Int = 5
    x = 10 
    ";
    
    let mut source_manager = SourceManager::new();
    source_manager.set_default_source(source);
    
    let mut parser = Parser::from_source(source);
    let _ = parser.parse_statements();
    
    let errors = parser.get_symbol_table().get_errors();
    assert!(!errors.is_empty(), "Should have caught an immutable assignment error");
    
    // Test error formatting
    let reporter = DiagnosticReporter::new(source_manager);
    for error in errors {
        let formatted = reporter.report_error(error);
        println!("{}", formatted);
        assert!(formatted.contains("Cannot assign to immutable variable"), "Error message should mention immutable variable");
    }
}


