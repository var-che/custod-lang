# Front End Compiler Pipeline

## Simplified Flow Diagram

            Human-Written Code
                   ↓
               +-------+
               | Lexer |  Converts source text into tokens
               +-------+
                   ↓
              +--------+
              | Parser |  Builds abstract syntax tree
              +--------+
                   ↓
          +----------------+
          | Symbol Table & |  Tracks variables, permissions, and scopes
          | Type Checker   |
          +----------------+
                   ↓
      +------------------------+
      | Diagnostics Reporter  |  Generates error messages
      +------------------------+
                   ↓
            Abstract Syntax Tree (AST)


## What the Front End Does

The front end is the first part of our custom programming language compiler. It takes human-written code and transforms it into a structured representation (AST) that later stages can work with. The front end is responsible for:

1. Reading the source code
2. Checking if it follows the language's syntax and semantics rules
3. Reporting any errors in a helpful way
4. Creating a structured representation of the program

Our language has a unique permission system (read/write, reads/writes) that controls how variables can be accessed and modified.

## Components Explained

### Lexer (`lexer.rs`)

The lexer is like a scanner that reads the raw text of your program character by character and groups them into meaningful "tokens" - the basic units of your language:

- It identifies keywords like `fn`, `return`, `if`, etc.
- It recognizes identifiers (variable and function names)
- It detects literals (numbers, strings)
- It keeps track of line and column positions for error reporting

For example, when it sees `reads write counter: Int = 10`, it converts this into tokens like `[READS, WRITE, IDENTIFIER("counter"), COLON, TYPE_INT, EQUAL, NUMBER(10)]`.

### Parser (`parser.rs`)

The parser takes the stream of tokens from the lexer and builds a tree structure (AST) that represents your program:

- It understands the grammar of your language
- It recognizes patterns like variable declarations, function definitions, etc.
- It ensures the code follows the syntactic rules of the language
- It builds expressions and statements that represent the program's structure

For example, it will recognize `reads write counter: Int = 10` as a variable declaration with read and write permissions, of type Int, with an initial value of 10.

### Symbol Table (`symbol_table.rs`)

The symbol table is a database of all symbols (variables, functions) in your program:

- It tracks what names are defined and where
- It records the type and permissions of each variable
- It manages scopes (global, function, block)
- It detects errors like using undeclared variables or redefining variables
- It enforces permission rules (e.g., can't write to a read-only variable)

This is crucial for our language's permission system, as it tracks which variables have read/write access.

### Types (`types.rs`)

This module defines the type system for the language:

- It defines basic types like Int, Float, Bool
- It handles "permissioned types" that include both a base type and permissions
- It provides validation for type compatibility


### AST (Abstract Syntax Tree) (`ast.rs`)

The AST represents your program as a tree structure that captures its meaning:

- It defines nodes for expressions (numbers, variables, operators)
- It defines nodes for statements (declarations, assignments, returns)
- It provides a structure that can be traversed for further processing
- It has builder patterns to construct complex structures like functions

For example, `counter = counter + amount` becomes an assignment statement with the target "counter" and a binary expression for the right side.

### Source Manager (`source_manager.rs`)

The source manager keeps track of the original source code:

- It stores the text of your program
- It provides access to specific lines or ranges of code
- It helps generate code snippets for error messages

### Diagnostics Reporter (`diagnostics_reporter.rs`)

This generates user-friendly error messages when problems are found:

- It formats errors with code snippets
- It highlights exactly where problems occur
- It provides helpful suggestions for fixing errors
- It uses Rust-like error formatting with colored output

For example, when you try to modify a read-only variable, it shows where the variable was declared and where you're trying to modify it.
