use crate::hir::{HirAssignment, HirProgram, HirStatement, HirValue, HirVariable, PermissionInfo};
use crate::mir::{MirFunction, MirInstruction, MirValue, lower_hir};
use front_end::types::Type;
use front_end::token::{PermissionType, TokenType};

/// Helper function to create a basic HirProgram with a counter variable
fn create_counter_declaration(initial_value: HirValue) -> HirProgram {
    HirProgram {
        statements: vec![
            HirStatement::Declaration(
                HirVariable {
                    name: "counter".to_string(),
                    typ: Type::I64,
                    permissions: PermissionInfo::new(vec![
                        PermissionType::Read,
                        PermissionType::Write
                    ]),
                    initializer: Some(initial_value),
                }
            ),
        ],
        type_info: Default::default(),
    }
}

#[test]
fn test_basic_arithmetic() {
    let program = create_counter_declaration(
        HirValue::Binary {
            left: Box::new(HirValue::Number(2, Type::I64)),
            right: Box::new(HirValue::Number(2, Type::I64)),
            operator: TokenType::Plus,
            result_type: Type::I64,
        }
    );

    let mir = lower_hir(&program);
    let expected = MirFunction {
        instructions: vec![
            MirInstruction::WriteBarrier { reference: "counter".to_string() },
            MirInstruction::Load { target: 0, value: MirValue::Number(2) },
            MirInstruction::Load { target: 1, value: MirValue::Number(2) },
            MirInstruction::Add { target: 2, left: MirValue::Temporary(0), right: MirValue::Temporary(1) },
            MirInstruction::Store { target: "counter".to_string(), value: MirValue::Temporary(2) },
        ]
    };

    assert_eq!(mir, expected);
}

#[test]
fn test_increment() {
    let program = HirProgram {
        statements: vec![
            HirStatement::Assignment(
                HirAssignment {
                    target: "counter".to_string(),
                    value: HirValue::Number(1, Type::I64),
                    permissions_used: vec![PermissionType::Write],
                }
            ),
        ],
        type_info: Default::default(),
    };

    let mir = lower_hir(&program);
    let expected = MirFunction {
        instructions: vec![
            MirInstruction::WriteBarrier { reference: "counter".to_string() },
            MirInstruction::Load { target: 0, value: MirValue::Variable("counter".to_string()) },
            MirInstruction::Add { target: 1, left: MirValue::Temporary(0), right: MirValue::Number(1) },
            MirInstruction::Store { target: "counter".to_string(), value: MirValue::Temporary(1) },
        ]
    };

    assert_eq!(mir, expected);
}

#[test]
fn test_complete_program() {
    let binary_op = HirValue::Binary {
        left: Box::new(HirValue::Number(2, Type::I64)),
        right: Box::new(HirValue::Number(2, Type::I64)),
        operator: TokenType::Plus,
        result_type: Type::I64,
    };

    let program = HirProgram {
        statements: vec![
            HirStatement::Declaration(
                HirVariable {
                    name: "counter".to_string(),
                    typ: Type::I64,
                    permissions: PermissionInfo::new(vec![
                        PermissionType::Read,
                        PermissionType::Write
                    ]),
                    initializer: Some(binary_op),
                }
            ),
            HirStatement::Assignment(
                HirAssignment {
                    target: "counter".to_string(),
                    value: HirValue::Number(1, Type::I64),
                    permissions_used: vec![PermissionType::Write],
                }
            ),
            HirStatement::Print(HirValue::Variable("counter".to_string(), Type::I64)),
        ],
        type_info: Default::default(),
    };

    let mir = lower_hir(&program);
    let expected = create_complete_program_mir();

    assert_eq!(mir, expected);
}

/// Helper function to create the expected MIR for the complete program test
fn create_complete_program_mir() -> MirFunction {
    MirFunction {
        instructions: vec![
            MirInstruction::WriteBarrier { reference: "counter".to_string() },
            MirInstruction::Load { target: 0, value: MirValue::Number(2) },
            MirInstruction::Load { target: 1, value: MirValue::Number(2) },
            MirInstruction::Add { target: 2, left: MirValue::Temporary(0), right: MirValue::Temporary(1) },
            MirInstruction::Store { target: "counter".to_string(), value: MirValue::Temporary(2) },
            MirInstruction::WriteBarrier { reference: "counter".to_string() },
            MirInstruction::Load { target: 3, value: MirValue::Variable("counter".to_string()) },
            MirInstruction::Add { target: 4, left: MirValue::Temporary(3), right: MirValue::Number(1) },
            MirInstruction::Store { target: "counter".to_string(), value: MirValue::Temporary(4) },
            MirInstruction::ReadBarrier { reference: "counter".to_string() },
            MirInstruction::Print { value: MirValue::Variable("counter".to_string()) },
        ]
    }
}