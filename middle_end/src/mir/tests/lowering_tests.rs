use crate::hir::{
    HirProgram, HirStatement, HirVariable, HirValue, HirAssignment, PermissionInfo
};
use crate::mir::{
    lowering::lower_hir,
    types::{MirFunction, MirInstruction, MirValue}
};
use front_end::token::PermissionType;
use front_end::types::Type;

#[test]
fn test_simple_declaration_and_assignment() {
    // Create a program with:
    // 1. Variable declaration: x = 42
    // 2. Variable assignment: x = 100
    let program = HirProgram {
        statements: vec![
            // Declaration: x = 42
            HirStatement::Declaration(HirVariable {
                name: "x".to_string(),
                typ: Type::I64,
                permissions: PermissionInfo::new(vec![
                    PermissionType::Read,
                    PermissionType::Write
                ]),
                initializer: Some(HirValue::Number(42, Type::I64)),
            }),
            // Assignment: x = 100
            HirStatement::Assignment(HirAssignment {
                target: "x".to_string(),
                value: HirValue::Number(100, Type::I64),
                permissions_used: vec![PermissionType::Write],
            }),
        ],
        type_info: Default::default(),
    };

    // Lower the HIR to MIR
    let mir = lower_hir(&program);
    
    // Print MIR for debugging
    println!("Generated MIR:\n{:?}", mir.instructions);
    
    // Verify we got the expected number of instructions
    // WriteBarrier + Store + WriteBarrier + Store = 4 instructions
    assert_eq!(mir.instructions.len(), 4, "Expected 4 MIR instructions");
    
    // Check the first instruction (WriteBarrier for declaration)
    match &mir.instructions[0] {
        MirInstruction::WriteBarrier { reference } => {
            assert_eq!(reference, "x", "First instruction should be WriteBarrier for x");
        },
        _ => panic!("Expected WriteBarrier as first instruction"),
    }
    
    // Check the second instruction (Store for declaration)
    match &mir.instructions[1] {
        MirInstruction::Store { target, value } => {
            assert_eq!(target, "x", "Second instruction should store to x");
            assert_eq!(*value, MirValue::Number(42), "Initial value should be 42");
        },
        _ => panic!("Expected Store as second instruction"),
    }
    
    // Check the third instruction (WriteBarrier for assignment)
    match &mir.instructions[2] {
        MirInstruction::WriteBarrier { reference } => {
            assert_eq!(reference, "x", "Third instruction should be WriteBarrier for x");
        },
        _ => panic!("Expected WriteBarrier as third instruction"),
    }
    
    // Check the fourth instruction (Store for assignment)
    match &mir.instructions[3] {
        MirInstruction::Store { target, value } => {
            assert_eq!(target, "x", "Fourth instruction should store to x");
            assert_eq!(*value, MirValue::Number(100), "New value should be 100");
        },
        _ => panic!("Expected Store as fourth instruction"),
    }
}

#[test]
fn test_binary_operation() {
    // Create a program with a binary operation: x = 5 + 7
    let program = HirProgram {
        statements: vec![
            HirStatement::Declaration(HirVariable {
                name: "x".to_string(),
                typ: Type::I64,
                permissions: PermissionInfo::new(vec![
                    PermissionType::Read,
                    PermissionType::Write
                ]),
                initializer: Some(HirValue::Binary {
                    left: Box::new(HirValue::Number(5, Type::I64)),
                    operator: front_end::token::TokenType::Plus,
                    right: Box::new(HirValue::Number(7, Type::I64)),
                    result_type: Type::I64,
                }),
            }),
        ],
        type_info: Default::default(),
    };

    // Lower HIR to MIR
    let mir = lower_hir(&program);
    
    // Print for debugging
    println!("Binary operation MIR:\n{:?}", mir.instructions);
    
    // Verify we get the expected instructions:
    // 1. WriteBarrier
    // 2. Add operation
    // 3. Store result
    assert_eq!(mir.instructions.len(), 3, "Expected 3 MIR instructions for binary operation");
    
    // First instruction: WriteBarrier
    match &mir.instructions[0] {
        MirInstruction::WriteBarrier { reference } => {
            assert_eq!(reference, "x", "WriteBarrier should be for variable x");
        },
        _ => panic!("Expected WriteBarrier as first instruction"),
    }
    
    // Second instruction: Add operation
    match &mir.instructions[1] {
        MirInstruction::Add { target, left, right } => {
            // Check the temporary register number
            assert_eq!(*target, 0, "Addition result should go to temporary 0");
            
            // Check operands
            assert_eq!(*left, MirValue::Number(5), "Left operand should be 5");
            assert_eq!(*right, MirValue::Number(7), "Right operand should be 7");
        },
        _ => panic!("Expected Add as second instruction"),
    }
    
    // Third instruction: Store
    match &mir.instructions[2] {
        MirInstruction::Store { target, value } => {
            assert_eq!(target, "x", "Store target should be x");
            assert_eq!(*value, MirValue::Temporary(0), "Store value should be temporary 0");
        },
        _ => panic!("Expected Store as third instruction"),
    }
}

