use crate::token::TokenType;
use crate::token::Token;

pub struct Lexer {
    source: String,
    start: usize,       // Start position of current token in source
    current: usize,     // Current position in source
    line: usize,        // Current line
    column: usize,      // Current column
    start_column: usize, // Starting column of current token
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,      // Lines are 1-indexed
            column: 1,    // Columns are 1-indexed
            start_column: 1,
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
            self.column += 1;
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
            self.column += 1;
            
            // Handle newlines for line/column tracking
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            }
            
            c
        } else {
            '\0'
        }
    }

    fn scan_identifier(&mut self) -> Token {
        while self.is_alphanumeric(self.peek()) {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let token_type = match text {
            // Existing keywords
            "fn" => TokenType::Fn,
            "on" => TokenType::On,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "print" => TokenType::Print,
            
            // Permission modifiers
            "reads" => TokenType::Reads,
            "writes" => TokenType::Writes,
            "read" => TokenType::Read,
            "write" => TokenType::Write,
            
            // Return keyword - add this line
            "return" => TokenType::Return,
            
            // Permission operations
            "peak" => TokenType::Peak,    // Add peak keyword
            "clone" => TokenType::Clone,  // Add clone keyword
            
            // Types
            "Int" => TokenType::TypeInt,
            "Int8" => TokenType::TypeInt8,
            "Float64" => TokenType::TypeFloat64,
            "Bool" => TokenType::TypeBool,
            // ... other types
            
            // Default case - it's an identifier
            _ => TokenType::Identifier(text.to_string()),
        };

        Token::new(token_type, text, self.line, self.start_column)
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
            Token::new(TokenType::Number(value), text, self.line, self.start_column)
        } else {
            // Provide a fallback in case parsing fails
            Token::new(TokenType::Error(format!("Invalid number: {}", text)), text, self.line, self.start_column)
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
            self.start_column = self.column;
            let token = self.scan_token();
            tokens.push(token);
        }
        
        // Only add EOF token if we don't already have one
        // This prevents duplicate EOF tokens
        if tokens.last().map_or(true, |t| t.token_type != TokenType::Eof) {
            tokens.push(Token::new(TokenType::Eof, "", self.line, self.start_column));
        }
        
        tokens
    }

    fn scan_token(&mut self) -> Token {
        // Skip whitespace before starting a new token
        self.skip_whitespace();
        
        // Remember the start position
        self.start = self.current;
        self.start_column = self.column;
        
        if self.is_at_end() {
            return Token::new(TokenType::Eof, "", self.line, self.start_column);
        }
        
        let c = self.advance();
        
        match c {
            // Single-character tokens
            '(' => Token::new(TokenType::LeftParen, "(", self.line, self.start_column),
            ')' => Token::new(TokenType::RightParen, ")", self.line, self.start_column),
            '{' => Token::new(TokenType::LeftBrace, "{", self.line, self.start_column),
            '}' => Token::new(TokenType::RightBrace, "}", self.line, self.start_column),
            ',' => Token::new(TokenType::Comma, ",", self.line, self.start_column),
            ':' => Token::new(TokenType::Colon, ":", self.line, self.start_column),
            ';' => Token::new(TokenType::Semicolon, ";", self.line, self.start_column),
            
            // Operators that might be one or two characters
            '+' => {
                if self.match_char('=') {
                    Token::new(TokenType::PlusEqual, "+=", self.line, self.start_column)
                } else {
                    Token::new(TokenType::Plus, "+", self.line, self.start_column)
                }
            },
            '-' => {
                if self.match_char('>') {
                    Token::new(TokenType::Arrow, "->", self.line, self.start_column)
                } else if self.match_char('=') {
                    Token::new(TokenType::MinusEqual, "-=", self.line, self.start_column)
                } else {
                    Token::new(TokenType::Minus, "-", self.line, self.start_column)
                }
            },
            '*' => {
                if self.match_char('=') {
                    Token::new(TokenType::StarEqual, "*=", self.line, self.start_column)
                } else {
                    Token::new(TokenType::Star, "*", self.line, self.start_column)
                }
            },
            '/' => {
                if self.match_char('=') {
                    Token::new(TokenType::SlashEqual, "/=", self.line, self.start_column)
                } else {
                    Token::new(TokenType::Slash, "/", self.line, self.start_column)
                }
            },
            '=' => {
                if self.match_char('=') {
                    Token::new(TokenType::EqualEqual, "==", self.line, self.start_column)
                } else {
                    Token::new(TokenType::Equal, "=", self.line, self.start_column)
                }
            },
            '!' => {
                if self.match_char('=') {
                    Token::new(TokenType::BangEqual, "!=", self.line, self.start_column)
                } else {
                    Token::new(TokenType::Bang, "!", self.line, self.start_column)
                }
            },
            '<' => {
                if self.match_char('=') {
                    Token::new(TokenType::LessEqual, "<=", self.line, self.start_column)
                } else {
                    Token::new(TokenType::Less, "<", self.line, self.start_column)
                }
            },
            '>' => {
                if self.match_char('=') {
                    Token::new(TokenType::GreaterEqual, ">=", self.line, self.start_column)
                } else {
                    Token::new(TokenType::Greater, ">", self.line, self.start_column)
                }
            },
            
            // Numbers
            '0'..='9' => self.scan_number(),
            
            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => self.scan_identifier(),
            
            _ => Token::new(TokenType::Error(format!("Unexpected character: {}", c)), &c.to_string(), self.line, self.start_column),
        }
    }
    
    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                ' ' | '\r' | '\t' => {
                    self.advance();
                },
                '\n' => {
                    self.advance();
                },
                // Skip comments
                '/' => {
                    if self.peek_next() == '/' {
                        // Line comment - advance until EOL or EOF
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return; // Not whitespace, so return
                    }
                },
                _ => return, // Not whitespace, so return
            }
        }
    }
}
