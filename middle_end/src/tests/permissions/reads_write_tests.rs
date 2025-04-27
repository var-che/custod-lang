use crate::hir::{ *, TypeEnvironment};
use front_end::token::PermissionType;

#[test]
fn test_reads_write_basic() {
    let program = HirProgram {
        statements: vec![
            HirStatement::Declaration(
                HirVariable {
                    name: "counter".to_string(),
                    typ: Type::I64,
                    permissions: PermissionInfo::new(vec![
                        PermissionType::Reads,
                        PermissionType::Write
                    ]),
                    initializer: Some(HirValue::Number(100, Type::I64)),
                }
            ),
        ],
        type_info: TypeEnvironment::default(),
    };

    let result = PermissionChecker::check_program(&program);
    assert!(result.is_ok());
}

#[test]
fn test_reads_write_others_can_read() {
    let program = HirProgram {
        statements: vec![
            // First declaration with reads,write permission
            HirStatement::Declaration(
                HirVariable {
                    name: "counter".to_string(),
                    typ: Type::I64,
                    permissions: PermissionInfo::new(vec![
                        PermissionType::Reads,
                        PermissionType::Write
                    ]),
                    initializer: Some(HirValue::Number(100, Type::I64)),
                }
            ),
            // Another variable reading from counter
            HirStatement::Declaration(
                HirVariable {
                    name: "reader".to_string(),
                    typ: Type::I64,
                    permissions: PermissionInfo::new(vec![PermissionType::Read]),
                    initializer: Some(HirValue::Variable("counter".to_string(), Type::I64)),
                }
            ),
        ],
        type_info: TypeEnvironment::default(),
    };

    let result = PermissionChecker::check_program(&program);
    assert!(result.is_ok());
}

#[test]
fn test_reads_write_others_cannot_write() {
    let program = HirProgram {
        statements: vec![
            // First declaration with reads,write permission
            HirStatement::Declaration(
                HirVariable {
                    name: "counter".to_string(),
                    typ: Type::I64,
                    permissions: PermissionInfo::new(vec![
                        PermissionType::Reads,
                        PermissionType::Write
                    ]),
                    initializer: Some(HirValue::Number(100, Type::I64)),
                }
            ),
            // Another variable trying to write to counter
            HirStatement::Declaration(
                HirVariable {
                    name: "writer".to_string(),
                    typ: Type::I64,
                    permissions: PermissionInfo::new(vec![PermissionType::Write]),
                    initializer: Some(HirValue::Variable("counter".to_string(), Type::I64)),
                }
            ),
        ],
        type_info: TypeEnvironment::default(),
    };

    let result = PermissionChecker::check_program(&program);
    assert!(result.is_err());
}

#[test]
fn test_reads_write_multiple_readers() {
    // Testing this code in our language:
    // reads write c = 55
    // read a = c     // this is good
    // read d = c     // good
    
    let program = HirProgram {
        statements: vec![
            // First declaration with reads,write permission
            HirStatement::Declaration(
                HirVariable {
                    name: "c".to_string(),
                    typ: Type::I64,
                    permissions: PermissionInfo::new(vec![
                        PermissionType::Reads,
                        PermissionType::Write
                    ]),
                    initializer: Some(HirValue::Number(55, Type::I64)),
                }
            ),
            // First reader
            HirStatement::Declaration(
                HirVariable {
                    name: "a".to_string(),
                    typ: Type::I64,
                    permissions: PermissionInfo::new(vec![PermissionType::Read]),
                    initializer: Some(HirValue::Variable("c".to_string(), Type::I64)),
                }
            ),
            // Second reader
            HirStatement::Declaration(
                HirVariable {
                    name: "d".to_string(),
                    typ: Type::I64,
                    permissions: PermissionInfo::new(vec![PermissionType::Read]),
                    initializer: Some(HirValue::Variable("c".to_string(), Type::I64)),
                }
            ),
        ],
        type_info: TypeEnvironment::default(),
    };

    let result = PermissionChecker::check_program(&program);
    assert!(result.is_ok());
}

#[test]
fn test_reads_write_multiple_writes() {
    // Testing this code in our language:
    // reads write counter = 55
    // counter = 66   // Valid - owner can write
    // counter = 77   // Valid - owner can write multiple times
    
    let program = HirProgram {
        statements: vec![
            // Initial declaration with reads,write permission
            HirStatement::Declaration(
                HirVariable {
                    name: "counter".to_string(),
                    typ: Type::I64,
                    permissions: PermissionInfo::new(vec![
                        PermissionType::Reads,
                        PermissionType::Write
                    ]),
                    initializer: Some(HirValue::Number(55, Type::I64)),
                }
            ),
            // First write
            HirStatement::Assignment(
                HirAssignment {
                    target: "counter".to_string(),
                    value: HirValue::Number(66, Type::I64),
                    permissions_used: vec![PermissionType::Write]
                }
            ),
            // Second write
            HirStatement::Assignment(
                HirAssignment {
                    target: "counter".to_string(),
                    value: HirValue::Number(77, Type::I64),
                    permissions_used: vec![PermissionType::Write]
                }
            ),
        ],
        type_info: TypeEnvironment::default(),
    };

    let result = PermissionChecker::check_program(&program);
    assert!(result.is_ok());
}