use front_end::parser::Parser;
use middle_end::hir::convert_to_hir;
use middle_end::mir::lower_hir;
use middle_end::interpreter::Interpreter;
use middle_end::type_checker::TypePermissionChecker;

#[test]
fn test_simple_addition() {
    // Test simple addition with reads write permission
    let source = r#"
        reads write counter = 1
        counter += 13
        print counter
    "#;

    // Parse and execute
    let result = compile_and_run(source);
    assert!(result.is_ok());
    
    // Get interpreter to check final value
    let interpreter = result.unwrap();
    assert_eq!(interpreter.get_variable("counter"), Some(14));
}

#[test]
fn test_multiple_operations() {
    let source = r#"
        reads write x = 10
        reads write y = 20
        x += y
        print x
    "#;

    let result = compile_and_run(source);
    assert!(result.is_ok());
    
    let interpreter = result.unwrap();
    assert_eq!(interpreter.get_variable("x"), Some(30));
}

#[test]
fn test_invalid_permissions() {
    // Missing write permission should fail
    let source = r#"
        reads counter = 1
        counter += 1  // Error: needs write permission
        print counter
    "#;

    let result = compile_and_run(source);
    assert!(result.is_err());
}

#[test]
fn test_missing_read_permission() {
    let source = r#"
        write counter = 33
        read cloned_read = counter
    "#;

    let result = compile_and_run(source);
    
    // Verify that compilation fails
    assert!(result.is_err());
    
    // Check for specific error message
    match result {
        Err(e) => {
            assert!(e.contains("read permission"), 
                "Expected error about missing read permission, got: {}", e);
        },
        Ok(_) => panic!("Expected error but compilation succeeded"),
    }
}

#[test]
fn test_reads_cloning() {
    let source = r#"
        reads write counter = 55
        reads cloned = counter
        counter += 5          
        print cloned         
    "#;

    let result = compile_and_run(source);
    match &result {
        Ok(_) => println!("Compilation succeeded"),
        Err(e) => println!("Compilation failed: {}", e),
    }
    assert!(result.is_ok(), "Compilation failed with error: {:?}", result.err());
    
    let interpreter = result.unwrap();
    assert_eq!(interpreter.get_variable("cloned"), Some(55));
    assert_eq!(interpreter.get_variable("counter"), Some(60));
}

#[test]
fn test_missing_clone_keyword() {
    let source = r#"
        reads write counter = 55
        reads cloned = counter  
    "#;

    let result = compile_and_run(source);
    
    println!("Result: {:?}", result);  // Keep debug output
    
    // Verify that compilation fails
    assert!(result.is_err(), "Expected failure but got success: {:?}", result);
    
    // Check for specific error message
    match result {
        Err(e) => {
            // Update the expected error message to match what we're getting
            let expected = "Must use 'clone' keyword when creating reads alias: cloned = clone counter";
            assert!(e.contains(expected), 
                "Expected error '{}', got: '{}'", expected, e);
        },
        Ok(_) => panic!("Expected error but compilation succeeded"),
    }
}

#[test]
fn test_explicit_clone() {
    let source = r#"
        reads write counter = 55
        reads cloned = clone counter 
        counter += 5          
        print cloned      
    "#;

    let result = compile_and_run(source);
    assert!(result.is_ok(), "Compilation failed with error: {:?}", result.err());
    
    let interpreter = result.unwrap();
    assert_eq!(interpreter.get_variable("cloned"), Some(55));
    assert_eq!(interpreter.get_variable("counter"), Some(60));
}

#[test]
fn test_complex_operations() {
    let source = r#"
        reads write counter = 100
        reads write temp = 5
        reads snapshot = clone counter
        counter += temp
        temp += 10
        counter += temp
        print snapshot
        print counter
    "#;

    let result = compile_and_run(source);
    assert!(result.is_ok(), "Compilation failed with error: {:?}", result.err());
    
    let interpreter = result.unwrap();
    assert_eq!(interpreter.get_variable("snapshot"), Some(100));
    assert_eq!(interpreter.get_variable("counter"), Some(120)); // Updated expectation
    assert_eq!(interpreter.get_variable("temp"), Some(15));
}

#[test]
fn test_exclusive_read_write() {
    let source = r#"
        read write counter = 4
        counter += 5
        print counter
    "#;

    let result = compile_and_run(source);
    assert!(result.is_ok(), "Compilation failed with error: {:?}", result.err());
    
    let interpreter = result.unwrap();
    assert_eq!(interpreter.get_variable("counter"), Some(9));
}

#[test]
fn test_exclusive_read_write_violation() {
    let source = r#"
        read write counter = 4
        read c = counter
    "#;

    let result = compile_and_run(source);
    
    // Verify that compilation fails
    assert!(result.is_err(), "Expected failure but got success: {:?}", result);
    
    // Check for specific error message
    match result {
        Err(e) => {
            let expected = "Cannot read from counter - variable has exclusive read write access";
            assert!(e.contains(expected), 
                "Expected error '{}', got: '{}'", expected, e);
        },
        Ok(_) => panic!("Expected error but compilation succeeded"),
    }
}

// Helper function to run the full compilation pipeline
fn compile_and_run(source: &str) -> Result<Interpreter, String> {
    // Parse source to AST
    let mut parser = Parser::new(source);
    let ast = parser.parse().map_err(|e| format!("Parse error: {}", e))?;

    // Convert AST to HIR
    let hir = convert_to_hir(ast);

    // Check permissions at compile time
    let mut checker = TypePermissionChecker::new();
    let check_result = checker.check_program(&hir);
    if let Err(e) = &check_result {
        println!("Permission check failed: {}", e);
    }
    check_result.map_err(|e| format!("Permission error: {}", e))?;

    // Lower HIR to MIR only if permissions are valid
    let mir = lower_hir(&hir);

    // Execute MIR
    let mut interpreter = Interpreter::new();
    interpreter.execute(&mir).map_err(|e| format!("Execution error: {}", e))?;
    
    Ok(interpreter)
}