# Overview
The front-end compiler handles the initial phase of compilation, converting source code into tokens that can be processed by later stages. This implementation focuses on lexical analysis for a language with TypeScript/Rust-like syntax and permission-based memory management.

## Components

### 1. Lexical Analysis (Lexer)
The lexer breaks down source code into a sequence of tokens. It's implemented in `lexer.rs`.

#### Key Features:
- Handles permission-based declarations (`read,write`)
- Processes basic arithmetic operators
- Tracks line and column numbers
- Manages whitespace and newlines

#### Token Types (`token.rs`):
```rust
pub enum TokenType {
    Permission(String),  // read,write
    Identifier(String), // variable names
    Equal,              // =
    Number(i64),       // numeric literals
    Comma,             // ,
    Plus,              // +
    Newline,
    EOF,
}
```

### 2. Example Processing

Given the input:
```rust
read,write counter = 1
```

The lexer produces the following tokens:
1. `Permission("read")`
2. `Comma`
3. `Permission("write")`
4. `Identifier("counter")`
5. `Equal`
6. `Number(1)`
7. `EOF`

### 3. Implementation Details

#### Lexer Structure
```rust
pub struct Lexer {
    input: Vec<char>,    // Input source code as characters
    position: usize,     // Current position in input
    line: usize,        // Current line number
    column: usize,      // Current column number
}
```

#### Key Methods
- `next_token()`: Main method for token generation
- `read_identifier()`: Handles variable names and keywords
- `read_number()`: Processes numeric literals
- `skip_whitespace()`: Manages spacing and formatting

### 4. Testing
The implementation includes unit tests for:
- Basic permission declarations
- Multi-line code processing
- Line number tracking
- Token sequence verification

### 5. Usage Example

```rust
let input = "read,write counter = 1";
let mut lexer = Lexer::new(input);

while let token = lexer.next_token() {
    if token.token_type == TokenType::EOF {
        break;
    }
    // Process token
}
```

## Error Handling
- Position tracking for error reporting
- Line and column number maintenance
- Panic on unexpected characters (to be enhanced with proper error handling)

## Future Enhancements
1. Error recovery mechanisms
2. Support for more operators
3. String literal handling
4. Comment processing
5. Enhanced position tracking for better error messages