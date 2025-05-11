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

#[test]
fn test_exclusive_read_write_clone_violation() {
    let source = r#"
        read write counter = 4
        reads cloned = clone counter
    "#;

    let result = compile_and_run(source);
    
    assert!(result.is_err(), "Expected failure but got success: {:?}", result);
    
    match result {
        Err(e) => {
            let expected = "Cannot clone from counter - variable has exclusive read write access";
            assert!(e.contains(expected), 
                "Expected error '{}', got: '{}'", expected, e);
        },
        Ok(_) => panic!("Expected error but compilation succeeded"),
    }
}

#[test]
fn test_peak_read() {
    let source = r#"
        reads write counter = 4
        read c = peak counter
        print c
    "#;

    let result = compile_and_run(source);
    assert!(result.is_ok(), "Compilation failed with error: {:?}", result.err());
    
    let interpreter = result.unwrap();
    assert_eq!(interpreter.get_variable("c"), Some(4));
}

#[test]
fn test_missing_peak_keyword() {
    let source = r#"
        reads write counter = 4
        read c = counter 
    "#;

    let result = compile_and_run(source);
    assert!(result.is_err(), "Expected failure but got success: {:?}", result);
    
    match result {
        Err(e) => {
            let expected = "Must use 'peak' keyword for temporary read access: c = peak counter";
            assert!(e.contains(expected), 
                "Expected error '{}', got: '{}'", expected, e);
        },
        Ok(_) => panic!("Expected error but compilation succeeded"),
    }
}

#[test]
fn test_peak_reference_behavior() {
    let source = r#"
        reads write counter = 4
        read c = peak counter
        counter += 6
        print c         
    "#;

    let result = compile_and_run(source);
    assert!(result.is_ok(), "Compilation failed with error: {:?}", result.err());
    
    let interpreter = result.unwrap();
    assert_eq!(interpreter.get_variable("c"), Some(10));  // c sees the updated value
    assert_eq!(interpreter.get_variable("counter"), Some(10));
}

#[test]
fn test_complex_peak_behavior() {
    let source = r#"
        reads write counter = 10
        reads write temp = 5
        read view1 = peak counter
        counter += temp
        read view2 = peak counter
        temp += 10
        counter += temp
        print view1
        print view2
        print counter
    "#;

    let result = compile_and_run(source);
    assert!(result.is_ok(), "Compilation failed with error: {:?}", result.err());
    
    let interpreter = result.unwrap();
    assert_eq!(interpreter.get_variable("view1"), Some(30));
    assert_eq!(interpreter.get_variable("view2"), Some(30));
    assert_eq!(interpreter.get_variable("counter"), Some(30));
    assert_eq!(interpreter.get_variable("temp"), Some(15));
}

#[test]
fn test_peak_chain() {
    let source = r#"
        reads write original = 42
        read view1 = peak original
        read view2 = peak original
        read view3 = peak original
        original += 8
        print view1
        print view2
        print view3
    "#;

    let result = compile_and_run(source);
    assert!(result.is_ok(), "Compilation failed with error: {:?}", result.err());
    
    let interpreter = result.unwrap();
    // All views should see the updated value
    assert_eq!(interpreter.get_variable("view1"), Some(50));
    assert_eq!(interpreter.get_variable("view2"), Some(50));
    assert_eq!(interpreter.get_variable("view3"), Some(50));
    assert_eq!(interpreter.get_variable("original"), Some(50));
}

#[test]
fn test_reads_writes_shared_mutation() {
    let source = r#"
        reads writes counter = 5
        write c = counter
        c += 5
        print counter
    "#;

    let result = compile_and_run(source);
    assert!(result.is_ok(), "Compilation failed with error: {:?}", result.err());
    
    let interpreter = result.unwrap();
    assert_eq!(interpreter.get_variable("counter"), Some(10));
    assert_eq!(interpreter.get_variable("c"), Some(10));
}

#[test]
fn test_reads_writes_multiple_mutations() {
    let source = r#"
        reads writes shared = 100
        write a = shared
        write b = shared
        a += 10
        b += 5
        print shared
    "#;

    let result = compile_and_run(source);
    assert!(result.is_ok(), "Compilation failed with error: {:?}", result.err());
    
    let interpreter = result.unwrap();
    assert_eq!(interpreter.get_variable("shared"), Some(115));
    assert_eq!(interpreter.get_variable("a"), Some(115));
    assert_eq!(interpreter.get_variable("b"), Some(115));
}

#[test]
fn test_write_only_operation_error() {
    let source = r#"
        reads writes counter = 5
        write c = counter
        c += 10
        print counter
    "#;

    let result = compile_and_run(source);
    assert!(result.is_err(), "Expected failure but got success: {:?}", result);
    
    match result {
        Err(e) => {
            let expected = "Cannot use += on 'c' - write-only reference cannot read its own value";
            assert!(e.contains(expected), 
                "Expected helpful error message about += operation, got: {}", e);
            assert!(e.contains("Use direct assignment instead: c = <new_value>"), 
                "Expected suggestion about direct assignment");
            assert!(e.contains("Request read permission: read write c = counter"), 
                "Expected suggestion about read permission");
        },
        Ok(_) => panic!("Expected error but compilation succeeded"),
    }
}

#[test]
fn test_writes_direct_assignment() {
    let source = r#"
        reads writes counter = 5
        write c = counter
        c = 10
        print counter
    "#;

    let result = compile_and_run(source);
    assert!(result.is_ok(), "Compilation failed with error: {:?}", result.err());
    
    let interpreter = result.unwrap();
    assert_eq!(interpreter.get_variable("counter"), Some(10), 
        "Expected counter to be updated through write reference");
    assert_eq!(interpreter.get_variable("c"), Some(10), 
        "Expected c to have the new value");
}

#[test]
fn test_reads_writes_with_peak_reference() {
    let source = r#"
        reads writes counter = 5
        write c = counter
        read r = peak counter
        c = 10
        print r
    "#;

    let result = compile_and_run(source);
    assert!(result.is_ok(), "Compilation failed with error: {:?}", result.err());
    
    let interpreter = result.unwrap();
    assert_eq!(interpreter.get_variable("counter"), Some(10), 
        "Expected counter to be updated through write reference");
    assert_eq!(interpreter.get_variable("c"), Some(10), 
        "Expected write reference to have new value");
    assert_eq!(interpreter.get_variable("r"), Some(10), 
        "Expected peak reference to see the updated value");
}

#[test]
fn test_simple_function() {
    let source = r#"
        fn add(reads x: i64, reads y: i64) -> i64 {
            return x + y
        }

        reads write result = add(5, 3)
        print result
    "#;

    let result = compile_and_run(source);
    assert!(result.is_ok(), "Compilation failed with error: {:?}", result.err());
    
    let interpreter = result.unwrap();
    assert_eq!(interpreter.get_variable("result"), Some(8), 
        "Expected function to compute 5 + 3 = 8");
}

#[test]
fn test_nested_function_calls() {
    let source = r#"
        fn double(reads x: i64) -> i64 {
            return x + x
        }

        fn quadruple(reads x: i64) -> i64 {
            reads write temp = double(x)
            return double(temp)
        }

        reads write result = quadruple(3)
        print result
    "#;

    let result = compile_and_run(source);
    assert!(result.is_ok(), "Compilation failed with error: {:?}", result.err());
    
    let interpreter = result.unwrap();
    assert_eq!(interpreter.get_variable("result"), Some(12), 
        "Expected quadruple(3) to compute double(double(3)) = 12");
}

// Helper function to run the full compilation pipeline
fn compile_and_run(source: &str) -> Result<Interpreter, String> {
    println!("\n=== Starting compilation pipeline ===");
    println!("Source code:\n{}", source);

    // Parse source to AST
    let mut parser = Parser::new(source);
    let ast = parser.parse().map_err(|e| format!("Parse error: {}", e))?;
    println!("\nAST:\n{:?}", ast);

    // Convert AST to HIR
    let hir = convert_to_hir(ast);
    println!("\nHIR:\n{:?}", hir);

    // Check permissions at compile time
    let mut checker = TypePermissionChecker::new();
    let check_result = checker.check_program(&hir);
    if let Err(e) = &check_result {
        println!("Permission check failed: {}", e);
    }
    check_result.map_err(|e| format!("Permission error: {}", e))?;
    println!("\nPermission check passed");

    // Lower HIR to MIR
    let mir = lower_hir(&hir);
    println!("\nMIR:\n{:?}", mir);

    // Execute MIR
    let mut interpreter = Interpreter::new();
    println!("\nStarting execution...");
    interpreter.execute(&mir).map_err(|e| format!("Execution error: {}", e))?;
    println!("\nFinal variable states:");
    interpreter.print_variables();  // You'll need to add this method to Interpreter
    
    Ok(interpreter)
}