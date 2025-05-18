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
        assert!(formatted.contains("cannot assign to immutable variable"), 
                "Error message should mention immutable variable");
    }
}

#[test]
fn test_write_permission_violation() {
    let source = r#"
    read write counter: Int = 5
    write c: Int = counter
    "#;
    
    let mut source_manager = SourceManager::new();
    source_manager.set_default_source(source);
    
    let mut parser = Parser::from_source(source);
    let statements = parser.parse_statements();
    
    // Print for debugging
    println!("Parsed statements: {:?}", statements);
    println!("Symbol table errors: {:?}", parser.get_symbol_table().get_errors());
    
    let errors = parser.get_symbol_table().get_errors();
    assert!(!errors.is_empty(), "Should have caught a permission violation error");
    
    // Test error formatting
    let reporter = DiagnosticReporter::new(source_manager);
    for error in errors {
        let formatted = reporter.report_error(error);
        println!("{}", formatted);
        assert!(formatted.contains("permission violation"), 
                "Error message should mention permission violation");
        assert!(formatted.contains("write"), 
                "Error message should mention write permission");
        assert!(formatted.contains("counter"), 
                "Error message should reference the source variable");
    }
}

#[test]
fn test_reads_assignment_without_clone() {
    let source = "
    reads counter: Int = 5
    reads c = counter
    ";
    
    let mut source_manager = SourceManager::new();
    source_manager.set_default_source(source);
    
    let mut parser = Parser::from_source(source);
    let _ = parser.parse_statements();
    
    let errors = parser.get_symbol_table().get_errors();
    assert!(!errors.is_empty(), "Should have caught a read access violation error");
    
    // Test error formatting
    let reporter = DiagnosticReporter::new(source_manager);
    for error in errors {
        let formatted = reporter.report_error(error);
        println!("{}", formatted);
        assert!(formatted.contains("cannot directly assign reads variable"), 
                "Error message should mention read access violation");
        assert!(formatted.contains("clone"), 
                "Error message should suggest using clone");
    }
}

#[test]
fn test_read_assignment_without_peak() {
    let source = "
    reads counter: Int = 5
    read c = counter
    ";
    
    let mut source_manager = SourceManager::new();
    source_manager.set_default_source(source);
    
    let mut parser = Parser::from_source(source);
    let _ = parser.parse_statements();
    
    let errors = parser.get_symbol_table().get_errors();
    assert!(!errors.is_empty(), "Should have caught a read access violation error");
    
    // Test error formatting
    let reporter = DiagnosticReporter::new(source_manager);
    for error in errors {
        let formatted = reporter.report_error(error);
        println!("{}", formatted);
        assert!(formatted.contains("cannot directly assign reads variable"), 
                "Error message should mention read access violation");
        assert!(formatted.contains("peak"), 
                "Error message should suggest using peak");
    }
}

#[test]
fn test_read_assignment_with_peak() {
    let source = "
    reads write counter: Int = 5
    read c = peak counter
    ";
    
    let mut source_manager = SourceManager::new();
    source_manager.set_default_source(source);
    
    let mut parser = Parser::from_source(source);
    let statements = parser.parse_statements();
    
    // Print the statements for debugging
    println!("Parsed statements: {:?}", statements);
    
    // Verify we have two statements
    assert_eq!(statements.len(), 2, "Should have parsed two statements");
    
    // No errors should be generated when using peak correctly
    let errors = parser.get_symbol_table().get_errors();
    assert!(errors.is_empty(), "Using peak should not produce any errors");
    
    // Verify the second statement is a declaration with the right name
    match &statements[1] {
        crate::ast::Statement::Declaration { name, typ, initializer } => {
            // Check the variable name
            assert_eq!(name, "c", "Variable should be named 'c'");
            
            // Check that initializer is a peak expression
            match initializer {
                Some(crate::ast::Expression::Peak(expr)) => {
                    // Check that the peak expression contains the counter variable
                    match &**expr {
                        crate::ast::Expression::Variable(var_name) => {
                            assert_eq!(var_name, "counter", "Should peak the 'counter' variable");
                        },
                        _ => panic!("Expected variable reference inside peak"),
                    }
                },
                _ => panic!("Expected peak expression as initializer, got: {:?}", initializer),
            }
            
            // Check permissions on the new variable
            assert_eq!(typ.permissions.len(), 1);
            assert!(typ.permissions.contains(&crate::types::Permission::Read),
                  "Variable should have Read permission");
        },
        _ => panic!("Expected variable declaration as second statement"),
    }
}


