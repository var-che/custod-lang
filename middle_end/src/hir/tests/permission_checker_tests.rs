use crate::hir::permissions::{PermissionChecker, PermissionInfo};
use crate::hir::types::*;
use front_end::token::PermissionType;
use front_end::types::Type;

#[test]
fn test_permission_checker_with_alias_tracking() {
    // Create a new checker
    let mut checker = PermissionChecker::new();
    
    // Create a variable declaration
    let x_var = HirVariable {
        name: "x".to_string(),
        typ: Type::I64,
        permissions: PermissionInfo::new(vec![PermissionType::Reads, PermissionType::Write]),
        initializer: Some(HirValue::Number(42, Type::I64)),
    };
    
    // Check the declaration (registers in alias table)
    let result = checker.check_declaration(&x_var);
    assert!(result.is_ok(), "Should successfully declare variable");
    
    // Create a read-only alias
    let y_var = HirVariable {
        name: "y".to_string(),
        typ: Type::I64,
        permissions: PermissionInfo::new(vec![PermissionType::Read]),
        initializer: Some(HirValue::Variable("x".to_string(), Type::I64)),
    };
    
    // Check the alias declaration
    let result = checker.check_declaration(&y_var);
    assert!(result.is_ok(), "Should successfully create read alias");
    
    // Print the current alias table
    println!("Alias table after creating variables:\n{}", checker.print_alias_table());
    
    // Test assignment to x (should succeed)
    let x_assign = HirAssignment {
        target: "x".to_string(),
        value: HirValue::Number(100, Type::I64),
        permissions_used: vec![PermissionType::Write],
    };
    
    let result = checker.check_assignment(&x_assign);
    assert!(result.is_ok(), "Should be able to write to x");
    
    // Attempt to assign to y (should fail - y is read-only)
    let y_assign = HirAssignment {
        target: "y".to_string(),
        value: HirValue::Number(200, Type::I64),
        permissions_used: vec![PermissionType::Write],
    };
    
    let result = checker.check_assignment(&y_assign);
    assert!(result.is_err(), "Should not be able to write to read-only alias y");
    
    // Print final alias table with access history
    println!("Final alias table with access history:\n{}", checker.print_alias_table());
}

#[test]
fn test_complex_aliasing_scenario() {
    let mut checker = PermissionChecker::new();
    
    // Original variable with reads+write
    let x_var = HirVariable {
        name: "x".to_string(),
        typ: Type::I64,
        permissions: PermissionInfo::new(vec![PermissionType::Reads, PermissionType::Write]),
        initializer: Some(HirValue::Number(42, Type::I64)),
    };
    checker.check_declaration(&x_var).unwrap();
    
    // First read-only alias
    let y_var = HirVariable {
        name: "y".to_string(),
        typ: Type::I64,
        permissions: PermissionInfo::new(vec![PermissionType::Read]),
        initializer: Some(HirValue::Variable("x".to_string(), Type::I64)),
    };
    checker.check_declaration(&y_var).unwrap();
    
    // Second read-only alias pointing to first alias
    let z_var = HirVariable {
        name: "z".to_string(),
        typ: Type::I64,
        permissions: PermissionInfo::new(vec![PermissionType::Read]),
        initializer: Some(HirValue::Variable("y".to_string(), Type::I64)),
    };
    checker.check_declaration(&z_var).unwrap();
    
    // Third variable with exclusive permissions
    let w_var = HirVariable {
        name: "w".to_string(),
        typ: Type::I64,
        permissions: PermissionInfo::new(vec![PermissionType::Read, PermissionType::Write]),
        initializer: Some(HirValue::Number(100, Type::I64)),
    };
    checker.check_declaration(&w_var).unwrap();
    
    // Try to consume x (should work - it's reads+write)
    let consume_x = HirValue::Consume(Box::new(HirValue::Variable("x".to_string(), Type::I64)));
    let result = checker.check_permissions(&consume_x);
    assert!(result.is_ok(), "Should be able to consume x");
    
    // Print alias table at this point
    println!("Complex aliasing scenario:\n{}", checker.print_alias_table());
}

#[test]
fn test_reads_writes_scenario() {
    let mut checker = PermissionChecker::new();
    
    // Create a variable with reads+writes permission
    let c_var = HirVariable {
        name: "c".to_string(),
        typ: Type::I64,
        permissions: PermissionInfo::new(vec![PermissionType::Reads, PermissionType::Writes]),
        initializer: Some(HirValue::Number(5, Type::I64)),
    };
    checker.check_declaration(&c_var).unwrap();
    
    // Create a write-only alias
    let d_var = HirVariable {
        name: "d".to_string(),
        typ: Type::I64,
        permissions: PermissionInfo::new(vec![PermissionType::Write]),
        initializer: Some(HirValue::Variable("c".to_string(), Type::I64)),
    };
    checker.check_declaration(&d_var).unwrap();
    
    // Create a read-only alias using peak
    let r_var = HirVariable {
        name: "r".to_string(),
        typ: Type::I64,
        permissions: PermissionInfo::new(vec![PermissionType::Read]),
        initializer: Some(HirValue::Peak(Box::new(HirValue::Variable("c".to_string(), Type::I64)))),
    };
    checker.check_declaration(&r_var).unwrap();
    
    // Verify access permissions
    assert!(checker.alias_table.check_read_access("c").is_ok(), "c should have read access");
    assert!(checker.alias_table.check_write_access("c").is_ok(), "c should have write access");
    assert!(checker.alias_table.check_write_access("d").is_ok(), "d should have write access");
    assert!(checker.alias_table.check_read_access("d").is_err(), "d should NOT have read access");
    assert!(checker.alias_table.check_read_access("r").is_ok(), "r should have read access");
    assert!(checker.alias_table.check_write_access("r").is_err(), "r should NOT have write access");
    
    println!("Reads+Writes permission scenario:\n{}", checker.print_alias_table());
}