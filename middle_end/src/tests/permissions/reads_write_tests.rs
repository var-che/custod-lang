use crate::hir::{
    HirProgram, HirStatement, HirVariable, HirValue,
    PermissionInfo, PermissionChecker, TypeEnvironment
};
use front_end::types::Type;  // Changed to use Type from ast
use front_end::token::PermissionType;  // Using PermissionType instead of Permission

#[test]
fn test_reads_write_basic() {
    // Create a program with a single variable that has reads write permission
    let program = HirProgram {
        statements: vec![
            HirStatement::Declaration(HirVariable {
                name: "counter".to_string(),
                typ: Type::I64,  // HirVariable expects Type directly, not PermissionedType
                permissions: PermissionInfo::new(vec![
                    PermissionType::Reads,  // Using PermissionType instead of Permission
                    PermissionType::Write
                ]),
                initializer: Some(HirValue::Number(0, Type::I64)),
            }),
        ],
        type_info: TypeEnvironment::default(),
    };

    // Verify that this is a valid permission combination
    let result = PermissionChecker::check_program(&program);
    assert!(result.is_ok(), "reads write should be a valid permission combination");
}

#[test]
fn test_reads_write_exclusive_write() {
    let program = HirProgram {
        statements: vec![
            // First declaration: reads write counter = 0
            HirStatement::Declaration(HirVariable {
                name: "counter".to_string(),
                typ: Type::I64,
                permissions: PermissionInfo::new(vec![
                    PermissionType::Reads,
                    PermissionType::Write
                ]),
                initializer: Some(HirValue::Number(0, Type::I64)),
            }),
            // Invalid: trying to get write permission to counter
            HirStatement::Declaration(HirVariable {
                name: "other".to_string(),
                typ: Type::I64,
                permissions: PermissionInfo::new(vec![
                    PermissionType::Write  // This should fail - counter already has exclusive write
                ]),
                initializer: Some(HirValue::Variable("counter".to_string(), Type::I64)),
            }),
        ],
        type_info: TypeEnvironment::default(),
    };

    // This should fail because only one variable can have write permission
    let result = PermissionChecker::check_program(&program);
    assert!(result.is_err(), "Should not allow multiple write permissions to same data");
    assert!(result.unwrap_err().contains("write permission"), 
           "Error should mention write permission conflict");
}