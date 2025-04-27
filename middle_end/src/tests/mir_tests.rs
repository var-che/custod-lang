use crate::hir::{HirAssignment, HirProgram, HirStatement, HirValue, HirVariable, PermissionInfo, Type};
use crate::mir::{MirFunction, MirInstruction, MirValue, lower_hir};
use front_end::token::{PermissionType, TokenType};

#[test]
fn test_basic_arithmetic() {
    // Testing lowering of:
    // read write counter = 2 + 2
    
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
                    initializer: Some(HirValue::Binary {
                        left: Box::new(HirValue::Number(2, Type::I64)),
                        right: Box::new(HirValue::Number(2, Type::I64)),
                        operator: TokenType::Plus,
                        result_type: Type::I64,
                    }),
                }
            ),
        ],
        type_info: Default::default(),
    };

    let mir = lower_hir(&program);

    // Expected MIR instructions
    let expected = MirFunction {
        instructions: vec![
            MirInstruction::WriteBarrier {
                reference: "counter".to_string()
            },
            MirInstruction::Load {
                target: 0,
                value: MirValue::Number(2)
            },
            MirInstruction::Load {
                target: 1,
                value: MirValue::Number(2)
            },
            MirInstruction::Add {
                target: 2,
                left: MirValue::Temporary(0),
                right: MirValue::Temporary(1)
            },
            MirInstruction::Store {
                target: "counter".to_string(),
                value: MirValue::Temporary(2)
            },
        ]
    };

    assert_eq!(mir, expected);
}

#[test]
fn test_increment() {
    // Testing lowering of:
    // counter += 1
    
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
            MirInstruction::WriteBarrier {
                reference: "counter".to_string()
            },
            MirInstruction::Load {
                target: 0,
                value: MirValue::Reference("counter".to_string())
            },
            MirInstruction::Add {
                target: 1,
                left: MirValue::Temporary(0),
                right: MirValue::Number(1)
            },
            MirInstruction::Store {
                target: "counter".to_string(),
                value: MirValue::Temporary(1)
            },
        ]
    };

    assert_eq!(mir, expected);
}

#[test]
fn test_complete_program() {
    // Testing lowering of:
    // read write counter = 2 + 2
    // counter += 1
    // println(counter)
    
    let program = HirProgram {
        statements: vec![
            // Initial declaration
            HirStatement::Declaration(
                HirVariable {
                    name: "counter".to_string(),
                    typ: Type::I64,
                    permissions: PermissionInfo::new(vec![
                        PermissionType::Read,
                        PermissionType::Write
                    ]),
                    initializer: Some(HirValue::Binary {
                        left: Box::new(HirValue::Number(2, Type::I64)),
                        right: Box::new(HirValue::Number(2, Type::I64)),
                        operator: TokenType::Plus,
                        result_type: Type::I64,
                    }),
                }
            ),
            // Increment
            HirStatement::Assignment(
                HirAssignment {
                    target: "counter".to_string(),
                    value: HirValue::Number(1, Type::I64),
                    permissions_used: vec![PermissionType::Write],
                }
            ),
            // Print statement
            HirStatement::Print(HirValue::Variable("counter".to_string(), Type::I64)),
        ],
        type_info: Default::default(),
    };

    let mir = lower_hir(&program);

    let expected = MirFunction {
        instructions: vec![
            // 2 + 2
            MirInstruction::WriteBarrier { reference: "counter".to_string() },
            MirInstruction::Load { target: 0, value: MirValue::Number(2) },
            MirInstruction::Load { target: 1, value: MirValue::Number(2) },
            MirInstruction::Add { target: 2, left: MirValue::Temporary(0), right: MirValue::Temporary(1) },
            MirInstruction::Store { target: "counter".to_string(), value: MirValue::Temporary(2) },
            
            // counter += 1
            MirInstruction::WriteBarrier { reference: "counter".to_string() },
            MirInstruction::Load { target: 3, value: MirValue::Reference("counter".to_string()) },
            MirInstruction::Add { target: 4, left: MirValue::Temporary(3), right: MirValue::Number(1) },
            MirInstruction::Store { target: "counter".to_string(), value: MirValue::Temporary(4) },
            
            // println(counter)
            MirInstruction::ReadBarrier { reference: "counter".to_string() },
            MirInstruction::Print { value: MirValue::Reference("counter".to_string()) },
        ]
    };

    assert_eq!(mir, expected);
}