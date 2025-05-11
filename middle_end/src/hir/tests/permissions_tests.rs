use crate::hir::permissions::*;
use crate::hir::types::*;
use front_end::token::PermissionType;
use front_end::types::Type;

#[test]
fn test_permission_info_basic() {
    // Test read-write combination
    let read_write = PermissionInfo::new(vec![
        PermissionType::Read,
        PermissionType::Write,
    ]);
    
    assert!(read_write.has_read_access());
    assert!(read_write.has_write_access());
    assert!(read_write.has_exclusive_permissions());
    assert!(!read_write.is_reads_write());
    
    // Test reads-write combination
    let reads_write = PermissionInfo::new(vec![
        PermissionType::Reads,
        PermissionType::Write,
    ]);
    
    assert!(reads_write.has_read_access());
    assert!(reads_write.has_write_access());
    assert!(!reads_write.has_exclusive_permissions());
    assert!(reads_write.is_reads_write());
    assert!(reads_write.can_be_consumed());
}

#[test]
fn test_permission_checker_valid_program() {
    // Create a valid program with proper permissions
    let program = HirProgram {
        statements: vec![
            HirStatement::Declaration(HirVariable {
                name: "x".to_string(),
                typ: Type::I64,
                permissions: PermissionInfo::new(vec![PermissionType::Reads]),
                initializer: Some(HirValue::Number(42, Type::I64)),
            }),
            HirStatement::Declaration(HirVariable {
                name: "y".to_string(),
                typ: Type::I64,
                permissions: PermissionInfo::new(vec![PermissionType::Read]),
                initializer: Some(HirValue::Variable("x".to_string(), Type::I64)),
            }),
        ],
        type_info: TypeEnvironment::default(),
    };
    
    // Check permissions
    let result = PermissionChecker::check_program(&program);
    assert!(result.is_ok(), "Valid program should pass permission check");
}

#[test]
fn test_permission_checker_invalid_program() {
    // Create an invalid program with permission violations
    let program = HirProgram {
        statements: vec![
            HirStatement::Declaration(HirVariable {
                name: "x".to_string(), 
                typ: Type::I64,
                permissions: PermissionInfo::new(vec![
                    PermissionType::Read,
                    PermissionType::Write,
                ]),
                initializer: Some(HirValue::Number(42, Type::I64)),
            }),
            // Try to get write access to an exclusive variable
            HirStatement::Declaration(HirVariable {
                name: "y".to_string(),
                typ: Type::I64,
                permissions: PermissionInfo::new(vec![PermissionType::Write]),
                initializer: Some(HirValue::Variable("x".to_string(), Type::I64)),
            }),
        ],
        type_info: TypeEnvironment::default(),
    };
    
    // Check permissions - should fail
    let result = PermissionChecker::check_program(&program);
    assert!(result.is_err(), "Invalid program should fail permission check");
}