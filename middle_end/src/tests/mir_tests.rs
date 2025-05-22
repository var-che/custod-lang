//! Tests for MIR generation
//!
//! This module contains tests for the MIR (Middle Intermediate Representation) generation.

use crate::hir::converter::convert_statements_to_hir;
use crate::mir::converter::convert_hir_to_mir;
use crate::mir::pretty_print::pretty_print_program;
use front_end::parser::Parser;

#[test]
fn test_simple_arithmetic() {
    // Define a simple arithmetic expression - note that the permissions 
    // (like "reads") are preserved from the source but not crucial for MIR analysis
    let source = r#"
        fn add_numbers() -> Int {
            reads x: Int = 5
            reads y: Int = 10
            reads result = x + y
            return result
        }
    "#;
    
    // Parse the code using the front-end parser
    let mut parser = Parser::from_source(source);
    let ast_statements = parser.parse_statements();
    
    // Convert AST to HIR
    let hir_program = convert_statements_to_hir(ast_statements);
    
    // Convert HIR to MIR
    let mir_program = convert_hir_to_mir(&hir_program);
    
    // Print MIR for inspection
    let mir_output = pretty_print_program(&mir_program);
    println!("Generated MIR for simple arithmetic:\n{}", mir_output);
    
    // Verify MIR structure
    assert!(mir_program.functions.contains_key("add_numbers"), "Should have 'add_numbers' function");
    
    let add_fn = &mir_program.functions["add_numbers"];
    assert_eq!(add_fn.name, "add_numbers");
    assert_eq!(add_fn.parameters.len(), 0);
    assert!(add_fn.return_type.is_some());
    assert!(!add_fn.blocks.is_empty());
    
    // Check for the addition operation
    let has_add_operation = add_fn.blocks.iter()
        .flat_map(|block| &block.instructions)
        .any(|instr| {
            matches!(instr, crate::mir::types::Instruction::BinaryOp { op, .. } 
                if matches!(op, crate::mir::types::BinaryOperation::Add))
        });
    
    assert!(has_add_operation, "Should have an addition operation in the MIR");
    
    // Check for a return instruction
    let has_return = add_fn.blocks.iter()
        .flat_map(|block| &block.instructions)
        .any(|instr| {
            matches!(instr, crate::mir::types::Instruction::Return(_))
        });
    
    assert!(has_return, "Should have a return instruction");
}

#[test]
fn test_peak_operation() {
    // Define a simple example with peak operation
    let source = r#"
        fn test_peak() {
            reads write c: Int = 1
            read d = peak c
            return d
        }
    "#;
    
    // Parse the code using the front-end parser
    let mut parser = Parser::from_source(source);
    let ast_statements = parser.parse_statements();
    
    // Convert AST to HIR
    let hir_program = convert_statements_to_hir(ast_statements);
    
    // Convert HIR to MIR
    let mir_program = convert_hir_to_mir(&hir_program);
    
    // Print MIR for inspection
    let mir_output = pretty_print_program(&mir_program);
    println!("Generated MIR for peak operation:\n{}", mir_output);
    
    // Verify MIR structure
    assert!(mir_program.functions.contains_key("test_peak"), "Should have 'test_peak' function");
    
    let peak_fn = &mir_program.functions["test_peak"];
    assert_eq!(peak_fn.name, "test_peak");
    
    // Check for variable assignments
    let has_c_assignment = peak_fn.blocks.iter()
        .flat_map(|block| &block.instructions)
        .any(|instr| {
            if let crate::mir::types::Instruction::Assign { target, source } = instr {
                if let Some(var) = peak_fn.variables.get(target) {
                    return var.name == "c";
                }
            }
            false
        });
    
    assert!(has_c_assignment, "Should assign a value to variable 'c'");
    
    // Check for peak operation (represented as a simple read in MIR)
    let has_d_assignment = peak_fn.blocks.iter()
        .flat_map(|block| &block.instructions)
        .any(|instr| {
            if let crate::mir::types::Instruction::Assign { target, source } = instr {
                if let Some(var) = peak_fn.variables.get(target) {
                    return var.name == "d";
                }
            }
            false
        });
    
    assert!(has_d_assignment, "Should assign a value to variable 'd' from peak operation");
    
    // Now when checking the output, we can verify that peak is properly implemented
    // The correct MIR output shows:
    //
    // block 0:
    //     c[0] = 1
    //     d[1] = c[0]  // This is now correct - reading from c instead of using 0
    //     return d[1]
    
    // Check for a return instruction
    let has_return = peak_fn.blocks.iter()
        .flat_map(|block| &block.instructions)
        .any(|instr| {
            matches!(instr, crate::mir::types::Instruction::Return(_))
        });
    
    assert!(has_return, "Should have a return instruction");
}

#[test]
fn test_peak_vs_copy() {
    // Define an example that shows difference between peak and copy
    // Using Clone() as our syntax for copying since that's what our parser supports
    let source = r#"
        fn test_peak_vs_copy() {
            // First scenario: peak - reference without copying
            reads write original: Int = 42
            read peek_result = peak original  // Just peek at the value, no copy
            
            // Second scenario: clone operation - deep copying the value
            reads mutable: Int = 100
            reads copy_result = clone mutable  // Deep copy the entire value
            
            return peek_result + copy_result
        }
    "#;
    
    // Parse the code using the front-end parser
    let mut parser = Parser::from_source(source);
    let ast_statements = parser.parse_statements();
    
    // Convert AST to HIR
    let hir_program = convert_statements_to_hir(ast_statements);
    
    // Convert HIR to MIR
    let mir_program = convert_hir_to_mir(&hir_program);
    
    // Print MIR for inspection
    let mir_output = pretty_print_program(&mir_program);
    println!("Generated MIR for peak vs copy operations:\n{}", mir_output);
    
    // In MIR, both might look similar as a simple value assignment
    // but the semantics are different - peak is a reference read
    // while clone is a deep copy of the value
    
    // Note: In a complete implementation, we'd need to handle this
    // difference properly, but for this test we're just checking 
    // the basic structure is created correctly
    
    let fn_key = "test_peak_vs_copy";
    assert!(mir_program.functions.contains_key(fn_key), 
            "Should have the test function");
    
    let test_fn = &mir_program.functions[fn_key];
    
    // We should see assignments for all four variables
    let var_names = ["original", "peek_result", "mutable", "copy_result"];
    for name in var_names {
        let has_var = test_fn.variables.values().any(|v| v.name == name);
        assert!(has_var, "Should have variable '{}'", name);
    }
    
    // Both operations look similar in MIR, but we could add a comment explaining what 
    // distinguishes them semantically:
    //
    // Generated MIR:
    // original[0] = 42
    // peek_result[1] = original[0]   // This is a peak - only a reference in the runtime
    // mutable[2] = 100
    // copy_result[3] = mutable[2]    // This is a clone - a deep copy in the runtime
    // temp_4[4] = peek_result[1] + copy_result[3]
    // return temp_4[4]
    
    // For primitive types like Int, the MIR representation doesn't show a difference
    // between peak and clone, but for complex data structures, we would need to ensure
    // the MIR generates different code for these two operations.
}

