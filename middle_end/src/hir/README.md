# HIR (High-level Intermediate Representation)

The HIR is the first intermediate representation in our compiler pipeline, sitting between the front-end parser and the MIR (Middle Intermediate Representation). It provides a more structured and typed view of the program while maintaining the high-level structure that closely resembles the source code.

## Key Components

### Core Modules

- **types.rs**: Defines the HIR data structures (statements, expressions, variables, functions)
- **converter.rs**: Transforms the front-end AST into HIR structures
- **validation.rs**: Validates the HIR for semantic correctness (type checking, etc.)
- **scope.rs**: Manages symbol tables and scoping information
- **name_resolver.rs**: Resolves variable/function references to their declarations
- **diagnostics.rs**: Provides detailed error reporting with source location context

### Permission System

- **permissions.rs**: Implements our capability-based permission system (read/write/reads/writes)
- **function_analysis.rs**: Analyzes permission flow in function calls

### Transformation & Optimization

- **desugar.rs**: Simplifies complex language constructs into simpler ones
- **const_fold.rs**: Performs constant folding optimizations
- **dce.rs**: Eliminates dead code (unused variables, unreachable code)

### Utilities

- **pretty_print.rs**: Converts HIR back to readable source-like format

## How They Work Together

1. **Parsing to HIR**:
   - Front-end parser produces an AST
   - `converter.rs` transforms this into the HIR structure

2. **Analysis Phase**:
   - `name_resolver.rs` links variable references to declarations
   - `scope.rs` tracks variable scopes and detects shadowing
   - `permissions.rs` validates permission rules (read/write/reads/writes)
   - `validation.rs` performs type checking and semantic validation

3. **Transformation Phase**:
   - `desugar.rs` simplifies complex constructs
   - `const_fold.rs` performs compile-time evaluation
   - `dce.rs` removes unused code

4. **Error Handling**:
   - `diagnostics.rs` formats and reports errors with source code context
   - Integration with front-end error reporting

## Permission System

Our HIR implements a capability-based permission system inspired by Rust and Pony:

- **read**: Non-shareable read permission (like Pony's box)
- **write**: Non-shareable write permission (like Pony's trn)
- **reads**: Shareable read permission (like Pony's val)
- **writes**: Shareable write permission
- **read write**: Exclusive access (like Pony's iso)
- **reads writes**: Fully shareable (like Pony's ref)

These permissions are enforced during HIR validation to ensure memory safety and prevent data races.

## Error Reporting

HIR provides rich error diagnostics with:

- Precise source code locations
- Syntax-highlighted code snippets
- Helpful suggestions for fixing issues
- Clear explanations of permission violations
- Type mismatch details with conversion hints

## Next Steps

The validated HIR is then lowered to MIR (Middle Intermediate Representation), which further flattens the code structure and makes control flow and memory operations explicit in preparation for backend code generation.
