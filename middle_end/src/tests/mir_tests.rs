//! Tests for MIR generation
//!
//! This module contains tests for the MIR (Middle Intermediate Representation) generation.

use crate::hir::converter::convert_statements_to_hir;
use crate::mir::converter::convert_hir_to_mir;
use crate::mir::pretty::pretty_print_program;

use front_end::parser::Parser;

#[test]
fn test_basic_mir_generation() {
    // Define a simple test program
    let source = r#"
        reads write counter: Int = 5
        
        fn increment(reads amount: Int) -> Int {
            counter = counter + amount
            return counter
        }
        
        reads result = increment(10)
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
    println!("Generated MIR:\n{}", mir_output);
    
    // Basic checks on the MIR
    assert!(!mir_program.globals.is_empty(), "Should have at least one global variable");
    assert!(mir_program.globals.contains_key("counter"), "Should have 'counter' global variable");
    
    assert!(mir_program.functions.contains_key("increment"), "Should have 'increment' function");
    
    let increment_fn = &mir_program.functions["increment"];
    assert_eq!(increment_fn.name, "increment", "Function should be named 'increment'");
    assert_eq!(increment_fn.parameters.len(), 1, "Should have one parameter");
    assert!(increment_fn.return_type.is_some(), "Should have a return type");
    assert!(!increment_fn.blocks.is_empty(), "Should have at least one block");
    
    // Check that blocks have reasonable instructions
    let has_add_instruction = increment_fn.blocks.iter()
        .flat_map(|block| &block.instructions)
        .any(|instr| {
            matches!(instr, crate::mir::types::Instruction::BinaryOp { op, .. } 
                if matches!(op, crate::mir::types::BinaryOperation::Add))
        });
        
    assert!(has_add_instruction, "Should have addition operation");
    
    let has_return = increment_fn.blocks.iter()
        .flat_map(|block| &block.instructions)
        .any(|instr| matches!(instr, crate::mir::types::Instruction::Return(_)));
        
    assert!(has_return, "Should have return instruction");
}
