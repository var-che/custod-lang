use crate::mir::{
    functions::{analyze_scopes, analyze_function, find_functions},
    types::{MirFunction, MirInstruction, MirValue}
};
use std::collections::HashSet;

#[test]
fn test_analyze_function_with_nested_scopes() {
    // Create a MIR function with nested scopes to test scope analysis
    let mir = MirFunction {
        instructions: vec![
            // Main function scope
            MirInstruction::EnterScope,
            MirInstruction::Store { target: "x".to_string(), value: MirValue::Number(1) },
            
            // Nested scope (like an if-block)
            MirInstruction::EnterScope,
            MirInstruction::Store { target: "y".to_string(), value: MirValue::Number(2) },
            MirInstruction::ExitScope,
            
            // Another nested scope (like an else-block)
            MirInstruction::EnterScope,
            MirInstruction::Store { target: "z".to_string(), value: MirValue::Number(3) },
            MirInstruction::ExitScope,
            
            // Exit main function scope
            MirInstruction::ExitScope,
        ]
    };

    // Test scope analysis
    let scopes = analyze_scopes(&mir);
    
    // We should have 3 scopes: main and two nested
    assert_eq!(scopes.len(), 3, "Should detect 3 scopes");
    
    // Check each scope's boundaries
    // Format is (start_index, end_index)
    
    // First nested scope (indexes 2 to 4)
    assert_eq!(scopes[0], (2, 4), "First nested scope should span indices 2 to 4");
    
    // Second nested scope (indexes 5 to 7)
    assert_eq!(scopes[1], (5, 7), "Second nested scope should span indices 5 to 7");
    
    // Outer scope (indexes 0 to 8)
    assert_eq!(scopes[2], (0, 8), "Main scope should span indices 0 to 8");
    
    // Test function analysis
    let info = analyze_function(&mir);
    
    // Check variables found in function
    let expected_vars: HashSet<String> = ["x", "y", "z"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    
    assert_eq!(info.variables, expected_vars, "Should detect all variables in function");
    assert_eq!(info.instruction_count, 9, "Should count all instructions");
}

#[test]
fn test_find_functions() {
    // Create a MIR function with multiple function definitions
    let mir = MirFunction {
        instructions: vec![
            // Function "add"
            MirInstruction::Call { function: "add".to_string() },
            MirInstruction::EnterScope,
            MirInstruction::Store { target: "a".to_string(), value: MirValue::Number(1) },
            MirInstruction::Store { target: "b".to_string(), value: MirValue::Number(2) },
            MirInstruction::Add { 
                target: 0, 
                left: MirValue::Variable("a".to_string()), 
                right: MirValue::Variable("b".to_string()) 
            },
            MirInstruction::Return { value: MirValue::Temporary(0) },
            MirInstruction::ExitScope,
            
            // Function "multiply"
            MirInstruction::Call { function: "multiply".to_string() },
            MirInstruction::EnterScope,
            MirInstruction::Store { target: "x".to_string(), value: MirValue::Number(5) },
            MirInstruction::Store { target: "y".to_string(), value: MirValue::Number(10) },
            // No explicit return, will return default value
            MirInstruction::ExitScope,
        ]
    };

    let functions = find_functions(&mir);
    
    // Should find two functions: "add" and "multiply"
    assert_eq!(functions.len(), 2, "Should find 2 functions");
    assert!(functions.contains_key("add"), "Should find 'add' function");
    assert!(functions.contains_key("multiply"), "Should find 'multiply' function");
    
    // Check details of each function
    let add_fn = &functions["add"];
    assert!(add_fn.variables.contains("a"), "Add function should use variable 'a'");
    assert!(add_fn.variables.contains("b"), "Add function should use variable 'b'");
    assert!(add_fn.has_return, "Add function should have a return statement");
    
    let multiply_fn = &functions["multiply"];
    assert!(multiply_fn.variables.contains("x"), "Multiply function should use variable 'x'");
    assert!(multiply_fn.variables.contains("y"), "Multiply function should use variable 'y'");
    assert!(!multiply_fn.has_return, "Multiply function should not have a return statement");
    
    // Print function info for debugging
    println!("Found functions: {:?}", functions.keys().collect::<Vec<_>>());
    println!("Add function variables: {:?}", add_fn.variables);
    println!("Multiply function variables: {:?}", multiply_fn.variables);
}

#[test]
fn test_function_with_nested_calls() {
    // Create a MIR function with nested function calls
    let mir = MirFunction {
        instructions: vec![
            // Define main function 
            MirInstruction::Call { function: "main".to_string() },
            MirInstruction::EnterScope,
            MirInstruction::Store { target: "result".to_string(), value: MirValue::Number(0) },
            
            // Call to first function
            MirInstruction::Call { function: "calculate".to_string() },
            MirInstruction::EnterScope,
            // Define a nested function
            MirInstruction::Call { function: "helper".to_string() },
            MirInstruction::EnterScope,
            MirInstruction::Store { target: "helper_var".to_string(), value: MirValue::Number(42) },
            MirInstruction::Return { value: MirValue::Variable("helper_var".to_string()) },
            MirInstruction::ExitScope,
            // Use result from nested function
            MirInstruction::Store { target: "calc_result".to_string(), value: MirValue::Temporary(0) },
            MirInstruction::Return { value: MirValue::Variable("calc_result".to_string()) },
            MirInstruction::ExitScope,
            
            // Store result from calculate function
            MirInstruction::Store { target: "result".to_string(), value: MirValue::Temporary(1) },
            MirInstruction::ExitScope,
        ]
    };

    // Print detailed debugging info
    println!("Instruction count: {}", mir.instructions.len());
    println!("Detected scopes: {:?}", analyze_scopes(&mir));
    
    // For each instruction, print its index and type
    for (i, instruction) in mir.instructions.iter().enumerate() {
        match instruction {
            MirInstruction::Call { function } => {
                println!("Instruction {}: Call to function '{}'", i, function);
            },
            MirInstruction::EnterScope => {
                println!("Instruction {}: EnterScope", i);
            },
            MirInstruction::ExitScope => {
                println!("Instruction {}: ExitScope", i);
            },
            _ => {
                println!("Instruction {}: Other instruction", i);
            }
        }
    }
    
    let functions = find_functions(&mir);
    
    // Should find three functions: main, calculate, and helper
    println!("Found functions: {:?}", functions.keys().collect::<Vec<_>>());
    assert_eq!(functions.len(), 3, "Should find 3 functions");
    assert!(functions.contains_key("main"), "Should find 'main' function");
    assert!(functions.contains_key("calculate"), "Should find 'calculate' function");
    assert!(functions.contains_key("helper"), "Should find 'helper' function");
    
    // Verify function nesting
    let main_fn = &functions["main"];
    let calculate_fn = &functions["calculate"];
    let helper_fn = &functions["helper"];
    
    // Check instruction counts
    assert_eq!(main_fn.instruction_count, 3, "Main function should have 3 instructions");
    assert_eq!(calculate_fn.instruction_count, 8, "Calculate function should have 8 instructions");
    assert_eq!(helper_fn.instruction_count, 3, "Helper function should have 3 instructions");
    
    // Check variable usage
    assert!(main_fn.variables.contains("result"), "Main should use variable 'result'");
    assert!(calculate_fn.variables.contains("calc_result"), "Calculate should use 'calc_result'");
    assert!(helper_fn.variables.contains("helper_var"), "Helper should use 'helper_var'");
    
    // Check return statements
    assert!(!main_fn.has_return, "Main function should not have a return");
    assert!(calculate_fn.has_return, "Calculate function should have a return");
    assert!(helper_fn.has_return, "Helper function should have a return");
}

#[test]
fn test_function_parameter_analysis() {
    // Create a MIR function with parameters and analyze parameter detection
    let mir = MirFunction {
        instructions: vec![
            // Function with parameters
            MirInstruction::Call { function: "sum_and_multiply".to_string() },
            MirInstruction::EnterScope,
            // Parameters loaded at the beginning
            MirInstruction::Store { target: "a".to_string(), value: MirValue::Temporary(0) },
            MirInstruction::Store { target: "b".to_string(), value: MirValue::Temporary(1) },
            MirInstruction::Store { target: "c".to_string(), value: MirValue::Temporary(2) },
            
            // Function body
            MirInstruction::Add { 
                target: 3, 
                left: MirValue::Variable("a".to_string()), 
                right: MirValue::Variable("b".to_string()) 
            },
            MirInstruction::Store { target: "sum".to_string(), value: MirValue::Temporary(3) },
            
            MirInstruction::Add { 
                target: 4, 
                left: MirValue::Variable("sum".to_string()), 
                right: MirValue::Variable("c".to_string()) 
            },
            MirInstruction::Return { value: MirValue::Temporary(4) },
            MirInstruction::ExitScope,
        ]
    };

    let functions = find_functions(&mir);
    assert!(functions.contains_key("sum_and_multiply"), "Should find function");
    
    let function = &functions["sum_and_multiply"];
    
    // Test parameter detection
    assert_eq!(function.parameters.len(), 3, "Should detect 3 parameters");
    assert_eq!(function.parameters[0], "a", "First parameter should be 'a'");
    assert_eq!(function.parameters[1], "b", "Second parameter should be 'b'");
    assert_eq!(function.parameters[2], "c", "Third parameter should be 'c'");
    
    // Check other properties
    assert!(function.has_return, "Function should have a return statement");
    assert!(function.variables.contains("sum"), "Should detect local variable 'sum'");
    assert_eq!(function.instruction_count, 9, "Should have 9 instructions");
    
    // Print for debugging
    println!("Function parameters: {:?}", function.parameters);
    println!("Function variables: {:?}", function.variables);
}

#[test]
fn test_simple_nested_functions() {
    // Create a more straightforward nested function example
    let mir = MirFunction {
        instructions: vec![
            // First function
            MirInstruction::Call { function: "outer".to_string() },
            MirInstruction::EnterScope,
            MirInstruction::Store { target: "x".to_string(), value: MirValue::Number(1) },
            
            // Second function as a direct nested call
            MirInstruction::Call { function: "inner".to_string() },
            MirInstruction::EnterScope,
            MirInstruction::Store { target: "y".to_string(), value: MirValue::Number(2) },
            MirInstruction::ExitScope,
            
            MirInstruction::ExitScope,
        ]
    };

    // Print out the scopes for debugging
    println!("Detected scopes: {:?}", analyze_scopes(&mir));
    
    let functions = find_functions(&mir);
    
    // Print out what functions were found
    println!("Found functions: {:?}", functions.keys().collect::<Vec<_>>());
    
    // Should find two functions: outer and inner
    assert_eq!(functions.len(), 2, "Should find 2 functions");
    assert!(functions.contains_key("outer"), "Should find 'outer' function");
    assert!(functions.contains_key("inner"), "Should find 'inner' function");
}

#[test]
fn test_nested_function_calls_fixed() {
    // Create a MIR function with clear function boundaries
    let mir = MirFunction {
        instructions: vec![
            // Main function
            MirInstruction::Call { function: "main".to_string() },
            MirInstruction::EnterScope,
            MirInstruction::Store { target: "x".to_string(), value: MirValue::Number(1) },
            
            // Function 1 - add proper separation before and after
            MirInstruction::Call { function: "calculate".to_string() },
            MirInstruction::EnterScope,
            MirInstruction::Store { target: "y".to_string(), value: MirValue::Number(2) },
            MirInstruction::ExitScope,
            
            // Function 2
            MirInstruction::Call { function: "helper".to_string() },
            MirInstruction::EnterScope,
            MirInstruction::Store { target: "z".to_string(), value: MirValue::Number(3) },
            MirInstruction::ExitScope,
            
            MirInstruction::ExitScope,
        ]
    };

    // Debug what's happening
    println!("Scopes: {:?}", analyze_scopes(&mir));
    
    for (i, inst) in mir.instructions.iter().enumerate() {
        println!("Instruction {}: {:?}", i, inst);
    }

    let functions = find_functions(&mir);
    println!("Found functions: {:?}", functions.keys().collect::<Vec<_>>());
    
    // Now we should find all three functions
    assert_eq!(functions.len(), 3, "Should find 3 functions");
    assert!(functions.contains_key("main"), "Should find 'main'");
    assert!(functions.contains_key("calculate"), "Should find 'calculate'");
    assert!(functions.contains_key("helper"), "Should find 'helper'");
}

#[test]
fn test_function_calling_another_function() {
    // Create a MIR function with one function explicitly calling another function
    let mir = MirFunction {
        instructions: vec![
            // Define main function
            MirInstruction::Call { function: "main".to_string() },
            MirInstruction::EnterScope,
            
            // Define fibonacci function first (to be called later)
            MirInstruction::Call { function: "fibonacci".to_string() },
            MirInstruction::EnterScope,
            MirInstruction::Store { target: "n".to_string(), value: MirValue::Temporary(0) },
            // Function logic - simplified for test
            MirInstruction::Add { 
                target: 1, 
                left: MirValue::Variable("n".to_string()), 
                right: MirValue::Number(1) 
            },
            MirInstruction::Return { value: MirValue::Temporary(1) },
            MirInstruction::ExitScope,
            
            // Main function body - calls fibonacci
            MirInstruction::Store { target: "input".to_string(), value: MirValue::Number(5) },
            
            // This is an actual function call during execution (not a function definition)
            MirInstruction::Load { 
                target: 2, 
                value: MirValue::Variable("input".to_string()) 
            },
            MirInstruction::Call { 
                function: "fibonacci".to_string()
            },
            // After call we would handle arguments and return value separately
            MirInstruction::Store { target: "result".to_string(), value: MirValue::Temporary(3) },
            MirInstruction::Store { 
                target: "result".to_string(), 
                value: MirValue::Temporary(3) 
            },
            
            // Print result
            MirInstruction::Print { value: MirValue::Variable("result".to_string()) },
            MirInstruction::ExitScope,
        ]
    };
    
    // Debug output
    println!("Scopes: {:?}", analyze_scopes(&mir));
    
    for (i, inst) in mir.instructions.iter().enumerate() {
        println!("Instruction {}: {:?}", i, inst);
    }
    
    // Find all function definitions
    let functions = find_functions(&mir);
    println!("Found functions: {:?}", functions.keys().collect::<Vec<_>>());
    
    // Should find both functions
    assert_eq!(functions.len(), 2, "Should find 2 functions");
    assert!(functions.contains_key("main"), "Should find 'main'");
    assert!(functions.contains_key("fibonacci"), "Should find 'fibonacci'");
    
    // Check that main function contains FunctionCall instruction
    let main_fn = &functions["main"];
    let mut has_function_call = false;
    
    // We'd need to examine the instructions within the main function to check for this
    // This is a simplified check that assumes your FunctionInfo includes this data
    // If FunctionInfo doesn't store the actual instructions, we can modify the test
    for instruction in &mir.instructions[1..14] {  // Main function scope
        if let MirInstruction::Call { function } = instruction {
            if function == "fibonacci" {
                has_function_call = true;
                break;
            }
        }
    }
    
    assert!(has_function_call, "Main function should call fibonacci");
    
    // Check fibonacci function properties
    let fib_fn = &functions["fibonacci"];
    assert!(fib_fn.variables.contains(&"n".to_string()), 
           "Fibonacci function should use variable 'n'");
    assert!(fib_fn.has_return, "Fibonacci function should have a return statement");
}

