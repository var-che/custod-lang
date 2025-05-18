use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::ast::{Statement, Expression};
use crate::types::{Type, Permission};

/// Test the complete front-end compilation pipeline
#[test]
fn test_front_end_pipeline() {
    // Sample program with our new Swift-style type syntax
    let source = r#"
        reads write counter: Int = 10
        
        fn increment(reads amount: Int) -> Int {
            counter = counter + amount
            return counter
        }
        
        reads result = increment(5)
    "#;
    
    println!("\n=== FRONT-END PIPELINE TEST ===");
    println!("Source code:\n{}", source);

    // Step 1: Lexical analysis
    println!("\n--- Lexical Analysis ---");
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    println!("Generated {} tokens", tokens.len());
    
    // Print first few tokens for inspection
    println!("First 10 tokens:");
    for (i, token) in tokens.iter().enumerate().take(10) {
        println!("  {}: {:?} ({})", i, token.token_type, token.lexeme);
    }

    // Step 2: Parsing
    println!("\n--- Syntax Parsing ---");
    let mut parser = Parser::from_source(source);
    let statements = parser.parse_statements();
    
    println!("Parsed {} statements", statements.len());
    
    // Validate that we have the expected number of statements
    assert_eq!(statements.len(), 3, "Should have parsed 3 statements");
    
    // Validate the first statement (variable declaration)
    println!("\n--- Validating Variable Declaration ---");
    match &statements[0] {
        Statement::Declaration { name, typ, initializer } => {
            println!("✓ First statement is a variable declaration");
            println!("  Name: {}", name);
            println!("  Type: {:?}", typ);
            println!("  Initializer: {:?}", initializer);
            
            // Verify variable details
            assert_eq!(name, "counter", "Variable should be named 'counter'");
            assert_eq!(typ.base_type, Type::Int, "Variable should be of type Int");
            assert_eq!(typ.permissions.len(), 2, "Variable should have 2 permissions");
            assert!(typ.permissions.contains(&Permission::Reads), "Should have Reads permission");
            assert!(typ.permissions.contains(&Permission::Write), "Should have Write permission");
            
            // Verify initializer
            match initializer {
                Some(Expression::Number(n)) => {
                    assert_eq!(*n, 10, "Initializer should be 10");
                    println!("  ✓ Variable initializer correct");
                },
                _ => panic!("Expected number initializer"),
            }
            
            println!("  ✓ Variable declaration validation successful");
        },
        _ => panic!("Expected variable declaration as first statement"),
    }
    
    // Validate the second statement (function declaration)
    println!("\n--- Validating Function Declaration ---");
    match &statements[1] {
        Statement::Function { name, params, body, return_type, .. } => {
            println!("✓ Second statement is a function declaration");
            println!("  Name: {}", name);
            println!("  Parameters: {} parameter(s)", params.len());
            println!("  Body: {} statement(s)", body.len());
            println!("  Return type: {:?}", return_type);
            
            // Verify function name
            assert_eq!(name, "increment", "Function should be named 'increment'");
            
            // Verify parameter
            assert_eq!(params.len(), 1, "Function should have 1 parameter");
            assert_eq!(params[0].0, "amount", "Parameter should be named 'amount'");
            assert_eq!(params[0].1.base_type, Type::Int, "Parameter should be of type Int");
            assert!(params[0].1.permissions.contains(&Permission::Reads), 
                  "Parameter should have Reads permission");
            
            // Verify return type
            assert!(return_type.is_some(), "Function should have a return type");
            if let Some(rt) = return_type {
                assert_eq!(rt.base_type, Type::Int, "Return type should be Int");
            }
            
            // Verify function body
            assert_eq!(body.len(), 2, "Function body should have 2 statements");
            
            println!("  ✓ Function declaration validation successful");
        },
        _ => panic!("Expected function declaration as second statement"),
    }
    
    // Validate the third statement (function call)
    println!("\n--- Validating Function Call ---");
    match &statements[2] {
        Statement::Declaration { name, initializer, .. } => {
            println!("✓ Third statement is a variable declaration");
            assert_eq!(name, "result", "Variable should be named 'result'");
            
            // Verify it's a function call
            match initializer {
                Some(Expression::Call { function, arguments }) => {
                    println!("  ✓ Initializer is a function call");
                    
                    // Verify function name (function is a String in your AST)
                    assert_eq!(function, "increment", "Should call 'increment' function");
                    println!("  ✓ Calling correct function");
                    
                    // Verify argument
                    assert_eq!(arguments.len(), 1, "Function call should have 1 argument");
                    match &arguments[0] {
                        Expression::Number(n) => {
                            assert_eq!(*n, 5, "Argument should be 5");
                            println!("  ✓ Function argument correct");
                        },
                        _ => panic!("Expected number as function argument"),
                    }
                },
                _ => panic!("Expected function call as initializer"),
            }
            
            println!("  ✓ Function call validation successful");
        },
        _ => panic!("Expected variable declaration as third statement"),
    }
    
    println!("\n=== FRONT-END PIPELINE TEST COMPLETED SUCCESSFULLY ===");
}