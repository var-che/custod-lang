use crate::hir::converters::*;
use crate::hir::types::*;
use front_end::ast::{Statement, Expression};
use front_end::token::{TokenType, PermissionType};
use front_end::types::{Type, PermissionedType, Permission};

#[test]
fn test_convert_simple_expression() {
    // Create AST expression: 5 + 3
    let expr = Expression::Binary {
        left: Box::new(Expression::Number(5)),
        operator: TokenType::Plus,
        right: Box::new(Expression::Number(3)),
    };
    
    // Convert to HIR
    let hir_expr = convert_expression(expr);
    
    // Verify converted expression
    match hir_expr {
        HirValue::Binary { left, operator, right, result_type } => {
            assert_eq!(operator, TokenType::Plus);
            assert_eq!(result_type, Type::I64);
            
            match *left {
                HirValue::Number(n, _) => assert_eq!(n, 5),
                _ => panic!("Expected number"),
            }
            
            match *right {
                HirValue::Number(n, _) => assert_eq!(n, 3),
                _ => panic!("Expected number"),
            }
        },
        _ => panic!("Expected binary expression"),
    }
}

#[test]
fn test_convert_variable_declaration() {
    // Create AST statement: read write x = 42
    let stmt = Statement::Declaration {
        name: "x".to_string(),
        typ: PermissionedType {
            base_type: Type::I64,
            permissions: vec![Permission::Read, Permission::Write],
        },
        initializer: Some(Expression::Number(42)),
    };
    
    // Convert to HIR
    let hir_program = convert_to_hir(stmt);
    
    // Verify converted program
    assert_eq!(hir_program.statements.len(), 1);
    match &hir_program.statements[0] {
        HirStatement::Declaration(var) => {
            assert_eq!(var.name, "x");
            assert_eq!(var.typ, Type::I64);
            assert_eq!(var.permissions.permissions.len(), 2);
            assert!(var.permissions.permissions.contains(&PermissionType::Read));
            assert!(var.permissions.permissions.contains(&PermissionType::Write));
            
            match &var.initializer {
                Some(HirValue::Number(n, _)) => assert_eq!(*n, 42),
                _ => panic!("Expected number initializer"),
            }
        },
        _ => panic!("Expected declaration"),
    }
}

#[test]
fn test_convert_atomic_block() {
    // Create AST statements inside atomic block
    let stmt = Statement::AtomicBlock(vec![
        Statement::Print(Expression::Number(42)),
    ]);
    
    // Convert to HIR
    let hir_program = convert_to_hir(stmt);
    
    // Verify converted atomic block
    assert_eq!(hir_program.statements.len(), 1);
    match &hir_program.statements[0] {
        HirStatement::AtomicBlock(statements) => {
            assert_eq!(statements.len(), 1);
            match &statements[0] {
                HirStatement::Print(value) => {
                    match value {
                        HirValue::Number(n, _) => assert_eq!(*n, 42),
                        _ => panic!("Expected number"),
                    }
                },
                _ => panic!("Expected print statement"),
            }
        },
        _ => panic!("Expected atomic block"),
    }
}