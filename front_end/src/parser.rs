use crate::token::{Token, TokenType};
use crate::ast::{Expression, FunctionBuilder, Statement};
use crate::types::{Type, Permission, PermissionedType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
        }
    }
    
    // Add a convenience constructor that uses the lexer
    pub fn from_source(source: &str) -> Self {
        use crate::lexer::Lexer;
        
        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.scan_tokens();
        Self::new(tokens)
    }
    
    // Replace existing methods for working with characters
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
    
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }
    
    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() { 
            false 
        } else {
            &self.peek().token_type == token_type
        }
    }
    
    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<&Token, String> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(format!("{}: expected {:?}, found {:?}", 
                       message, 
                       token_type, 
                       self.peek().token_type))
        }
    }
    
    pub fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_addition()
    }

    fn parse_addition(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_multiplication()?;

        while self.match_token(&TokenType::Plus) || self.match_token(&TokenType::Minus) {
            let operator = self.previous().token_type.clone();
            let right = self.parse_multiplication()?;
            
            left = Expression::new_binary(left, operator, right);
        }

        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_primary()?;

        // Handle * and / operators (higher precedence)
        while self.match_token(&TokenType::Star) || self.match_token(&TokenType::Slash) {
            let operator = self.previous().token_type.clone();
            let right = self.parse_primary()?;
            
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        // Handle peak keyword
        if self.match_token(&TokenType::Peak) {
            // After 'peak', we expect a variable name
            if let TokenType::Identifier(ref name) = self.peek().token_type.clone() {
                let name = name.clone();
                self.advance(); // Consume the identifier
                return Ok(Expression::Peak(Box::new(Expression::Variable(name))));
            } else {
                return Err("Expected variable name after 'peak'".to_string());
            }
        }
        
        // Handle number literals
        if let TokenType::Number(value) = self.peek().token_type {
            self.advance(); // Consume the number token
            return Ok(Expression::Number(value));
        }
        
        // Handle identifiers (variables and function calls)
        if let TokenType::Identifier(ref name) = self.peek().token_type.clone() {
            let name = name.clone();
            self.advance(); // Consume the identifier
            
            // Check if this is a function call with arguments
            if self.match_token(&TokenType::LeftParen) {
                let mut arguments = Vec::new();
                
                // Parse arguments list if not empty
                if !self.check(&TokenType::RightParen) {
                    loop {
                        arguments.push(self.parse_expression()?);
                        
                        if !self.match_token(&TokenType::Comma) {
                            break;
                        }
                    }
                }
                
                self.consume(&TokenType::RightParen, "Expected ')' after function arguments")?;
                
                return Ok(Expression::Call {
                    function: name,
                    arguments,
                });
            }
            
            // Regular variable reference
            return Ok(Expression::Variable(name));
        }
        
        // Handle grouped expressions: ( expr )
        if self.match_token(&TokenType::LeftParen) {
            let expr = self.parse_expression()?;
            self.consume(&TokenType::RightParen, "Expected ')' after expression")?;
            return Ok(expr);
        }
        
        Err(format!("Unexpected token: {:?}", self.peek().token_type))
    }

    pub fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.peek().token_type {
            TokenType::Reads | TokenType::Read | TokenType::Write | TokenType::Writes => {
                self.parse_variable_declaration()
            },
            TokenType::Fn => {
                self.parse_function_declaration(false)
            },
            TokenType::On => {
                self.parse_function_declaration(true) // behavior
            },
            TokenType::Return => {
                self.advance(); // consume 'return'
                let value = self.parse_expression()?;
                Ok(Statement::new_return(value))
            },
            TokenType::Print => {
                self.advance(); // consume 'print'
                let expr = self.parse_expression()?;
                Ok(Statement::new_print(expr))
            },
            TokenType::Identifier(_) => {
                // This could be an assignment or function call
                let name = self.get_identifier_name()?;
                
                if self.match_token(&TokenType::Equal) {
                    // Assignment: name = expr
                    let value = self.parse_expression()?;
                    Ok(Statement::new_assignment(
                        name,
                        value,
                        PermissionedType::new(Type::I64, vec![Permission::Write]),
                    ))
                } else if self.match_token(&TokenType::LeftParen) {
                    // Function call: name(args...)
                    let mut arguments = Vec::new();
                    
                    // Parse arguments if any
                    if !self.check(&TokenType::RightParen) {
                        loop {
                            arguments.push(self.parse_expression()?);
                            if !self.match_token(&TokenType::Comma) {
                                break;
                            }
                        }
                    }
                    
                    self.consume(&TokenType::RightParen, "Expected ')' after function arguments")?;
                    
                    Ok(Statement::new_expression(Expression::new_call(name, arguments)))
                } else {
                    Err(format!("Expected '=' or '(' after identifier, found {:?}", self.peek().token_type))
                }
            },
            TokenType::LeftBrace => {
                self.parse_block()
            },
            _ => {
                // Try to parse as an expression statement
                let expr = self.parse_expression()?;
                Ok(Statement::Expression(expr))
            },
        }
    }

    // Helper method to get identifier name
    fn get_identifier_name(&mut self) -> Result<String, String> {
        if let TokenType::Identifier(ref name) = self.peek().token_type.clone() {
            self.advance(); // Consume the identifier
            Ok(name.clone())
        } else {
            Err(format!("Expected identifier, found {:?}", self.peek().token_type))
        }
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement, String> {
        // Determine the permission type
        let permission = match self.peek().token_type {
            TokenType::Reads => {
                self.advance(); // Consume 'reads'
                Permission::Reads
            },
            TokenType::Read => {
                self.advance(); // Consume 'read'
                Permission::Read
            },
            TokenType::Write => {
                self.advance(); // Consume 'write'
                Permission::Write
            },
            TokenType::Writes => {
                self.advance(); // Consume 'writes'
                Permission::Writes
            },
            _ => return Err("Expected permission keyword".to_string())
        };
        
        let mut permissions = vec![permission];
        
        // Check for additional permission
        match self.peek().token_type {
            TokenType::Write => {
                self.advance(); // Consume 'write'
                permissions.push(Permission::Write);
            },
            TokenType::Read => {
                self.advance(); // Consume 'read'
                permissions.push(Permission::Read);
            },
            _ => {} // No additional permission
        }
        
        // Get variable name
        let name = self.get_identifier_name()?;
        
        // Check for type annotation (optional)
        let typ = if self.match_token(&TokenType::Colon) {
            if self.match_token(&TokenType::TypeI64) {
                PermissionedType::new(Type::I64, permissions)
            } else {
                return Err("Expected type after ':'".to_string());
            }
        } else {
            // Default to i64 if no type specified
            PermissionedType::new(Type::I64, permissions)
        };
        
        // Expect assignment with initializer
        self.consume(&TokenType::Equal, "Expected '=' after variable name")?;
        
        let initializer = self.parse_expression()?;
        
        Ok(Statement::new_declaration(name, typ, Some(initializer)))
    }

    fn parse_block(&mut self) -> Result<Statement, String> {
        self.consume(&TokenType::LeftBrace, "Expected '{'")?;
        
        let mut statements = Vec::new();
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        self.consume(&TokenType::RightBrace, "Expected '}' after block")?;
        
        Ok(Statement::Block(statements))
    }

    fn parse_function_declaration(&mut self, is_behavior: bool) -> Result<Statement, String> {
        self.advance(); // Consume 'fn' or 'on'
        
        let name = self.get_identifier_name()?;
        
        self.consume(&TokenType::LeftParen, "Expected '(' after function name")?;
        
        // Parse parameters
        let mut parameters = Vec::new();
        
        if !self.check(&TokenType::RightParen) {
            loop {
                // Parse parameter permissions
                let mut permissions = Vec::new();
                
                // Check for permission keywords
                match self.peek().token_type {
                    TokenType::Reads => {
                        self.advance();
                        permissions.push(Permission::Reads);
                    },
                    TokenType::Writes => {
                        self.advance();
                        permissions.push(Permission::Writes);
                    },
                    TokenType::Read => {
                        self.advance();
                        permissions.push(Permission::Read);
                    },
                    TokenType::Write => {
                        self.advance();
                        permissions.push(Permission::Write);
                    },
                    _ => {} // No permissions specified
                }
                
                // Check for additional permission
                match self.peek().token_type {
                    TokenType::Write => {
                        self.advance();
                        permissions.push(Permission::Write);
                    },
                    TokenType::Writes => {
                        self.advance();
                        permissions.push(Permission::Writes);
                    },
                    _ => {} // No additional permission
                }
                
                // Get parameter name
                let param_name = self.get_identifier_name()?;
                
                // Parse parameter type
                let param_type = if self.match_token(&TokenType::Colon) {
                    if self.match_token(&TokenType::TypeI64) {
                        PermissionedType::new(Type::I64, permissions)
                    } else {
                        return Err("Expected type after ':'".to_string());
                    }
                } else {
                    // Default to i64 if no type specified
                    PermissionedType::new(Type::I64, permissions)
                };
                
                parameters.push((param_name, param_type));
                
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }
        
        self.consume(&TokenType::RightParen, "Expected ')' after parameters")?;
        
        // Parse return type
        let return_type = if self.match_token(&TokenType::Arrow) {
            if self.match_token(&TokenType::TypeI64) {
                Some(PermissionedType::new(Type::I64, vec![]))
            } else {
                return Err("Expected return type after '->'".to_string());
            }
        } else {
            None
        };
        
        // Parse function body
        let body_stmt = self.parse_block()?;
        
        // Extract statements from body block
        let body = match body_stmt {
            Statement::Block(statements) => statements,
            _ => return Err("Expected block for function body".to_string())
        };
        
        // Create function using builder
        let function = FunctionBuilder::new(name)
            .as_behavior(is_behavior)
            .with_return_type(return_type)
            .with_body(body)
            .build();
        
        Ok(function)
    }

    pub fn parse_statements(&mut self) -> Vec<Statement> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    eprintln!("Error parsing statement: {}", err);
                    // Try to synchronize - skip until something that looks like a statement boundary
                    self.synchronize();
                }
            }
        }
        
        statements
    }

    // Add a synchronization method to recover from errors
    fn synchronize(&mut self) {
        self.advance(); // Skip the token that caused the error
        
        while !self.is_at_end() {
            // If we see a token that could start a new statement, break
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }
            
            match self.peek().token_type {
                // Add token types that could start a statement
                TokenType::Read | 
                TokenType::Reads | 
                TokenType::Write |
                TokenType::Writes |

                _ => {}
            }
            
            self.advance();
        }
    }
}