pub mod token;
pub mod ast;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_creation() {
        use ast::{Expression, Statement};
        use token::TokenType;

        let stmt = Statement::Declaration {
            name: "counter".to_string(),
            permissions: vec![TokenType::Read, TokenType::Write],
            initializer: Some(Expression::Number(42)),
        };

        assert!(matches!(stmt, Statement::Declaration { .. }));
    }
}