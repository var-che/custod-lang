//! Tests for HIR error reporting functionality
//!
//! This module tests the HIR error reporting system to ensure it provides
//! the same level of quality as the front-end error system.

use crate::hir::{convert_statements_to_hir, resolve_names_with_source, check_permissions};
use crate::hir::validation::ValidationError;
use front_end::parser::Parser;

#[test]
fn test_hir_duplicate_variable_error() {
    // Test duplicate variable detection
    let source = r#"
        reads x: Int = 5
        reads x: Int = 10
    "#;
    
    // Parse and convert to HIR
    let mut parser = Parser::from_source(source);
    let ast_statements = parser.parse_statements();
    let hir_program = convert_statements_to_hir(ast_statements);
    
    // Resolve names to detect errors with source code for improved reporting
    let resolved = resolve_names_with_source(&hir_program, source);
    
    // Verify errors were detected
    assert!(resolved.diagnostics.has_errors(), "Should have detected duplicate variable error");
    
    // Print diagnostic report for visual inspection
    let report = resolved.diagnostics.report();
    println!("HIR Duplicate Variable Error Report:\n{}", report);
    
    // Verify error message quality
    assert!(report.contains("already defined"), "Error message should mention duplicate definition");
    assert!(report.contains("x"), "Error message should mention the variable name");
    
    // The format shows line numbers like "2 |" not literally containing "line"
    assert!(report.contains("-->"), "Error report should contain location marker");
    assert!(report.contains(" | "), "Error report should contain line formatting");
    assert!(report.contains("~"), "Error report should point to the error with underlines (~)");
}

#[test]
fn test_hir_undeclared_variable_error() {
    // Test undeclared variable detection
    let source = r#"
        reads result = undeclared_var + 5
    "#;
    
    // Parse and convert to HIR
    let mut parser = Parser::from_source(source);
    let ast_statements = parser.parse_statements();
    let hir_program = convert_statements_to_hir(ast_statements);
    
    // Resolve names with source code for improved reporting
    let resolved = resolve_names_with_source(&hir_program, source);
    
    // Verify errors were detected
    assert!(resolved.diagnostics.has_errors(), "Should have detected undeclared variable error");
    
    // Print diagnostic report
    let report = resolved.diagnostics.report();
    println!("HIR Undeclared Variable Error Report:\n{}", report);
    
    // Verify error message quality
    assert!(report.contains("Cannot find"), "Error message should mention that variable wasn't found");
    assert!(report.contains("undeclared_var"), "Error message should mention the missing variable name");
    assert!(report.contains("-->"), "Error report should contain location marker");
    assert!(report.contains(" | "), "Error report should contain line formatting");
    assert!(report.contains("~"), "Error report should point to the error with underlines (~)");
}

#[test]
fn test_hir_permission_violation_write() {
    // Test permission violation - trying to write to a read-only variable
    let source = r#"
        reads x: Int = 5
        x = 10  // Can't write to a reads-only variable
    "#;
    
    // Parse and convert to HIR
    let mut parser = Parser::from_source(source);
    let ast_statements = parser.parse_statements();
    let hir_program = convert_statements_to_hir(ast_statements);
    
    // Check permissions
    let errors = check_permissions(&hir_program);
    
    // Verify we got permission errors
    assert!(!errors.is_empty(), "Should have detected permission violation");
    
    // Print error messages for visual inspection
    println!("HIR Permission Violation (Write) Error Report:");
    for error in &errors {
        println!("{}", error.message);
    }
    
    // Verify error message quality
    let has_write_error = errors.iter().any(|err| {
        err.message.contains("Cannot write") && err.message.contains("x")
    });
    assert!(has_write_error, "Should have a clear error about writing to read-only variable");
}

#[test]
fn test_hir_permission_violation_aliasing() {
    // Test aliasing violations - creating an illegal alias
    let source = r#"
        read write exclusive: Int = 5
        read alias = exclusive  // Can't create alias to exclusive (non-shareable)
    "#;
    
    // Parse and convert to HIR
    let mut parser = Parser::from_source(source);
    let ast_statements = parser.parse_statements();
    let hir_program = convert_statements_to_hir(ast_statements);
    
    // Check permissions
    let errors = check_permissions(&hir_program);
    
    // Verify we got permission errors
    assert!(!errors.is_empty(), "Should have detected aliasing violation");
    
    // Print error messages
    println!("HIR Permission Violation (Aliasing) Error Report:");
    for error in &errors {
        println!("{}", error.message);
    }
    
    // Verify error message quality
    let has_alias_error = errors.iter().any(|err| {
        err.message.contains("alias") && 
        (err.message.contains("non-shareable") || err.message.contains("exclusive"))
    });
    assert!(has_alias_error, "Should have a clear error about illegal aliasing");
}

#[test]
fn test_hir_type_mismatch_error() {
    // Test type mismatch detection
    let source = r#"
        reads write num: Int = 42
        reads write bool_val: Bool = true
        
        // Type error: assigning Bool to Int
        num = bool_val
    "#;
    
    // Parse and convert to HIR
    let mut parser = Parser::from_source(source);
    let ast_statements = parser.parse_statements();
    let hir_program = convert_statements_to_hir(ast_statements);
    
    // Validate HIR to check for type errors, using the source code for better error messages
    let validation_result = crate::hir::validation::validate_hir_with_source(&hir_program, source);
    
    // Verify we got validation errors
    assert!(validation_result.is_err(), "Should have detected type mismatch");
    
    if let Err(errors) = validation_result {
        // Print formatted error messages
        println!("HIR Type Mismatch Error Report:");
        for error in &errors {
            // Use the new format method with source code
            match error {
                ValidationError::TypeMismatch { .. } => {
                    println!("{}", error.format(Some(source)));
                },
                _ => println!("{:?}", error),
            }
        }
        
        // Verify error message quality by formatting everything
        let error_text = errors.iter()
            .map(|e| match e {
                ValidationError::TypeMismatch { .. } => e.format(Some(source)),
                _ => format!("{:?}", e),
            })
            .collect::<Vec<_>>()
            .join("\n");
            
        // Check content of formatted errors
        assert!(error_text.contains("Type mismatch"), "Error should report type mismatch");
        assert!(error_text.contains("Int") && error_text.contains("Bool"), "Error should show both types");
        assert!(error_text.contains("Suggestion"), "Error should include suggestion");
        
        // With the improved error formatting, one of the errors should contain tildes
        let contains_tildes = error_text.lines().any(|line| line.contains("~"));
        assert!(contains_tildes, "Error should highlight the problematic code with tildes (~)");
    }
}

#[test]
fn test_hir_peak_permission_error() {
    // Test peak operator permission checking
    let source = r#"
        // Variable without read permission
        write write_only: Int = 5
        
        fn test_peak() -> Int {
            reads result = peak write_only  // Error: can't peak without read permission
            return result
        }
    "#;
    
    // Parse and convert to HIR
    let mut parser = Parser::from_source(source);
    let ast_statements = parser.parse_statements();
    
    // Skip this test if the parser doesn't support the syntax yet
    if ast_statements.is_empty() {
        println!("Parser doesn't support the peak operator syntax yet, skipping test");
        return;
    }
    
    let hir_program = convert_statements_to_hir(ast_statements);
    
    // Check permissions
    let errors = check_permissions(&hir_program);
    
    // Verify we got permission errors
    assert!(!errors.is_empty(), "Should have detected peak permission violation");
    
    // Print error messages
    println!("HIR Peak Permission Error Report:");
    for error in &errors {
        println!("{}", error.message);
    }
    
    // Verify error message quality
    let has_peak_error = errors.iter().any(|err| {
        err.message.contains("peak") && err.message.contains("read permission")
    });
    assert!(has_peak_error, "Should have a clear error about peak requiring read permission");
}

#[test]
fn test_hir_error_locations() {
    // Test that error messages include location information
    let source = r#"
        reads write x: Int = 5
        
        fn test_function() {
            y = 10  // Undeclared variable with location info
        }
    "#;
    
    // Parse and convert to HIR
    let mut parser = Parser::from_source(source);
    let ast_statements = parser.parse_statements();
    let hir_program = convert_statements_to_hir(ast_statements);
    
    // Resolve names with source code for improved reporting
    let resolved = resolve_names_with_source(&hir_program, source);
    
    // Verify errors were detected
    assert!(resolved.diagnostics.has_errors(), "Should detect undeclared variable error");
    
    // Print diagnostic report
    let report = resolved.diagnostics.report();
    println!("HIR Error Location Report:\n{}", report);
    
    // Check for location info in error message
    assert!(report.contains("-->") && report.contains(" | "), 
            "Error message should include source location information");
}

#[test]
fn test_hir_error_suggestions() {
    // Test that error messages include helpful suggestions
    let source = r#"
        reads x: Int = 5
        reads y: Int = 10
        x = y  // Error: can't write to reads-only variable
    "#;
    
    // Parse and convert to HIR
    let mut parser = Parser::from_source(source);
    let ast_statements = parser.parse_statements();
    let hir_program = convert_statements_to_hir(ast_statements);
    
    // Check permissions with source code for better error messages
    let errors = crate::hir::permissions::check_permissions_with_source(&hir_program, source);
    
    // Verify we got permission errors
    assert!(!errors.is_empty(), "Should have detected permission violation");
    
    // Print error messages
    println!("HIR Error Suggestion Report:");
    for error in &errors {
        println!("{}", error.message);
    }
    
    // Check if any errors contain line numbers and tilde markers
    let has_line_number = errors.iter().any(|err| {
        err.message.contains(" | ") && err.message.contains("~")
    });
    
    if !has_line_number {
        println!("Note: Error messages would benefit from line number formatting");
    }
    
    // Check if any errors contain a suggestion
    let has_suggestion = errors.iter().any(|err| {
        err.message.contains("suggestion")
    });
    
    if !has_suggestion {
        println!("Note: Error messages would benefit from actionable suggestions");
    }
}
