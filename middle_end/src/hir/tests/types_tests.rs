use crate::hir::{types::*, PermissionInfo};
use front_end::types::Type;
use front_end::token::{TokenType, PermissionType};

#[test]
fn test_hir_value_creation() {
    // Test basic value constructors
    let num = HirValue::Number(42, Type::I64);
    let var = HirValue::Variable("x".to_string(), Type::I64);
    
    // Test binary expression
    let binary = HirValue::Binary {
        left: Box::new(num),
        operator: TokenType::Plus,
        right: Box::new(var),
        result_type: Type::I64,
    };
    
    // Verify structure
    match binary {
        HirValue::Binary { left, operator, right, result_type } => {
            assert_eq!(operator, TokenType::Plus);
            assert_eq!(result_type, Type::I64);
            
            match *left {
                HirValue::Number(n, t) => {
                    assert_eq!(n, 42);
                    assert_eq!(t, Type::I64);
                },
                _ => panic!("Expected number"),
            }
            
            match *right {
                HirValue::Variable(name, t) => {
                    assert_eq!(name, "x");
                    assert_eq!(t, Type::I64);
                },
                _ => panic!("Expected variable"),
            }
        },
        _ => panic!("Expected binary expression"),
    }
}

#[test]
fn test_hir_statement_hierarchy() {
    // Create a declaration statement
    let var = HirVariable {
        name: "x".to_string(),
        typ: Type::I64,
        permissions: PermissionInfo::new(vec![PermissionType::Read, PermissionType::Write]),
        initializer: Some(HirValue::Number(10, Type::I64)),
    };
    
    let decl = HirStatement::Declaration(var);
    
    // Create a block containing the declaration
    let block = HirStatement::AtomicBlock(vec![decl]);
    
    // Verify structure
    match block {
        HirStatement::AtomicBlock(statements) => {
            assert_eq!(statements.len(), 1);
            
            match &statements[0] {
                HirStatement::Declaration(var) => {
                    assert_eq!(var.name, "x");
                    assert_eq!(var.typ, Type::I64);
                    
                    match &var.initializer {
                        Some(HirValue::Number(n, _)) => assert_eq!(*n, 10),
                        _ => panic!("Expected number initializer"),
                    }
                },
                _ => panic!("Expected declaration"),
            }
        },
        _ => panic!("Expected atomic block"),
    }
}