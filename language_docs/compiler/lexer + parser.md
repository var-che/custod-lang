# Lexer and Parser Documentation

## Overview
This document demonstrates how source code is processed through the lexical analysis and parsing stages.

## Processing Example
```rust
reads counter = 44
writes total = 0
```

## Lexical Analysis
The lexer breaks down source code into tokens, tracking line and column positions:

```rust
[
    Token { token_type: Permission(Reads), line: 1, column: 1 },
    Token { token_type: Identifier("counter"), line: 1, column: 7 },
    Token { token_type: Equal, line: 1, column: 15 },
    Token { token_type: Number(44), line: 1, column: 17 },
    Token { token_type: Permission(Writes), line: 2, column: 1 },
    Token { token_type: Identifier("total"), line: 2, column: 7 },
    Token { token_type: Equal, line: 2, column: 13 },
    Token { token_type: Number(0), line: 2, column: 15 }
]
```

## Abstract Syntax Tree
The parser organizes tokens into a hierarchical structure:

```rust
Vec<Statement>[
    Statement::Declaration(
        VariableDeclaration {
            permission: PermissionType::Reads,
            name: "counter",
            initializer: Expression::Number(44)
        }
    ),
    Statement::Declaration(
        VariableDeclaration {
            permission: PermissionType::Writes,
            name: "total",
            initializer: Expression::Number(0)
        }
    )
]
```

## Key Components

### Token Types
- `Permission` - Language permission keywords (reads, writes)
- `Identifier` - Variable names
- `Equal` - Assignment operator
- `Number` - Numeric literals

### AST Nodes
- `Statement::Declaration` - Variable declarations
- `VariableDeclaration` - Holds permission, name and value
- `Expression` - Represents values and operations

## Error Handling
The parser provides error detection for:
- Missing identifiers
- Invalid permission declarations
- Incorrect syntax

## Position Tracking
Both lexer and parser maintain:
- Line numbers
- Column positions
- Source location for error reporting