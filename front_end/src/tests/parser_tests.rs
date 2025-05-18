use crate::parser::Parser;
use crate::ast::{Statement, Expression};
use crate::token::{self, TokenType};
use crate::types::{Type, Permission};

#[test]
fn test_parse_variable_declaration_with_type() {
    // Use just one permission modifier for now
    let source = "reads x: Int = 42";
    
    // Add debug prints to understand what's happening
    println!("Test source: {}", source);
    
    let mut parser = Parser::from_source(&source.to_string());

    
    let statements = parser.parse_statements();
    println!("Parsed statements: {}", statements.len());
    
    assert_eq!(statements.len(), 1, "Should have parsed one statement");
    
    // Check that it's a variable declaration
    match &statements[0] {
        Statement::Declaration { name, initializer, typ } => {
            // Check the variable name
            assert_eq!(name, "x");
            
            // Check the type - make sure Type::Int is defined in your enum
            assert_eq!(typ.base_type, Type::Int);
            assert_eq!(typ.permissions.len(), 1);
            assert!(typ.permissions.contains(&Permission::Reads));
            
            // Check the initializer value
            match initializer {
                Some(Expression::Number(val)) => assert_eq!(*val, 42),
                _ => panic!("Expected number initializer"),
            }
        },
        _ => panic!("Expected variable declaration"),
    }
}

// Add a second test for multiple permissions when that's implemented
#[test]
fn test_multiple_permissions() {
    let source = "reads write x: Int = 42";
    let mut parser = Parser::from_source(&source.to_string());
    
    let statements = parser.parse_statements();
    assert_eq!(statements.len(), 1, "Should have parsed one statement");
    
    match &statements[0] {
        Statement::Declaration { typ, .. } => {
            assert_eq!(typ.permissions.len(), 2);
            assert!(typ.permissions.contains(&Permission::Reads));
            assert!(typ.permissions.contains(&Permission::Write));
        },
        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn test_parse_simple_function_declaration() {
    let source = "fn calculate(reads a: Int) -> Bool {\n  return a\n}";
    
    println!("Test source: {}", source);
    
    let mut parser = Parser::from_source(&source.to_string());
    let statements = parser.parse_statements();
    
    println!("Parsed statements: {}", statements.len());
    
    assert_eq!(statements.len(), 1, "Should have parsed one function declaration");
    
    // Basic checks
    match &statements[0] {
        Statement::Function { name, .. } => {
            assert_eq!(name, "calculate", "Function should be named 'calculate'");
            println!("Function declaration parsed successfully!");
        },
        _ => panic!("Expected function declaration"),
    }
}

#[test]
fn test_read_write_and_anon_function() {
    let source = "read write a: Int = 32\nfn add() -> Bool {\n  return 1\n}";
    
    println!("Test source: {}", source);
    
    let mut parser = Parser::from_source(&source.to_string());
    let statements = parser.parse_statements();
    
    println!("Parsed statements: {}", statements.len());
    
    // We should have two statements: a variable declaration and a function declaration
    assert_eq!(statements.len(), 2, "Should have parsed two statements");
    
    // First statement: Variable declaration with read and write permissions
    match &statements[0] {
        Statement::Declaration { name, typ, initializer } => {
            // Check name
            assert_eq!(name, "a", "Variable should be named 'a'");
            
            // Check type
            assert_eq!(typ.base_type, Type::Int, "Variable should have type Int");
            
            // Check permissions
            assert_eq!(typ.permissions.len(), 2, "Variable should have 2 permissions");
            assert!(typ.permissions.contains(&Permission::Read), 
                   "Variable should have Read permission");
            assert!(typ.permissions.contains(&Permission::Write), 
                   "Variable should have Write permission");
            
            // Check initializer
            match initializer {
                Some(Expression::Number(val)) => assert_eq!(*val, 32),
                _ => panic!("Expected number initializer with value 32"),
            }
            
            println!("Variable declaration parsed successfully!");
        },
        _ => panic!("Expected variable declaration as first statement"),
    }
    
    // Second statement: Named function declaration
    match &statements[1] {
        Statement::Function { name, params, body, return_type, is_behavior } => {
            // Check function name
            assert_eq!(name, "add", "Function should be named 'add'");
            
            // Check it's not a behavior
            assert_eq!(*is_behavior, false, "Should not be a behavior method");
            
            // Check that it has no parameters
            assert_eq!(params.len(), 0, "Function should have no parameters");
            
            // Check return type
            assert!(return_type.is_some(), "Function should have a return type");
            if let Some(ret_type) = return_type {
                assert_eq!(ret_type.base_type, Type::Bool, 
                          "Return type should be Bool");
            }
            
            // Check function body
            assert_eq!(body.len(), 1, "Function body should have 1 statement");
            
            // Check the return statement
            match &body[0] {
                Statement::Return(expr) => {
                    // Should return the number 1
                    match expr {
                        Expression::Number(val) => assert_eq!(*val, 1),
                        _ => panic!("Expected number 1 in return statement"),
                    }
                },
                _ => panic!("Expected return statement in function body"),
            }
            
            println!("Function declaration parsed successfully!");
        },
        _ => panic!("Expected function declaration as second statement"),
    }
}

