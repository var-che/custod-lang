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

