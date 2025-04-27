Single ownership memory menagement
Explicit
Typescriptian/Rustian syntax.
Easy `async`

## What is the goal of this project?
This language aims to combine the best aspects of modern programming languages to create a developer-friendly environment for building highly scalable server-side systems. Key features include:

- **Familiar Syntax**: Inspired by TypeScript and Rust, making it approachable for web developers
- **[[Actor]] Model**: Built-in support for actor-based concurrency, similar to Pony and Inko languages
- **Memory Safety**: Single ownership model with explicit memory management
- **Target Audience**: Backend developers focusing on scalable server systems
- **Performance**: Native performance with lightweight process-based implementation

### Implementation Details
#### Compiler Pipeline
- **Frontend**:
  - Tokenizer: Breaks source code into tokens
  - Lexer: Analyzes tokens for language constructs
  - Parser: Builds Abstract Syntax Tree (AST)
- **Middle-end**:
  - HIR (High-level IR): Initial intermediate representation
  - MIR (Mid-level IR): Optimized intermediate representation
- **Backend**:
  - Runtime: Lightweight process model similar to Erlang
  - Concurrency: Actor-based message passing system
  - Memory Management: Single ownership system

### Current Scope
- Server-side application development
- Concurrent and distributed systems
- High-performance network services

Permissions: 
- [[read,write]]
- [[reads]]
- [[reads write]]

## TODO / read
- [Whirley spec pdf](https://whiley.org/pdfs/WhileyLanguageSpec.pdf) - inspiration of how to write a spec, and consider `requires` and `ensures` keywords for functions.