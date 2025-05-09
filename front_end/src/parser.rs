use crate::ast::{Statement, Expression};
use crate::token::TokenType;
use crate::types::{Type, Permission, PermissionedType};

pub struct Parser {
    input: String,
    position: usize,
    line: usize,
    column: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Parser {
            input: input.to_string(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn parse(&mut self) -> Result<Statement, String> {
        let mut statements = Vec::new();

        while self.position < self.input.len() {
            self.skip_whitespace();
            if self.position >= self.input.len() {
                break;
            }

            let statement = self.parse_statement()?;
            statements.push(statement);
        }

        if statements.len() == 1 {
            Ok(statements.remove(0))
        } else {
            Ok(Statement::Block(statements))
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        self.skip_whitespace();
        let word = self.read_word();

        match word.as_str() {
            "reads" | "read" => {
                let mut permissions = vec![
                    if word == "reads" { Permission::Reads } else { Permission::Read }
                ];
                
                self.skip_whitespace();
                if self.peek_word() == "write" {
                    self.read_word(); // consume "write"
                    permissions.push(Permission::Write);
                }

                self.skip_whitespace();
                let name = self.read_word();
                
                self.skip_whitespace();
                if self.peek_char() != '=' {
                    return Err(format!("Expected '=', found '{}'", self.peek_char()));
                }
                self.position += 1; // consume '='
                
                self.skip_whitespace();
                
                // Check for clone keyword
                let initializer = if self.peek_word() == "clone" {
                    self.read_word(); // consume "clone"
                    self.skip_whitespace();
                    Expression::Clone(Box::new(Expression::Variable(self.read_word())))
                } else {
                    self.parse_expression()?
                };

                Ok(Statement::Declaration {
                    name,
                    typ: PermissionedType::new(Type::I64, permissions),
                    initializer: Some(initializer),
                })
            },
            "write" => {
                // Handle write-only variable declaration
                let permissions = vec![Permission::Write];
                
                self.skip_whitespace();
                let name = self.read_word();
                
                self.skip_whitespace();
                if self.peek_char() != '=' {
                    return Err(format!("Expected '=', found '{}'", self.peek_char()));
                }
                self.position += 1; // consume '='
                
                self.skip_whitespace();
                let initializer = self.parse_expression()?;

                Ok(Statement::Declaration {
                    name,
                    typ: PermissionedType::new(Type::I64, permissions),
                    initializer: Some(initializer),
                })
            },
            "print" => {
                self.skip_whitespace();
                let expr = self.parse_expression()?;
                Ok(Statement::Print(expr))
            },
            _ => {
                // Handle any variable name for assignment
                self.skip_whitespace();
                if self.peek_char() == '+' && self.peek_next_char() == '=' {
                    self.position += 2; // skip +=
                    
                    self.skip_whitespace();
                    let value = self.parse_expression()?;
                    
                    Ok(Statement::Assignment {
                        target: word.clone(),
                        value: Expression::Binary {
                            left: Box::new(Expression::Variable(word)),
                            operator: TokenType::Plus,
                            right: Box::new(value),
                        },
                        target_type: PermissionedType::new(Type::I64, vec![Permission::Write]),
                    })
                } else {
                    Err(format!("Expected '+=', found '{}'", self.peek_char()))
                }
            }
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.skip_whitespace();
        
        if let Some(digit) = self.peek_char().to_digit(10) {
            let mut number = 0;
            while let Some(d) = self.peek_char().to_digit(10) {
                number = number * 10 + d as i64;
                self.position += 1;
            }
            Ok(Expression::Number(number))
        } else if self.peek_char().is_alphabetic() {
            Ok(Expression::Variable(self.read_word()))
        } else {
            Err(format!("Invalid expression at line {}, column {}", self.line, self.column))
        }
    }

    // Helper methods
    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() {
            let c = self.input[self.position..].chars().next();
            match c {
                Some(c) if c.is_whitespace() => {
                    if c == '\n' {
                        self.line += 1;
                        self.column = 1;
                    } else {
                        self.column += 1;
                    }
                    self.position += 1;
                }
                _ => break,
            }
        }
    }

    fn read_word(&mut self) -> String {
        self.skip_whitespace();
        let mut word = String::new();
        
        while self.position < self.input.len() {
            let c = self.peek_char();
            if !c.is_alphanumeric() && c != '_' {
                break;
            }
            word.push(c);
            self.position += 1;
            self.column += 1;
        }
        
        word
    }

    fn peek_word(&mut self) -> String {
        let saved_pos = self.position;
        let saved_col = self.column;
        let word = self.read_word();
        self.position = saved_pos;
        self.column = saved_col;
        word
    }

    fn peek_char(&self) -> char {
        self.input[self.position..].chars().next().unwrap_or('\0')
    }

    fn peek_next_char(&self) -> char {
        if self.position + 1 < self.input.len() {
            self.input[self.position + 1..].chars().next().unwrap()
        } else {
            '\0'
        }
    }

    fn expect_token(&mut self, expected: char) -> Result<(), String> {
        self.skip_whitespace();
        if self.peek_char() == expected {
            self.position += 1;
            self.column += 1;
            Ok(())
        } else {
            Err(format!("Expected '{}', found '{}'", expected, self.peek_char()))
        }
    }
}