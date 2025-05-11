use crate::parser::Parser;
use crate::ast::{Statement, Expression};
use crate::token::TokenType;
use crate::lexer::Lexer;
use crate::types::{Permission, Type};

// Helper function to create a parser from source
fn create_parser(source: &str) -> Parser {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    Parser::new(tokens)
}

#[test]
fn test_parse_simple_number_expression() {
    let source = "42";
    let mut parser = create_parser(source);
    
    let result = parser.parse_expression();
    assert!(result.is_ok(), "Failed to parse number expression");
    
    if let Ok(expr) = result {
        match expr {
            Expression::Number(value) => {
                assert_eq!(value, 42, "Number value should be 42");
            },
            _ => panic!("Expected a number expression"),
        }
    }
}

#[test]
fn test_parse_simple_addition() {
    let source = "5 + 3";
    let mut parser = create_parser(source);
    
    let result = parser.parse_expression();
    assert!(result.is_ok(), "Failed to parse addition expression");
    
    if let Ok(expr) = result {
        match expr {
            Expression::Binary { left, operator, right } => {
                // Verify the operator is +
                assert_eq!(operator, TokenType::Plus, "Operator should be '+'");
                
                // Verify left operand is 5
                if let Expression::Number(value) = *left {
                    assert_eq!(value, 5, "Left operand should be 5");
                } else {
                    panic!("Left operand should be a number");
                }
                
                // Verify right operand is 3
                if let Expression::Number(value) = *right {
                    assert_eq!(value, 3, "Right operand should be 3");
                } else {
                    panic!("Right operand should be a number");
                }
            },
            _ => panic!("Expected a binary expression"),
        }
    }
}

#[test]
fn test_parse_compound_addition() {
    let source = "5 + 5 + 7";
    let mut parser = create_parser(source);
    
    let result = parser.parse_expression();
    assert!(result.is_ok(), "Failed to parse compound addition expression");
    
    if let Ok(expr) = result {
        // For "5 + 5 + 7", we expect a binary tree where:
        // - The root is a Binary with operator '+'
        // - The left child is another Binary: (5 + 5)
        // - The right child is the number 7
        
        match expr {
            Expression::Binary { left, operator, right } => {
                // Verify the top-level operator is +
                assert_eq!(operator, TokenType::Plus, "Top-level operator should be '+'");
                
                // Right operand should be 7
                if let Expression::Number(value) = *right {
                    assert_eq!(value, 7, "Right operand should be 7");
                } else {
                    panic!("Right operand should be a number");
                }
                
                // Left operand should be another binary expression: 5 + 5
                match *left {
                    Expression::Binary { left: inner_left, operator: inner_op, right: inner_right } => {
                        assert_eq!(inner_op, TokenType::Plus, "Inner operator should be '+'");
                        
                        // Verify inner left operand is 5
                        if let Expression::Number(value) = *inner_left {
                            assert_eq!(value, 5, "Inner left operand should be 5");
                        } else {
                            panic!("Inner left operand should be a number");
                        }
                        
                        // Verify inner right operand is 5
                        if let Expression::Number(value) = *inner_right {
                            assert_eq!(value, 5, "Inner right operand should be 5");
                        } else {
                            panic!("Inner right operand should be a number");
                        }
                    },
                    _ => panic!("Left operand should be a binary expression"),
                }
            },
            _ => panic!("Expected a binary expression"),
        }
    }
}

#[test]
fn test_parse_variable_declaration_with_compound_expression() {
    let source = "reads c = 4 + 6 + 6";
    let mut parser = create_parser(source);
    
    let result = parser.parse_statement();
    assert!(result.is_ok(), "Failed to parse variable declaration statement");
    
    if let Ok(stmt) = result {
        match stmt {
            Statement::Declaration { name, typ, initializer } => {
                // Verify variable name
                assert_eq!(name, "c", "Variable name should be 'c'");
                
                // Verify permission type
                assert_eq!(typ.permissions, vec![Permission::Reads], "Permission should be 'Reads'");
                
                // Verify initializer expression
                if let Some(expr) = initializer {
                    // Should be a compound binary expression (4 + 6) + 6
                    match expr {
                        Expression::Binary { left, operator, right } => {
                            // Top level should be addition
                            assert_eq!(operator, TokenType::Plus, "Top-level operator should be '+'");
                            
                            // Right side should be 6
                            if let Expression::Number(value) = *right {
                                assert_eq!(value, 6, "Right operand should be 6");
                            } else {
                                panic!("Right operand should be a number");
                            }
                            
                            // Left side should be another binary: 4 + 6
                            match *left {
                                Expression::Binary { left: inner_left, operator: inner_op, right: inner_right } => {
                                    assert_eq!(inner_op, TokenType::Plus, "Inner operator should be '+'");
                                    
                                    // Inner left should be 4
                                    if let Expression::Number(value) = *inner_left {
                                        assert_eq!(value, 4, "Inner left operand should be 4");
                                    } else {
                                        panic!("Inner left operand should be a number");
                                    }
                                    
                                    // Inner right should be 6
                                    if let Expression::Number(value) = *inner_right {
                                        assert_eq!(value, 6, "Inner right operand should be 6");
                                    } else {
                                        panic!("Inner right operand should be a number");
                                    }
                                },
                                _ => panic!("Left operand should be a binary expression"),
                            }
                        },
                        _ => panic!("Initializer should be a binary expression"),
                    }
                } else {
                    panic!("Expected an initializer for the variable");
                }
            },
            _ => panic!("Expected a declaration statement"),
        }
    }
}

#[test]
fn test_parse_functions_with_call() {
    let source = "
fn first_add() -> i64 {
    return second_add()
}
fn second_add() -> i64 {
    return 4
}
read write c = first_add()";

    let mut parser = create_parser(source);
    
    // Parse the first function (first_add)
    let result1 = parser.parse_statement();
    assert!(result1.is_ok(), "Failed to parse first function declaration");
    
    if let Ok(stmt) = result1 {
        match stmt {
            Statement::Function { name, params, body, return_type, is_behavior } => {
                // Check function name and signature
                assert_eq!(name, "first_add", "First function name should be 'first_add'");
                assert!(params.is_empty(), "First function should have no parameters");
                assert!(!is_behavior, "Should not be a behavior");
                
                // Check return type
                assert!(return_type.is_some(), "Should have a return type");
                if let Some(ret) = &return_type {
                    assert_eq!(ret.base_type, Type::I64, "Return type should be i64");
                }
                
                // Check function body (should contain a return statement)
                assert_eq!(body.len(), 1, "Function body should have one statement");
                match &body[0] {
                    Statement::Return(expr) => {
                        // Return statement should contain a function call
                        match expr {
                            Expression::Call { function, arguments } => {
                                assert_eq!(function, "second_add", "Should call second_add");
                                assert!(arguments.is_empty(), "Call should have no arguments");
                            },
                            _ => panic!("Expected function call expression in return statement"),
                        }
                    },
                    _ => panic!("Expected return statement in function body"),
                }
            },
            _ => panic!("Expected function declaration"),
        }
    }
    
    // Parse the second function (second_add)
    let result2 = parser.parse_statement();
    assert!(result2.is_ok(), "Failed to parse second function declaration");
    
    if let Ok(stmt) = result2 {
        match stmt {
            Statement::Function { name, params, body, return_type, .. } => {
                // Check function name
                assert_eq!(name, "second_add", "Second function name should be 'second_add'");
                
                // Check return value is 4
                assert_eq!(body.len(), 1, "Function body should have one statement");
                match &body[0] {
                    Statement::Return(expr) => {
                        match expr {
                            Expression::Number(val) => {
                                assert_eq!(*val, 4, "Return value should be 4");
                            },
                            _ => panic!("Expected number in return statement"),
                        }
                    },
                    _ => panic!("Expected return statement in function body"),
                }
            },
            _ => panic!("Expected function declaration"),
        }
    }
    
    // Parse the variable declaration (read write c = first_add())
    let result3 = parser.parse_statement();
    assert!(result3.is_ok(), "Failed to parse variable declaration");
    
    if let Ok(stmt) = result3 {
        match stmt {
            Statement::Declaration { name, typ, initializer } => {
                // Check variable name
                assert_eq!(name, "c", "Variable name should be 'c'");
                
                // Verify permission types
                assert_eq!(typ.permissions.len(), 2, "Should have 2 permissions");
                assert!(typ.permissions.contains(&Permission::Read), "Should have Read permission");
                assert!(typ.permissions.contains(&Permission::Write), "Should have Write permission");
                
                // Check initializer is a function call
                if let Some(expr) = initializer {
                    match expr {
                        Expression::Call { function, arguments } => {
                            assert_eq!(function, "first_add", "Should call first_add");
                            assert!(arguments.is_empty(), "Call should have no arguments");
                        },
                        _ => panic!("Expected function call in initializer"),
                    }
                } else {
                    panic!("Expected initializer for variable");
                }
            },
            _ => panic!("Expected variable declaration"),
        }
    }
}

#[test]
fn test_parse_peak_expression() {
    let source = "reads write c = 4 + 6\nread b = peak c";
    let mut parser = create_parser(source);
    
    // Parse first statement: reads write c = 4 + 6
    let result1 = parser.parse_statement();
    assert!(result1.is_ok(), "Failed to parse first variable declaration");
    
    if let Ok(stmt) = result1 {
        match stmt {
            Statement::Declaration { name, typ, initializer } => {
                // Check variable name
                assert_eq!(name, "c", "First variable name should be 'c'");
                
                // Verify permission types - should have both reads and write
                assert_eq!(typ.permissions.len(), 2, "Should have 2 permissions");
                assert!(typ.permissions.contains(&Permission::Reads), "Should have Reads permission");
                assert!(typ.permissions.contains(&Permission::Write), "Should have Write permission");
                
                // Check initializer is 4 + 6
                if let Some(expr) = initializer {
                    match expr {
                        Expression::Binary { left, operator, right } => {
                            assert_eq!(operator, TokenType::Plus, "Operator should be '+'");
                            
                            // Left operand should be 4
                            if let Expression::Number(value) = *left {
                                assert_eq!(value, 4, "Left operand should be 4");
                            } else {
                                panic!("Left operand should be a number");
                            }
                            
                            // Right operand should be 6
                            if let Expression::Number(value) = *right {
                                assert_eq!(value, 6, "Right operand should be 6");
                            } else {
                                panic!("Right operand should be a number");
                            }
                        },
                        _ => panic!("Expected binary expression as initializer"),
                    }
                } else {
                    panic!("Expected initializer for first variable");
                }
            },
            _ => panic!("Expected declaration statement for first variable"),
        }
    }
    
    // Parse second statement: read b = peak c
    let result2 = parser.parse_statement();
    assert!(result2.is_ok(), "Failed to parse second variable declaration");
    
    if let Ok(stmt) = result2 {
        match stmt {
            Statement::Declaration { name, typ, initializer } => {
                // Check variable name
                assert_eq!(name, "b", "Second variable name should be 'b'");
                
                // Verify permission type - should be read only
                assert_eq!(typ.permissions.len(), 1, "Should have 1 permission");
                assert!(typ.permissions.contains(&Permission::Read), "Should have Read permission");
                
                // Check initializer is peak c
                if let Some(expr) = initializer {
                    match expr {
                        Expression::Peak(inner) => {
                            // The inner expression should be a variable reference to c
                            match *inner {
                                Expression::Variable(ref var_name) => {
                                    assert_eq!(var_name, "c", "Peak should reference variable 'c'");
                                },
                                _ => panic!("Expected variable reference inside peak expression"),
                            }
                        },
                        _ => panic!("Expected peak expression as initializer"),
                    }
                } else {
                    panic!("Expected initializer for second variable");
                }
            },
            _ => panic!("Expected declaration statement for second variable"),
        }
    }
}

