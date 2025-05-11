use crate::lexer::Lexer;
use crate::token::{Token, TokenType};

#[test]
fn test_lexer_numbers() {
    let source = "123 456 0";
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    debug_print_tokens(&tokens);
    
    assert_eq!(tokens.len(), 4); // 3 numbers + EOF
    assert_eq!(tokens[0].token_type, TokenType::Number(123));
    assert_eq!(tokens[1].token_type, TokenType::Number(456));
    assert_eq!(tokens[2].token_type, TokenType::Number(0));
    assert_eq!(tokens[3].token_type, TokenType::Eof);
}

#[test]
fn test_lexer_operators() {
    let source = "+ - * / = == != < > <= >= += -= *= /=";
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    
    assert_eq!(tokens.len(), 15); // 14 operators + EOF
    assert_eq!(tokens[0].token_type, TokenType::Plus);
    assert_eq!(tokens[1].token_type, TokenType::Minus);
    assert_eq!(tokens[2].token_type, TokenType::Star);
    assert_eq!(tokens[3].token_type, TokenType::Slash);
    assert_eq!(tokens[4].token_type, TokenType::Equal);
    // And so on...
}

#[test]
fn test_lexer_keywords() {
    let source = "fn actor on reads writes read write peak return if else print i64";
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    debug_print_tokens(&tokens);
    
    assert_eq!(tokens.len(), 14); // Update to 14 tokens based on actual output
    assert_eq!(tokens[0].token_type, TokenType::Fn);
    assert_eq!(tokens[1].token_type, TokenType::Actor);
    assert_eq!(tokens[2].token_type, TokenType::On);
    assert_eq!(tokens[3].token_type, TokenType::Reads);
    assert_eq!(tokens[4].token_type, TokenType::Writes);
    // And so on...
}

#[test]
fn test_lexer_identifiers() {
    let source = "x y foo bar baz counter_123";
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    debug_print_tokens(&tokens);

    assert_eq!(tokens.len(), 7); // 6 identifiers + EOF
    
    if let TokenType::Identifier(name) = &tokens[0].token_type {
        assert_eq!(name, "x");
    } else {
        panic!("Expected identifier token");
    }
    
    if let TokenType::Identifier(name) = &tokens[5].token_type {
        assert_eq!(name, "counter_123");
    } else {
        panic!("Expected identifier token");
    }
}

#[test]
fn test_lexer_mixed_code() {
    let source = "fn add(reads x: i64, reads y: i64) -> i64 { return x + y }";
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    debug_print_tokens(&tokens);
    
    // Check the token count and some key tokens
    assert!(tokens.len() > 10);
    assert_eq!(tokens[0].token_type, TokenType::Fn);
    
    // Find the return token and check what follows
    let mut return_index = 0;
    for (i, token) in tokens.iter().enumerate() {
        if token.token_type == TokenType::Return {
            return_index = i;
            break;
        }
    }
    
    assert!(return_index > 0);
    
    // Check tokens after return
    if let TokenType::Identifier(name) = &tokens[return_index + 1].token_type {
        assert_eq!(name, "x");
    } else {
        panic!("Expected identifier after return");
    }
    
    assert_eq!(tokens[return_index + 2].token_type, TokenType::Plus);
    
    if let TokenType::Identifier(name) = &tokens[return_index + 3].token_type {
        assert_eq!(name, "y");
    } else {
        panic!("Expected identifier after plus");
    }
}

#[test]
fn test_lexer_permission_keywords() {
    let source = "reads writes read write peak";
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    debug_print_tokens(&tokens);
    
    assert_eq!(tokens.len(), 6); // 5 keywords + EOF
    assert_eq!(tokens[0].token_type, TokenType::Reads);
    assert_eq!(tokens[1].token_type, TokenType::Writes);
    assert_eq!(tokens[2].token_type, TokenType::Read);
    assert_eq!(tokens[3].token_type, TokenType::Write);
    assert_eq!(tokens[4].token_type, TokenType::Peak);
}

#[test]
fn test_lexer_variable_declarations_with_permissions() {
    let source = "reads counter = 5\nwrite value = 10";
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    debug_print_tokens(&tokens);
    
    assert_eq!(tokens[0].token_type, TokenType::Reads);
    
    if let TokenType::Identifier(name) = &tokens[1].token_type {
        assert_eq!(name, "counter");
    } else {
        panic!("Expected identifier after reads");
    }
    
    assert_eq!(tokens[2].token_type, TokenType::Equal);
    assert_eq!(tokens[3].token_type, TokenType::Number(5));
    
    assert_eq!(tokens[4].token_type, TokenType::Write);
    
    if let TokenType::Identifier(name) = &tokens[5].token_type {
        assert_eq!(name, "value");
    } else {
        panic!("Expected identifier after write");
    }
}

#[test]
fn test_lexer_function_with_permission_parameters() {
    let source = "fn increment(reads writes value: i64) -> i64 {
        value = value + 1
        return value
    }";
    
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    debug_print_tokens(&tokens);
    
    assert_eq!(tokens[0].token_type, TokenType::Fn);
    
    if let TokenType::Identifier(name) = &tokens[1].token_type {
        assert_eq!(name, "increment");
    }
    
    assert_eq!(tokens[2].token_type, TokenType::LeftParen);
    assert_eq!(tokens[3].token_type, TokenType::Reads);
    assert_eq!(tokens[4].token_type, TokenType::Writes);
    
    if let TokenType::Identifier(name) = &tokens[5].token_type {
        assert_eq!(name, "value");
    }
    
    assert_eq!(tokens[6].token_type, TokenType::Colon);
    assert_eq!(tokens[7].token_type, TokenType::TypeI64);
}

#[test]
fn test_lexer_complex_permissions() {
    let source = "reads write counter = 5
        
    fn increment(reads writes value: i64) -> i64 {
        value = value + 1
        return value
    }
        
    reads write result = increment(counter)
    print result
    print counter";
    
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    debug_print_tokens(&tokens);
    
    // Verify the initial declaration
    assert_eq!(tokens[0].token_type, TokenType::Reads);
    assert_eq!(tokens[1].token_type, TokenType::Write);
    
    if let TokenType::Identifier(name) = &tokens[2].token_type {
        assert_eq!(name, "counter");
    }
    
    // Find the specific function call "increment(counter)"
    // Look after the second "reads write" which marks the second variable declaration
    let mut start_index = 0;
    let mut reads_write_count = 0;
    
    for (i, token) in tokens.iter().enumerate() {
        if token.token_type == TokenType::Reads && 
           i + 1 < tokens.len() && 
           tokens[i+1].token_type == TokenType::Write {
            reads_write_count += 1;
            
            if reads_write_count == 2 {
                start_index = i;
                break;
            }
        }
    }
    
    assert!(start_index > 0, "Failed to find second 'reads write' sequence");
    
    // Now look for 'increment' after this point
    let mut function_call_index = 0;
    for i in start_index..tokens.len() {
        if let TokenType::Identifier(name) = &tokens[i].token_type {
            if name == "increment" && 
               i + 1 < tokens.len() && 
               tokens[i+1].token_type == TokenType::LeftParen {
                function_call_index = i;
                break;
            }
        }
    }
    
    assert!(function_call_index > 0, "Failed to find increment function call");
    
    // Print the token we're trying to check for debugging
    println!("Function call found at index: {}", function_call_index);
    println!("Token at function_call_index + 1: {:?}", tokens[function_call_index + 1].token_type);
    println!("Token at function_call_index + 2: {:?}", tokens[function_call_index + 2].token_type);
    
    // Updated check for function argument
    match &tokens[function_call_index + 2].token_type {
        TokenType::Identifier(name) => {
            assert_eq!(name, "counter", "Expected counter as function argument");
        },
        other => {
            panic!("Expected identifier as function argument, got {:?}", other);
        }
    }
}

#[test]
fn test_lexer_peak_reference() {
    let source = "reads writes counter = 5
    read r = peak counter";
    
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    debug_print_tokens(&tokens);
    
    // Check for the peak keyword
    let mut peak_index = 0;
    for (i, token) in tokens.iter().enumerate() {
        if token.token_type == TokenType::Peak {
            peak_index = i;
            break;
        }
    }
    
    assert!(peak_index > 0, "Failed to find peak keyword");
    
    // Check what comes after peak
    if let TokenType::Identifier(name) = &tokens[peak_index + 1].token_type {
        assert_eq!(name, "counter", "Expected counter after peak");
    } else {
        panic!("Expected identifier after peak");
    }
}

fn debug_print_tokens(tokens: &[Token]) {
    for (i, token) in tokens.iter().enumerate() {
        println!("Token {}: {:?} (\"{}\")", i, token.token_type, token.lexeme);
    }
    println!("Total tokens: {}", tokens.len()); // Add this line to verify the count
}