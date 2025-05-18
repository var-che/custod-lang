use crate::token::{Token, TokenType};
use crate::ast::{Expression, FunctionBuilder, Statement};
use crate::types::{Type, Permission, PermissionedType};
use crate::symbol_table::{ResolutionError, Span, Symbol, SymbolKind, SymbolTable};
use crate::error::{ParseError, CompileError};
use crate::type_inference::{TypeInferer, TypeInferenceExt};
use crate::type_checker::{TypeChecker}; // Add this import
use std::collections::HashMap;

// Define a new Result type alias for parser operations
pub type ParseResult<T> = Result<T, ParseError>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    symbol_table: SymbolTable,
    token_locations: HashMap<usize, Span>,
    errors: Vec<CompileError>, // Track errors separately from symbol table
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut token_locations = HashMap::new();
        
        // Store token locations for error reporting
        for (i, _token) in tokens.iter().enumerate() {
            // For now we use default positions until Token has proper line/column tracking
            token_locations.insert(i, Span::point(0, 0));
        }
         
        Parser {
            tokens,
            current: 0,
            symbol_table: SymbolTable::new(),
            token_locations,
            errors: Vec::new(),
        }
    }
    
    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }
    
    // New method to get all compile errors
    pub fn get_errors(&self) -> Vec<CompileError> {
        let mut all_errors = self.errors.clone();
        
        // Also include symbol table errors - properly clone each error
        for error in self.symbol_table.get_errors() {
            all_errors.push(CompileError::Resolution(error.clone())); 
        }
        
        all_errors
    }
    
    // Add a convenience constructor that uses the lexer
    pub fn from_source(source: &str) -> Self {
        use crate::lexer::Lexer;
        
        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.scan_tokens();
        
        // Create token locations with accurate positions from token data
        let mut token_locations = HashMap::new();
        
        for (i, token) in tokens.iter().enumerate() {
            token_locations.insert(i, Span::new(
                token.line,
                token.column,
                token.line,
                token.column + token.length - 1
            ));
        }
        
        Parser {
            tokens,
            current: 0,
            symbol_table: SymbolTable::new(),
            token_locations,
            errors: Vec::new(),
        }
    }
    
    // Move these position tracking methods to a new SourcePosition trait or struct
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
    
    fn match_any(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }
    
    fn match_token_type(&mut self, _expected: &TokenType) -> bool {
        // Special match for Identifier and Number - we match the type but not the exact value
        match (&self.peek().token_type, _expected) {
            (TokenType::Identifier(_), TokenType::Identifier(_)) => {
                self.advance();
                true
            },
            (TokenType::Number(_), TokenType::Number(_)) => {
                self.advance();
                true
            },
            _ => self.match_token(_expected),
        }
    }
    
    // Update consume to return ParseResult instead of Result<&Token, String>
    fn consume(&mut self, token_type: &TokenType, message: &str) -> ParseResult<&Token> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            let span = self.current_span();
            Err(ParseError::unexpected_token(
                span,
                format!("{}: expected {:?}, found {:?}", 
                       message, 
                       token_type, 
                       self.peek().token_type)
            ))
        }
    }
    
    // Helper method to get the current span
    fn current_span(&self) -> Span {
        if let Some(span) = self.token_locations.get(&self.current) {
            span.clone()
        } else {
            // Default span if not found
            Span::point(0, 0)
        }
    }
    
    // Update parse_expression to return ParseResult
    pub fn parse_expression(&mut self) -> ParseResult<Expression> {
        // First, log what we're trying to parse
        println!("Parsing expression, current token: {:?}", self.peek().token_type);
        
        // Delegate to comparison which handles operators via parse_addition, etc.
        self.parse_comparison()
    }

    // Update all parsing methods to use ParseResult
    fn parse_addition(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_multiplication()?;

        while self.match_token(&TokenType::Plus) || self.match_token(&TokenType::Minus) {
            let operator = self.previous().token_type.clone();
            let right = self.parse_multiplication()?;
            
            left = Expression::new_binary(left, operator, right);
        }

        Ok(left)
    }

    fn parse_multiplication(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_primary()?;

        // Handle * and / operators (higher precedence)
        while self.match_token(&TokenType::Star) || self.match_token(&TokenType::Slash) {
            println!("Found multiplication/division operator: {:?}", self.previous().token_type);
            let operator = self.previous().token_type.clone();
            
            // Print token for debugging
            println!("Parsing right side of operation");
            
            let right = self.parse_primary()?;
            
            println!("Creating binary expression: {:?} {:?} {:?}", left, operator, right);
            
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> ParseResult<Expression> {
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

    fn parse_primary(&mut self) -> ParseResult<Expression> {
        // Handle different primary expression types
        if self.match_token_type(&TokenType::Number(0)) { // The value doesn't matter here
            let value = match self.previous().token_type {
                TokenType::Number(val) => val,
                _ => unreachable!(),
            };
            return Ok(Expression::Number(value));
        }
        
        // Handle grouping with parentheses
        if self.match_token(&TokenType::LeftParen) {
            println!("Parsing grouped expression");
            let expr = self.parse_expression()?;
            self.consume(&TokenType::RightParen, "Expected ')' after expression")?;
            return Ok(expr); // Return the inner expression directly
        }
        
        // Handle peak operator
        if self.match_token(&TokenType::Peak) {
            let expr = self.parse_primary()?;
            return Ok(Expression::Peak(Box::new(expr)));
        }
        
        // Handle clone operator
        if self.match_token(&TokenType::Clone) {
            let expr = self.parse_primary()?;
            return Ok(Expression::Clone(Box::new(expr)));
        }
        
        // Handle variable references
        if self.match_token_type(&TokenType::Identifier("".to_string())) {
            let name = match self.previous().token_type {
                TokenType::Identifier(ref name) => name.clone(),
                _ => unreachable!(),
            };
            
            // Create a span for this variable reference
            let token = self.previous();
            let span = Span::new(
                token.line, 
                token.column,
                token.line,
                token.column + token.length - 1
            );
            
            // Allow identifiers even if they're not in the symbol table yet
            // (particularly for function parameters which might be referenced before they're added)
            let _ = self.symbol_table.resolve(&name, span);
            
            // Return the variable reference expression
            return Ok(Expression::Variable(name));
        }
        
        // Other primary expression types...
        
        Err(ParseError::unexpected_token(
            self.current_span(),
            format!("Expected expression, found {:?}", self.peek().token_type)
        ))
    }

    // Improve error handling in parse_statement
    pub fn parse_statement(&mut self) -> ParseResult<Statement> {
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
                // This could be an assignment, function call, or a standalone expression
                let start_pos = self.current;
                let name = self.get_identifier_name()?;
                
                if self.match_token(&TokenType::Equal) {
                    // Check symbol table first for permission
                    let token = self.previous();
                    let span = Span::new(
                        token.line,
                        token.column,
                        token.line,
                        token.column + token.lexeme.len()
                    );
                    
                    // Instead of returning the error directly, record it and continue
                    if let Err(err) = self.symbol_table.check_assignment(&name, span.clone()) {
                        // Add the error to the symbol table's error list
                        match err {
                            ResolutionError::ImmutableAssignment { name, span, declaration_span } => {
                                self.symbol_table.add_error(ResolutionError::ImmutableAssignment {
                                    name,
                                    span,
                                    declaration_span
                                });
                            },
                            _ => {
                                // Handle other error types
                                self.symbol_table.add_error(err);
                            }
                        }
                        
                        // Continue parsing to handle error recovery
                        let right = self.parse_expression()?;
                        let target_type = match self.symbol_table.resolve(&name, span.clone()) {
                            Some(symbol) => symbol.typ.clone(),
                            None => PermissionedType::new(Type::Int, vec![])
                        };
                        return Ok(Statement::new_assignment(name, right, target_type));
                    }
                    
                    let right = self.parse_expression()?;
                    let target_type = match self.symbol_table.resolve(&name, span.clone()) {
                        Some(symbol) => symbol.typ.clone(),
                        None => PermissionedType::new(Type::Int, vec![])
                    };
                    return Ok(Statement::new_assignment(name, right, target_type));
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
                    // This is a standalone identifier, which could be part of an expression
                    // Reset position and try parsing as an expression
                    self.current = start_pos;
                    let expr = self.parse_expression()?;
                    Ok(Statement::Expression(expr))
                }
            },
            TokenType::LeftBrace => {
                self.parse_block()
            },
            _ => {
                // Try to parse as an expression statement
                println!("Attempting to parse expression statement with token: {:?}", self.peek().token_type);
                let expr = self.parse_expression()?;
                Ok(Statement::Expression(expr))
            },
        }
    }

    // Helper method to get identifier name
    fn get_identifier_name(&mut self) -> ParseResult<String> {
        if let TokenType::Identifier(ref name) = self.peek().token_type.clone() {
            self.advance(); // Consume the identifier
            Ok(name.clone())
        } else {
            Err(ParseError::unexpected_token(
                self.current_span(),
                format!("Expected identifier, found {:?}", self.peek().token_type)
            ))
        }
    }

    fn parse_variable_declaration(&mut self) -> ParseResult<Statement> {
        // Store the first token position
        let start_token_pos = self.current;
        
        // Check for permission modifiers
        let mut permissions = Vec::new();
        
        // Loop to handle multiple permissions (read, write, reads, writes)
        while self.match_any(&[
            TokenType::Read, 
            TokenType::Write, 
            TokenType::Reads, 
            TokenType::Writes
        ]) {
            match self.previous().token_type {
                TokenType::Read => permissions.push(Permission::Read),
                TokenType::Write => permissions.push(Permission::Write),
                TokenType::Reads => permissions.push(Permission::Reads),
                TokenType::Writes => permissions.push(Permission::Writes),
                _ => {}
            }
        }
        
        // Get variable name and create span for it
        let name_token_pos = self.current; // Position before consuming the identifier
        let name = self.get_identifier_name()?;
        
        // Create span using the token's position data
        let token = &self.tokens[name_token_pos];
        let name_span = Span::new(
            token.line,
            token.column,
            token.line,
            token.column + token.length - 1
        );
        
        // Check for type annotation (optional)
        let typ = if self.match_token(&TokenType::Colon) {
            // Use the new parse_type function
            let base_type = self.parse_type()?;
            PermissionedType::new(base_type, permissions)
        } else {
            // If no type annotation, infer from the initializer if possible
            self.consume(&TokenType::Equal, "Expected '=' after variable name")?;
            let initializer_expr = self.parse_expression()?;
            
            // Create a type inferer to infer the type
            let mut inferer = TypeInferer::new(&mut self.symbol_table);
            let inferred_type = initializer_expr.infer_type(&mut inferer);
            
            // Return to the parser with the inferred type and prepare to continue
            let typ = PermissionedType::new(inferred_type, permissions);
            
            // Create the declaration statement with the inferred type
            let declaration = Statement::new_declaration(name.clone(), typ.clone(), Some(initializer_expr));
            
            // Define the symbol with the accurate span and inferred type
            self.symbol_table.define(Symbol {
                name,
                typ: typ.clone(),
                kind: SymbolKind::Variable,
                span: name_span,
            });
            
            return Ok(declaration);
        };
        
        // Expect assignment with initializer
        self.consume(&TokenType::Equal, "Expected '=' after variable name")?;
        
        let initializer_expr = self.parse_expression()?;
        
        // Check permission compatibility if initializer is a variable
        if let Expression::Variable(ref source_name) = initializer_expr {
            // Create span for the expression
            let expr_span = Span::new(
                self.previous().line,
                self.previous().column,
                self.previous().line,
                self.previous().column + self.previous().length - 1
            );
            
            // Check permission compatibility
            if let Err(err) = self.symbol_table.check_permission_compatibility(source_name, &typ.permissions, expr_span) {
                self.symbol_table.add_error(err);
            }
        }
        
        // Don't check permission errors when using peak operator
        // This allows read c = peak counter to work
        if let Expression::Peak(_) = initializer_expr {
            // Peak expressions bypass normal permission checking
        }
        
        // Create declaration statement
        let declaration = Statement::new_declaration(name.clone(), typ.clone(), Some(initializer_expr));
        
        // Define the symbol with the accurate span
        self.symbol_table.define(Symbol {
            name,
            typ,
            kind: SymbolKind::Variable,
            span: name_span,
        });
        
        Ok(declaration)
    }

    fn parse_block(&mut self) -> ParseResult<Statement> {
        self.consume(&TokenType::LeftBrace, "Expected '{'")?;
        
        let mut statements = Vec::new();
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        self.consume(&TokenType::RightBrace, "Expected '}' after block")?;
        
        Ok(Statement::Block(statements))
    }

    fn parse_function_declaration(&mut self, is_behavior: bool) -> ParseResult<Statement> {
        println!("Starting to parse a function declaration, is_behavior={}", is_behavior);
        
        // Store the function start position for error reporting
        let function_start_pos = self.current;
        
        self.advance(); // Consume 'fn' or 'on'
        
        let name = self.get_identifier_name()?;
        println!("Parsing function with name: {}", name);
        
        self.consume(&TokenType::LeftParen, "Expected '(' after function name")?;
        println!("Found opening parenthesis");
        
        // Parse parameters
        let mut parameters = Vec::new();
        
        if !self.check(&TokenType::RightParen) {
            println!("Parsing parameters");
            loop {
                // Parse parameter permissions
                let mut permissions = Vec::new();
                
                // Check for permission keywords
                match self.peek().token_type {
                    TokenType::Reads => {
                        self.advance();
                        permissions.push(Permission::Reads);
                        println!("Found Reads permission");
                    },
                    TokenType::Writes => {
                        self.advance();
                        permissions.push(Permission::Writes);
                        println!("Found Writes permission");
                    },
                    TokenType::Read => {
                        self.advance();
                        permissions.push(Permission::Read);
                        println!("Found Read permission");
                    },
                    TokenType::Write => {
                        self.advance();
                        permissions.push(Permission::Write);
                        println!("Found Write permission");
                    },
                    _ => {
                        println!("No permission specified for parameter");
                    }
                }
                
                // Check for additional permission
                match self.peek().token_type {
                    TokenType::Write => {
                        self.advance();
                        permissions.push(Permission::Write);
                        println!("Found additional Write permission");
                    },
                    TokenType::Writes => {
                        self.advance();
                        permissions.push(Permission::Writes);
                        println!("Found additional Writes permission");
                    },
                    _ => {}
                }
                
                // Get parameter name
                let param_name = self.get_identifier_name()?;
                println!("Parameter name: {}", param_name);
                
                // Parse parameter type
                let param_type = if self.match_token(&TokenType::Colon) {
                    println!("Found colon, parsing parameter type");
                    // Use the parse_type function instead of checking for specific types
                    match self.parse_type() {
                        Ok(base_type) => {
                            println!("Parameter type: {:?}", base_type);
                            PermissionedType::new(base_type, permissions.clone())
                        },
                        Err(err) => {
                            println!("Error parsing parameter type: {:?}", err);
                            return Err(ParseError::unexpected_token(
                                self.current_span(),
                                "Expected type after ':'".to_string()
                            ));
                        }
                    }
                } else {
                    println!("No type specified, using default Int");
                    // Default to Int if no type specified
                    PermissionedType::new(Type::Int, permissions.clone())
                };
                
                // Add the parameter to our list
                parameters.push((param_name.clone(), param_type));
                println!("Added parameter {} to function", param_name);
                
                if !self.match_token(&TokenType::Comma) {
                    println!("No more parameters");
                    break;
                }
                println!("Found comma, parsing next parameter");
            }
        } else {
            println!("No parameters to parse");
        }
        
        println!("Expecting right parenthesis");
        self.consume(&TokenType::RightParen, "Expected ')' after parameters")?;
        println!("Found closing parenthesis");
        
        // Update return type parsing in parse_function_declaration
        let return_type = if self.match_token(&TokenType::Arrow) {
            println!("Found return type arrow ->");
            // Use parse_type instead of checking for specific types
            match self.parse_type() {
                Ok(base_type) => {
                    println!("Return type: {:?}", base_type);
                    Some(PermissionedType::new(base_type, vec![]))
                },
                Err(err) => {
                    println!("Error parsing return type: {:?}", err);
                    return Err(ParseError::unexpected_token(
                        self.current_span(),
                        "Expected return type after '->'".to_string()
                    ));
                }
            }
        } else {
            println!("No return type specified");
            None
        };
        
        println!("Parsing function body");
        // Parse function body
        let body_stmt = self.parse_block()?;
        println!("Parsed function body block");
        
        // Extract statements from body block
        let body = match body_stmt {
            Statement::Block(statements) => {
                // If there's no explicit return statement and the body isn't empty,
                // add an implicit return for the last expression
                println!("Function body has {} statements", statements.len());
                
                if !statements.is_empty() {
                    let mut modified_statements = statements.clone();
                    
                    // Check if the last statement can be treated as an implicit return
                    if let Some(last) = modified_statements.last() {
                        match last {
                            // If the last statement is already a return, don't modify
                            Statement::Return(_) => {
                                println!("Last statement is already a return");
                            },
                            
                            // If it's an expression, convert it to a return statement
                            Statement::Expression(expr) => {
                                println!("Converting expression to return: {:?}", expr);
                                let last_idx = modified_statements.len() - 1;
                                modified_statements[last_idx] = Statement::Return(expr.clone());
                            },
                            
                            // For other types, we don't create an implicit return
                            _ => {
                                println!("Last statement is not an expression, not creating return");
                            }
                        }
                    }
                    
                    modified_statements
                } else {
                    println!("Function body is empty");
                    statements
                }
            },
            _ => return Err(ParseError::unexpected_token(
                self.current_span(),
                "Expected block for function body".to_string()
            ))
        };
        
        println!("Creating function {} with {} params and {} body statements", 
                 name, parameters.len(), body.len());
        
        // Register parameters in symbol table - add this for future validations
        for (param_name, param_type) in &parameters {
            let span = Span::point(0, 0); // Default span for now
            self.symbol_table.define(Symbol {
                name: param_name.clone(),
                typ: param_type.clone(),
                kind: SymbolKind::Parameter,
                span,
            });
        }
        
        // Create function using builder - pass parameters correctly
        let mut builder = FunctionBuilder::new(name)
            .as_behavior(is_behavior)
            .with_return_type(return_type.clone())
            .with_body(body.clone());
        
        // Explicitly add each parameter to the builder
        for (name, typ) in parameters {
            builder = builder.with_parameter(name, typ);
        }
        
        let function = builder.build();
        
        // Type check the function
        let function_span = if let Some(span) = self.token_locations.get(&function_start_pos) {
            span.clone()
        } else {
            Span::point(0, 0)
        };
        
        // Check return type compatibility - now with immutable reference
        let type_checker = TypeChecker::new(&self.symbol_table);
        let type_errors = type_checker.check_function(&function, function_span);
        
        // Add any type errors to our errors list
        for error in type_errors {
            self.symbol_table.add_error(error);
        }
        
        println!("Successfully built function statement");
        Ok(function)
    }

    // Update parse_statements to collect errors instead of printing them
    pub fn parse_statements(&mut self) -> Vec<Statement> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            println!("Parsing statement, current token: {:?}", self.peek().token_type);
            match self.parse_statement() {
                Ok(stmt) => {
                    println!("Successfully parsed statement: {:?}", stmt);
                    statements.push(stmt);
                },
                Err(err) => {
                    println!("Error parsing statement: {:?}", err);
                    // Record the error instead of printing it
                    self.errors.push(CompileError::Parse(err));
                    self.synchronize();
                }
            }
        }
        
        println!("Finished parsing statements, found {}", statements.len());
        statements
    }

    // Fix the synchronize method:
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
                TokenType::Fn |
                TokenType::On |
                TokenType::Return |
                TokenType::Print => return,
                _ => {}
            }
            
            self.advance();
        }
    }

    fn parse_type(&mut self) -> ParseResult<Type> {
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
            _ => Err(ParseError::unexpected_token(
                self.current_span(),
                format!("Expected type name, got {:?}", self.peek().token_type)
            )),
        }
    }

    // Add a debug function to test our expression parsing directly
    pub fn test_parse_expression(&mut self, expr_str: &str) -> ParseResult<Expression> {
        // Create a temporary parser with just this expression
        let mut temp_parser = Parser::from_source(expr_str);
        
        // Parse and return the expression
        temp_parser.parse_expression()
    }
}
