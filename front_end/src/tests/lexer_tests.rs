use crate::lexer::Lexer;
use crate::token::TokenType;

#[test]
fn test_type_name_tokens() {
    let source = "Int Int8 Int16 Int32 Int64 UInt UInt8 UInt16 UInt32 UInt64 Float Float32 Float64 Bool String";
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    
    // Remove the EOF token
    let tokens = &tokens[0..tokens.len() - 1];
    
    // Check that we have the expected number of tokens
    assert_eq!(tokens.len(), 15, "Should have 15 type tokens");
    
    // Check each token type
    assert_eq!(tokens[0].token_type, TokenType::TypeInt);
    assert_eq!(tokens[1].token_type, TokenType::TypeInt8);
    assert_eq!(tokens[2].token_type, TokenType::TypeInt16);
    assert_eq!(tokens[3].token_type, TokenType::TypeInt32);
    assert_eq!(tokens[4].token_type, TokenType::TypeInt64);
    assert_eq!(tokens[5].token_type, TokenType::TypeUInt);
    assert_eq!(tokens[6].token_type, TokenType::TypeUInt8);
    assert_eq!(tokens[7].token_type, TokenType::TypeUInt16);
    assert_eq!(tokens[8].token_type, TokenType::TypeUInt32);
    assert_eq!(tokens[9].token_type, TokenType::TypeUInt64);
    assert_eq!(tokens[10].token_type, TokenType::TypeFloat);
    assert_eq!(tokens[11].token_type, TokenType::TypeFloat32);
    assert_eq!(tokens[12].token_type, TokenType::TypeFloat64);
    assert_eq!(tokens[13].token_type, TokenType::TypeBool);
    assert_eq!(tokens[14].token_type, TokenType::TypeString);
}

#[test]
fn test_variable_declaration_with_types() {
    let source = "reads x: Int = 42\nwrites y: String = \"hello\"";
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    
    // Check for 'reads' keyword (now implemented as a token type)
    assert_eq!(tokens[0].token_type, TokenType::Reads);
    
    // Check for identifier 'x'
    match &tokens[1].token_type {
        TokenType::Identifier(name) => assert_eq!(name, "x"),
        _ => panic!("Expected identifier token"),
    }
    
    // Check for colon and type
    assert_eq!(tokens[2].token_type, TokenType::Colon);
    assert_eq!(tokens[3].token_type, TokenType::TypeInt);
    
    // Check for equals and number
    assert_eq!(tokens[4].token_type, TokenType::Equal);
    assert_eq!(tokens[5].token_type, TokenType::Number(42));
    
    // Check for 'writes' keyword
    assert_eq!(tokens[6].token_type, TokenType::Writes);
}

#[test]
fn test_function_declaration() {
    let source = "fn calculate(reads a: Int, writes b: Float64) -> Bool {\n  return a > b\n}";
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    println!("{:?}", tokens);
    // Check basic function syntax elements
    assert_eq!(tokens[0].token_type, TokenType::Fn);
    
    // Check function name
    match &tokens[1].token_type {
        TokenType::Identifier(name) => assert_eq!(name, "calculate"),
        _ => panic!("Expected identifier for function name"),
    }
    
    // Check opening parenthesis
    assert_eq!(tokens[2].token_type, TokenType::LeftParen);
    
    // Check first parameter
    assert_eq!(tokens[3].token_type, TokenType::Reads);
    match &tokens[4].token_type {
        TokenType::Identifier(name) => assert_eq!(name, "a"),
        _ => panic!("Expected identifier for parameter name"),
    }
    assert_eq!(tokens[5].token_type, TokenType::Colon);
    assert_eq!(tokens[6].token_type, TokenType::TypeInt);
    assert_eq!(tokens[7].token_type, TokenType::Comma);
    
    // Check second parameter
    assert_eq!(tokens[8].token_type, TokenType::Writes);
    match &tokens[9].token_type {
        TokenType::Identifier(name) => assert_eq!(name, "b"),
        _ => panic!("Expected identifier for parameter name"),
    }
    assert_eq!(tokens[10].token_type, TokenType::Colon);
    assert_eq!(tokens[11].token_type, TokenType::TypeFloat64);
    
    // Check closing parenthesis and return type
    assert_eq!(tokens[12].token_type, TokenType::RightParen);
    assert_eq!(tokens[13].token_type, TokenType::Arrow);
    assert_eq!(tokens[14].token_type, TokenType::TypeBool);
    
    // Check function body opening
    assert_eq!(tokens[15].token_type, TokenType::LeftBrace);
    
    // Check return statement
    assert_eq!(tokens[16].token_type, TokenType::Return);
    match &tokens[17].token_type {
        TokenType::Identifier(name) => assert_eq!(name, "a"),
        _ => panic!("Expected identifier in expression"),
    }
    
    // Check comparison operator
    assert_eq!(tokens[18].token_type, TokenType::Greater);
    
    match &tokens[19].token_type {
        TokenType::Identifier(name) => assert_eq!(name, "b"),
        _ => panic!("Expected identifier in expression"),
    }
    
    // Check closing brace
    assert_eq!(tokens[20].token_type, TokenType::RightBrace);
    
    // Check for EOF
    assert_eq!(tokens[21].token_type, TokenType::Eof);
    
    // Make sure we have the expected number of tokens
    assert_eq!(tokens.len(), 22, "Should have 22 tokens in the function declaration");
}