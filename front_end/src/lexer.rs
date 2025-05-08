impl Lexer {
    fn scan_keyword(&mut self) -> Token {
        while self.is_alphanumeric(self.peek()) {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        match text {
            "actor" => Token::new(TokenType::Actor, text),
            "on" => Token::new(TokenType::On, text),
            "fn" => Token::new(TokenType::Fn, text),
            "atomic" => Token::new(TokenType::Atomic, text),
            "reads" => Token::new(TokenType::Reads, text),
            "write" => Token::new(TokenType::Write, text),
            _ => Token::new(TokenType::Identifier(text.to_string()), text),
        }
    }
}