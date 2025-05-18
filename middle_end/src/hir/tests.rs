//! Tests for HIR conversion
//!
//! This module contains tests for the HIR conversion functionality.

#[cfg(test)]
mod hir_conversion_tests {
    use crate::hir::converter::convert_statements_to_hir;

    use super::super::*;
    use front_end::parser::Parser;
    
    #[test]
    fn test_basic_hir_conversion() {
        // Define a simple program source
        let source = r#"
            reads write counter: Int = 5
            
            fn increment(reads amount: Int) -> Int {
                counter = counter + amount
                return counter
            }
            
            read result = increment(10)
        "#;
        
        // Parse using the front-end parser
        let mut parser = front_end::parser::Parser::from_source(source);
        let ast_statements = parser.parse_statements();
        
        // Check that parsing succeeded
        assert!(!ast_statements.is_empty(), "Parser should produce at least one statement");
        
        // Convert AST to HIR
        let hir_program = convert_statements_to_hir(ast_statements);
        
        // Verify basic HIR structure
        assert_eq!(hir_program.statements.len(), 3, "HIR should have three statements");
        
        // Check the first statement (variable declaration)
        match &hir_program.statements[0] {
            HirStatement::Declaration(var) => {
                assert_eq!(var.name, "counter", "First statement should declare 'counter'");
                assert_eq!(var.typ, front_end::types::Type::Int, "Counter should be an Int");
                
                // Check permissions
                assert!(var.permissions.contains(&Permission::Read), "Counter should have Read permission");
                assert!(var.permissions.contains(&Permission::Write), "Counter should have Write permission");
                
                // Check initializer
                match &var.initializer {
                    Some(HirExpression::Integer(val)) => {
                        assert_eq!(*val, 5, "Counter should be initialized to 5");
                    },
                    _ => panic!("Counter should have an Integer initializer"),
                }
            },
            _ => panic!("First HIR statement should be a Declaration"),
        }
        
        // Check the second statement (function declaration)
        match &hir_program.statements[1] {
            HirStatement::Function(func) => {
                assert_eq!(func.name, "increment", "Second statement should declare 'increment'");
                assert_eq!(func.parameters.len(), 1, "Function should have one parameter");
                assert_eq!(func.parameters[0].name, "amount", "Parameter should be named 'amount'");
                assert_eq!(func.body.len(), 2, "Function body should have two statements");
                assert_eq!(func.return_type, Some(front_end::types::Type::Int), "Function should return Int");
            },
            _ => panic!("Second HIR statement should be a Function"),
        }
        
        // Check the third statement (variable declaration with function call)
        match &hir_program.statements[2] {
            HirStatement::Declaration(var) => {
                assert_eq!(var.name, "result", "Third statement should declare 'result'");
                
                // Check that initializer is a function call
                match &var.initializer {
                    Some(HirExpression::Call { function, arguments, .. }) => {
                        assert_eq!(function, "increment", "Should call 'increment' function");
                        assert_eq!(arguments.len(), 1, "Call should have one argument");
                        
                        // Check the argument
                        match &arguments[0] {
                            HirExpression::Integer(val) => {
                                assert_eq!(*val, 10, "Argument should be 10");
                            },
                            _ => panic!("Argument should be an Integer"),
                        }
                    },
                    _ => panic!("Result initializer should be a function call"),
                }
            },
            _ => panic!("Third HIR statement should be a Declaration"),
        }
        
        // Verify type information was collected
        assert!(hir_program.type_info.variables.contains_key("counter"), "Type info should include 'counter'");
        assert!(hir_program.type_info.variables.contains_key("result"), "Type info should include 'result'");
        assert!(hir_program.type_info.variables.contains_key("amount"), "Type info should include 'amount'");
        assert!(hir_program.type_info.functions.contains_key("increment"), "Type info should include 'increment'");
        
        // Check result of validation
        let validation_result = validate_hir(&hir_program);
        assert!(validation_result.is_ok(), "HIR validation should pass");
    }
}
