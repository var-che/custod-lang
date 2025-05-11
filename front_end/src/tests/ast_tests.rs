use crate::ast::{Expression, Statement, FunctionBuilder};
use crate::token::TokenType;
use crate::types::{Type, Permission, PermissionedType};

#[test]
fn test_expression_binary_creation() {
    // Test that Expression::new_binary correctly builds a binary expression
    let left = Expression::Number(5);
    let right = Expression::Number(7);
    let operator = TokenType::Plus;
    
    let binary = Expression::new_binary(left, operator, right);
    
    match binary {
        Expression::Binary { left: boxed_left, operator: op, right: boxed_right } => {
            assert_eq!(op, TokenType::Plus);
            
            match *boxed_left {
                Expression::Number(n) => assert_eq!(n, 5),
                _ => panic!("Expected Number expression"),
            }
            
            match *boxed_right {
                Expression::Number(n) => assert_eq!(n, 7),
                _ => panic!("Expected Number expression"),
            }
        },
        _ => panic!("Expected Binary expression"),
    }
}

#[test]
fn test_statement_declaration_factory() {
    // Test Statement::new_declaration factory method
    let name = "counter".to_string();
    let typ = PermissionedType::new(Type::I64, vec![Permission::Reads]);
    let initializer = Some(Expression::Number(42));
    
    let decl = Statement::new_declaration(name.clone(), typ.clone(), initializer.clone());
    
    match decl {
        Statement::Declaration { name: n, typ: t, initializer: init } => {
            assert_eq!(n, name);
            assert_eq!(t, typ);
            assert_eq!(init, initializer);
        },
        _ => panic!("Expected Declaration statement"),
    }
}

#[test]
fn test_function_builder() {
    // Test that FunctionBuilder correctly constructs a function
    let builder = FunctionBuilder::new("add".to_string())
        .as_behavior(false)
        .with_parameter(
            "x".to_string(), 
            PermissionedType::new(Type::I64, vec![Permission::Reads])
        )
        .with_parameter(
            "y".to_string(), 
            PermissionedType::new(Type::I64, vec![Permission::Reads])
        )
        .with_return_type(Some(PermissionedType::new(Type::I64, vec![])))
        .with_body(vec![
            Statement::Return(Expression::new_binary(
                Expression::Variable("x".to_string()),
                TokenType::Plus,
                Expression::Variable("y".to_string())
            ))
        ]);
    
    let function = builder.build();
    
    match function {
        Statement::Function { name, params, body, return_type, is_behavior } => {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].0, "x");
            assert_eq!(params[1].0, "y");
            assert!(return_type.is_some());
            assert_eq!(body.len(), 1);
            assert!(!is_behavior);
            
            // Check the return statement
            match &body[0] {
                Statement::Return(expr) => {
                    match expr {
                        Expression::Binary { left, operator, right } => {
                            assert_eq!(*operator, TokenType::Plus);
                            
                            match **left {
                                Expression::Variable(ref name) => assert_eq!(name, "x"),
                                _ => panic!("Expected Variable expression"),
                            }
                            
                            match **right {
                                Expression::Variable(ref name) => assert_eq!(name, "y"),
                                _ => panic!("Expected Variable expression"),
                            }
                        },
                        _ => panic!("Expected Binary expression"),
                    }
                },
                _ => panic!("Expected Return statement"),
            }
        },
        _ => panic!("Expected Function statement"),
    }
}

#[test]
fn test_permission_storage() {
    // Test that permissions are correctly stored in AST nodes
    let read_write_type = PermissionedType::new(Type::I64, vec![Permission::Read, Permission::Write]);
    
    let decl = Statement::new_declaration(
        "x".to_string(),
        read_write_type.clone(),
        None
    );
    
    match decl {
        Statement::Declaration { typ, .. } => {
            assert_eq!(typ.permissions.len(), 2, "Should have 2 permissions");
            assert!(typ.permissions.contains(&Permission::Read), "Should contain Read permission");
            assert!(typ.permissions.contains(&Permission::Write), "Should contain Write permission");
        },
        _ => panic!("Expected Declaration statement"),
    }
}