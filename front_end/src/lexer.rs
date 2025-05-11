use crate::token::TokenType;
use crate::token::Token;

pub struct Lexer {
    source: String,
    start: usize,
    current: usize,
    line: usize,  // Add line tracking for better error reporting
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,  // Start at line 1
        }
    }
    
    // Check if we've reached the end of the source
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    #[allow(dead_code)]
    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap_or('\0')
        }
    }
    
    // Advance only if the current character matches expected
    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn peek(&self) -> char {
        if self.current >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap_or('\0')
        }
    }

    fn is_alphanumeric(&self, c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    fn advance(&mut self) -> char {
        if self.current < self.source.len() {
            let c = self.source.chars().nth(self.current).unwrap_or('\0');
            self.current += 1;
            c
        } else {
            '\0'
        }
    }

    fn scan_keyword(&mut self) -> Token {
        while self.is_alphanumeric(self.peek()) {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        match text {
            // Keywords relevant for your language features
            "actor" => Token::new(TokenType::Actor, text),
            "on" => Token::new(TokenType::On, text),
            "fn" => Token::new(TokenType::Fn, text),
            "atomic" => Token::new(TokenType::Atomic, text),
            "reads" => Token::new(TokenType::Reads, text),
            "writes" => Token::new(TokenType::Writes, text),  // Added "writes"
            "write" => Token::new(TokenType::Write, text),
            "read" => Token::new(TokenType::Read, text),      // Added "read"
            "peak" => Token::new(TokenType::Peak, text),      // Added "peak" 
            "return" => Token::new(TokenType::Return, text),  // Added "return"
            "if" => Token::new(TokenType::If, text),          // Added "if"
            "else" => Token::new(TokenType::Else, text),      // Added "else"
            "print" => Token::new(TokenType::Print, text),    // Added "print"
            // Type keywords
            "i64" => Token::new(TokenType::TypeI64, text),    // Added "i64"
            
            _ => Token::new(TokenType::Identifier(text.to_string()), text),
        }
    }

    fn scan_number(&mut self) -> Token {
        // Consume the first digit
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Parse the number from the text
        let text = &self.source[self.start..self.current];
        
        // Actually parse the number from the text
        if let Ok(value) = text.parse::<i64>() {
            Token::new(TokenType::Number(value), text)
        } else {
            // Provide a fallback in case parsing fails
            Token::new(TokenType::Error(format!("Invalid number: {}", text)), text)
        }
    }

    // Lexer should convert source text into a stream of tokens
    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        // Reset position counters to be safe
        self.start = 0;
        self.current = 0;
        
        // Keep scanning tokens until we reach the end of the input
        while !self.is_at_end() {
            self.start = self.current; // Reset start for each new token
            let token = self.scan_token();
            tokens.push(token);
        }
        
        // Only add EOF token if we don't already have one
        // This prevents duplicate EOF tokens
        if tokens.last().map_or(true, |t| t.token_type != TokenType::Eof) {
            tokens.push(Token::new(TokenType::Eof, ""));
        }
        
        tokens
    }

    fn scan_token(&mut self) -> Token {
        // Skip whitespace before starting a new token
        while self.peek().is_whitespace() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
            
            // Update the start position after skipping whitespace
            self.start = self.current;
            
            if self.is_at_end() {
                return Token::new(TokenType::Eof, "");
            }
        }
        
        // Now we're at the start of a new token
        let c = self.advance();
        
        match c {
            // Single-character tokens
            '(' => Token::new(TokenType::LeftParen, "("),
            ')' => Token::new(TokenType::RightParen, ")"),
            '{' => Token::new(TokenType::LeftBrace, "{"),
            '}' => Token::new(TokenType::RightBrace, "}"),
            ',' => Token::new(TokenType::Comma, ","),
            ':' => Token::new(TokenType::Colon, ":"),
            ';' => Token::new(TokenType::Semicolon, ";"),
            
            // Operators that might be one or two characters
            '+' => {
                if self.match_char('=') {
                    Token::new(TokenType::PlusEqual, "+=")
                } else {
                    Token::new(TokenType::Plus, "+")
                }
            },
            '-' => {
                if self.match_char('>') {
                    Token::new(TokenType::Arrow, "->")
                } else if self.match_char('=') {
                    Token::new(TokenType::MinusEqual, "-=")
                } else {
                    Token::new(TokenType::Minus, "-")
                }
            },
            '*' => {
                if self.match_char('=') {
                    Token::new(TokenType::StarEqual, "*=")
                } else {
                    Token::new(TokenType::Star, "*")
                }
            },
            '/' => {
                if self.match_char('=') {
                    Token::new(TokenType::SlashEqual, "/=")
                } else {
                    Token::new(TokenType::Slash, "/")
                }
            },
            '=' => {
                if self.match_char('=') {
                    Token::new(TokenType::EqualEqual, "==")
                } else {
                    Token::new(TokenType::Equal, "=")
                }
            },
            '!' => {
                if self.match_char('=') {
                    Token::new(TokenType::BangEqual, "!=")
                } else {
                    Token::new(TokenType::Bang, "!")
                }
            },
            '<' => {
                if self.match_char('=') {
                    Token::new(TokenType::LessEqual, "<=")
                } else {
                    Token::new(TokenType::Less, "<")
                }
            },
            '>' => {
                if self.match_char('=') {
                    Token::new(TokenType::GreaterEqual, ">=")
                } else {
                    Token::new(TokenType::Greater, ">")
                }
            },
            
            // Numbers
            '0'..='9' => self.scan_number(),
            
            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => self.scan_keyword(),
            
            _ => Token::new(TokenType::Error(format!("Unexpected character: {}", c)), &c.to_string()),
        }
    }
}
