use crate::token::{Token, TokenType};
use crate::ast::{Expression, FunctionBuilder, Statement};
use crate::types::{Type, Permission, PermissionedType};
use crate::symbol_table::{Location, Symbol, SymbolKind, SymbolTable};
use std::collections::HashMap;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    symbol_table: SymbolTable,  // Add this field
    token_locations: HashMap<usize, Location>, // Track token locations
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut token_locations = HashMap::new();
        
        // Store token locations for error reporting
        for (i, token) in tokens.iter().enumerate() {
            // Assuming Token has lexeme field that contains the original token text
            // We'll use default positions until Token has proper line/column tracking
            token_locations.insert(i, Location {
                line: 0,   // Default line number
                column: 0, // Default column number
            });
        }
         
        Parser {
            tokens,
            current: 0,
            symbol_table: SymbolTable::new(),
            token_locations,
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
    
    fn match_any(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
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
        self.parse_comparison() // Call the comparison parser instead of directly calling addition
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

    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_addition()?;
        
        while self.match_any(&[
            TokenType::Greater, TokenType::GreaterEqual,
            TokenType::Less, TokenType::LessEqual,
            TokenType::EqualEqual, TokenType::BangEqual,
        ]) {
            let operator = self.previous().token_type.clone();
            let right = self.parse_addition()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
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
                    // Check symbol table first
                    let location = Location {
                        line: 0, // Default line since Token doesn't have line field
                        column: 0, // Default column since Token doesn't have column field
                    };
                    
                    if let Err(err) = self.symbol_table.check_assignment(&name, location) {
                        return Err(err.to_string());
                    }
                    
                    // Assignment: name = expr
                    let value = self.parse_expression()?;
                    Ok(Statement::new_assignment(
                        name,
                        value,
                        PermissionedType::new(Type::Int, vec![Permission::Write]),
                    ))
                } else if self.match_token(&TokenType::LeftParen) {
                    // Function call handling
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
                    
                    Ok(Statement::Expression(Expression::Call {
                        function: name,
                        arguments,
                    }))
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
            // Use the new parse_type function
            let base_type = self.parse_type()?;
            PermissionedType::new(base_type, permissions)
        } else {
            // Default to Int if no type specified
            PermissionedType::new(Type::Int, permissions)
        };
        
        // Expect assignment with initializer
        self.consume(&TokenType::Equal, "Expected '=' after variable name")?;
        
        let initializer = self.parse_expression()?;
        
        // Get current position for error reporting
        let location = Location {
            line: 0, // Default line since Token doesn't have line field
            column: 0, // Default column since Token doesn't have column field
        };
        
        // Create declaration statement
        let declaration = Statement::new_declaration(name.clone(), typ.clone(), Some(initializer));
        
        // Add to symbol table
        self.symbol_table.define(Symbol {
            name,
            typ,
            kind: SymbolKind::Variable,
            location,
        });
        
        Ok(declaration)
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
                    // Use the parse_type function instead of checking for specific types
                    match self.parse_type() {
                        Ok(base_type) => PermissionedType::new(base_type, permissions),
                        Err(_) => return Err("Expected type after ':'".to_string())
                    }
                } else {
                    // Default to Int if no type specified
                    PermissionedType::new(Type::Int, permissions)
                };
                
                parameters.push((param_name, param_type));
                
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }
        
        self.consume(&TokenType::RightParen, "Expected ')' after parameters")?;
        
        // Update return type parsing in parse_function_declaration
        let return_type = if self.match_token(&TokenType::Arrow) {
            // Use parse_type instead of checking for specific types
            match self.parse_type() {
                Ok(base_type) => Some(PermissionedType::new(base_type, vec![])),
                Err(_) => return Err("Expected return type after '->'".to_string())
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
                    // Print current token for more context
                    eprintln!("Current token: {:?}", self.peek().token_type);
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

    fn parse_type(&mut self) -> Result<Type, String> {
        match self.peek().token_type {
            TokenType::TypeInt => {
                self.advance();
                Ok(Type::Int)
            },
            TokenType::TypeInt8 => {
                self.advance();
                Ok(Type::Int8)
            },
            TokenType::TypeInt16 => {
                self.advance();
                Ok(Type::Int16)
            },
            TokenType::TypeInt32 => {
                self.advance();
                Ok(Type::Int32)
            },
            TokenType::TypeInt64 => {
                self.advance();
                Ok(Type::Int64)
            },
            TokenType::TypeUInt => {
                self.advance();
                Ok(Type::UInt)
            },
            TokenType::TypeUInt8 => {
                self.advance();
                Ok(Type::UInt8)
            },
            TokenType::TypeUInt16 => {
                self.advance();
                Ok(Type::UInt16)
            },
            TokenType::TypeUInt32 => {
                self.advance();
                Ok(Type::UInt32)
            },
            TokenType::TypeUInt64 => {
                self.advance();
                Ok(Type::UInt64)
            },
            TokenType::TypeFloat => {
                self.advance();
                Ok(Type::Float)
            },
            TokenType::TypeFloat32 => {
                self.advance();
                Ok(Type::Float32)
            },
            TokenType::TypeFloat64 => {
                self.advance();
                Ok(Type::Float64)
            },
            TokenType::TypeBool => {
                self.advance();
                Ok(Type::Bool)
            },
            TokenType::TypeString => {
                self.advance();
                Ok(Type::String)
            },
            TokenType::TypeUnit => {
                self.advance();
                Ok(Type::Unit)
            },
            _ => Err(format!("Expected type name, got {:?}", self.peek().token_type)),
        }
    }
}