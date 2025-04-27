use crate::token::TokenType;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Number(i64),
    Binary {
        left: Box<Expression>,
        operator: TokenType,
        right: Box<Expression>,
    },
    Variable(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Declaration {
        name: String,
        permissions: Vec<TokenType>,
        initializer: Option<Expression>,
    },
    Assignment {
        target: String,
        value: Expression,
    },
    Print(Expression),
}