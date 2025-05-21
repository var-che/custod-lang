# MIR (Middle-level Intermediate Representation)

The MIR sits between the HIR and the code generation phase, providing a lower-level representation that makes control flow and memory operations explicit while still being machine-independent.

## Core Objectives

1. **Simplify Control Flow**: Convert high-level control structures into basic blocks with explicit jumps and branches.
2. **Flatten Expressions**: Break down nested expressions into simpler operations with temporary variables.
3. **Explicit Memory Operations**: Make memory allocations, borrows, and moves explicit.
4. **Optimize for Analysis**: Provide a form that's easier to analyze and optimize than HIR.
5. **Lower Permissions**: Translate our permission system into concrete memory access patterns.

## Initial Implementation

For our first MIR implementation, we'll focus on these core components:

### Data Structures

- **Basic Blocks**: Sequences of instructions that execute linearly, with explicit jumps at the end
- **Instructions**: Simple operations like:
  - Arithmetic operations
  - Variable assignments
  - Memory allocations
  - Function calls
  - Conditional and unconditional jumps
- **MIR Functions**: Collections of basic blocks with parameter and return information
- **MIR Program**: Collection of functions and global variables

### Simple Implementation Components

1. **types.rs**: Core MIR data structures (blocks, instructions, etc.)
2. **converter.rs**: Transform HIR to MIR with basic block creation
3. **validation.rs**: Simple validator to ensure MIR correctness

### First Operations to Support

- Integer arithmetic
- Simple assignments
- Function calls
- Conditional branching
- Basic variable declarations

## Future Enhancements (Post-Initial Implementation)

1. **Optimization Passes**
   - Constant propagation
   - Common subexpression elimination
   - Dead code elimination
   
2. **Memory Model**
   - Explicit lifetime tracking
   - Borrow checking implementation
   
3. **SSA Form**
   - Conversion to Static Single Assignment for better optimization
   
4. **Advanced Control Flow**
   - Exception handling
   - Loop optimizations
   
5. **Visualization**
   - Basic block graph visualization

## Relationship with HIR
The MIR takes the output of HIR validation and lowers it to a representation where:
- All types are fully resolved
- Control flow is explicit
- Permission system is translated to concrete memory access patterns
- Expression trees are flattened

This representation is ideal for performing machine-independent optimizations before moving to backend code generation.