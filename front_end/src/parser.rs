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
            "fn" => {
                // Read function name
                self.skip_whitespace();
                let name = self.read_word();
                
                // Handle parameters
                self.skip_whitespace();
                self.expect('(')?;
                let mut params = Vec::new();
                
                while self.peek_char() != ')' {
                    if !params.is_empty() {
                        self.expect(',')?;
                        self.skip_whitespace();
                    }
                    
                    // Read parameter permissions
                    let permissions = self.parse_permissions()?;
                    
                    // Read parameter name
                    self.skip_whitespace();
                    let param_name = self.read_word();
                    
                    // Read parameter type
                    self.skip_whitespace();
                    self.expect(':')?;
                    self.skip_whitespace();
                    let param_type = self.read_word();
                    
                    params.push((param_name, PermissionedType {
                        base_type: Type::from_str(&param_type)?,
                        permissions,
                    }));
                }
                self.expect(')')?;
                
                // Handle return type
                self.skip_whitespace();
                let return_type = if self.peek_char() == '-' && self.peek_next_char() == '>' {
                    self.position += 2; // skip ->
                    self.skip_whitespace();
                    let typ = self.read_word();
                    Some(PermissionedType {
                        base_type: Type::from_str(&typ)?,
                        permissions: vec![],
                    })
                } else {
                    None
                };
                
                // Parse function body
                self.skip_whitespace();
                self.expect('{')?;
                let mut body = Vec::new();
                
                while self.peek_char() != '}' {
                    self.skip_whitespace();
                    if self.peek_word() == "return" {
                        self.read_word(); // consume "return"
                        self.skip_whitespace();
                        let value = self.parse_expression()?;
                        body.push(Statement::Return(value));
                    } else {
                        body.push(self.parse_statement()?);
                    }
                }
                self.expect('}')?;
                
                Ok(Statement::Function {
                    name,
                    params,
                    body,
                    return_type,
                    is_behavior: false,  // Regular function
                })
            },
            "on" => {
                // Read behavior name
                self.skip_whitespace();
                let name = self.read_word();
                
                // Handle parameters
                self.skip_whitespace();
                self.expect('(')?;
                let mut params = Vec::new();
                
                while self.peek_char() != ')' {
                    if !params.is_empty() {
                        self.expect(',')?;
                        self.skip_whitespace();
                    }
                    
                    // Read parameter permissions
                    let permissions = self.parse_permissions()?;
                    
                    // Read parameter name
                    self.skip_whitespace();
                    let param_name = self.read_word();
                    
                    // Read parameter type
                    self.skip_whitespace();
                    self.expect(':')?;
                    self.skip_whitespace();
                    let param_type = self.read_word();
                    
                    params.push((param_name, PermissionedType {
                        base_type: Type::from_str(&param_type)?,
                        permissions,
                    }));
                }
                self.expect(')')?;
                
                // Handle return type
                self.skip_whitespace();
                let return_type = if self.peek_char() == '-' && self.peek_next_char() == '>' {
                    self.position += 2; // skip ->
                    self.skip_whitespace();
                    let typ = self.read_word();
                    Some(PermissionedType {
                        base_type: Type::from_str(&typ)?,
                        permissions: vec![],
                    })
                } else {
                    None
                };
                
                // Parse behavior body
                self.skip_whitespace();
                self.expect('{')?;
                let mut body = Vec::new();
                
                while self.peek_char() != '}' {
                    self.skip_whitespace();
                    if self.peek_word() == "return" {
                        self.read_word(); // consume "return"
                        self.skip_whitespace();
                        let value = self.parse_expression()?;
                        body.push(Statement::Return(value));
                    } else {
                        body.push(self.parse_statement()?);
                    }
                }
                self.expect('}')?;
                
                Ok(Statement::Function {
                    name,
                    params,
                    body,
                    return_type,
                    is_behavior: true,  // Behavior
                })
            },
            "reads" | "read" | "write" => {
                self.skip_whitespace();
                let mut permissions = match word.as_str() {
                    "reads" => vec![Permission::Reads],
                    "read" => vec![Permission::Read],
                    "write" => vec![Permission::Write],
                    _ => unreachable!(),
                };

                if word != "write" {
                    self.skip_whitespace();
                    if self.peek_word() == "write" {
                        self.read_word(); // consume "write"
                        permissions.push(Permission::Write);
                    } else if self.peek_word() == "writes" {
                        self.read_word(); // consume "writes"
                        permissions.push(Permission::Writes);
                    }
                }

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
                self.skip_whitespace();
                let target = word;

                match self.peek_char() {
                    '+' if self.peek_next_char() == '=' => {
                        self.position += 2; // skip +=
                        self.skip_whitespace();
                        let value = self.parse_expression()?;

                        Ok(Statement::Assignment {
                            target: target.clone(),
                            value: Expression::Binary {
                                left: Box::new(Expression::Variable(target)),
                                operator: TokenType::Plus,
                                right: Box::new(value),
                            },
                            target_type: PermissionedType::new(Type::I64, vec![Permission::Write]),
                        })
                    },
                    '=' => {
                        self.position += 1; // skip =
                        self.skip_whitespace();
                        let value = self.parse_expression()?;

                        Ok(Statement::Assignment {
                            target,
                            value,
                            target_type: PermissionedType::new(Type::I64, vec![Permission::Write]),
                        })
                    },
                    other => Err(format!("Expected '=' or '+=', found '{}'", other)),
                }
            }
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.skip_whitespace();
        let left = self.parse_term()?;

        self.skip_whitespace();
        match self.peek_char() {
            '+' => {
                self.position += 1;
                self.skip_whitespace();
                let right = self.parse_term()?;
                Ok(Expression::Binary {
                    left: Box::new(left),
                    operator: TokenType::Plus,
                    right: Box::new(right),
                })
            },
            _ => Ok(left),
        }
    }

    fn parse_term(&mut self) -> Result<Expression, String> {
        self.skip_whitespace();

        if let Some(digit) = self.peek_char().to_digit(10) {
            let mut number = 0;
            while let Some(d) = self.peek_char().to_digit(10) {
                number = number * 10 + d as i64;
                self.position += 1;
            }
            Ok(Expression::Number(number))
        } else if self.peek_char().is_alphabetic() {
            let word = self.read_word();
            
            self.skip_whitespace();
            match self.peek_char() {
                '(' => {
                    // Function call
                    self.position += 1; // skip (
                    let mut args = Vec::new();
                    
                    while self.peek_char() != ')' {
                        if !args.is_empty() {
                            self.expect(',')?;
                            self.skip_whitespace();
                        }
                        
                        args.push(self.parse_expression()?);
                    }
                    self.expect(')')?;
                    
                    Ok(Expression::Call {
                        function: word,
                        arguments: args,
                    })
                },
                _ => {
                    if word == "peak" {
                        self.skip_whitespace();
                        let var_name = self.read_word();
                        Ok(Expression::Peak(Box::new(Expression::Variable(var_name))))
                    } else {
                        Ok(Expression::Variable(word))
                    }
                }
            }
        } else {
            Err(format!("Invalid expression at line {}, column {}", self.line, self.column))
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        self.skip_whitespace();
        if self.peek_char() == expected {
            self.position += 1;
            Ok(())
        } else {
            Err(format!("Expected '{}', found '{}'", expected, self.peek_char()))
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

    fn parse_permissions(&mut self) -> Result<Vec<Permission>, String> {
        let mut permissions = Vec::new();
        let word = self.peek_word();
        
        match word.as_str() {
            "reads" => {
                self.read_word();
                permissions.push(Permission::Reads);
            },
            "read" => {
                self.read_word();
                permissions.push(Permission::Read);
            },
            "write" => {
                self.read_word();
                permissions.push(Permission::Write);
            },
            "writes" => {
                self.read_word();
                permissions.push(Permission::Writes);
            },
            _ => {}
        }
        
        Ok(permissions)
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