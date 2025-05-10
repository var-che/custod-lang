# Permission-Based Language

A programming language with explicit permissions system inspired by Pony, focusing on safe memory management and clear access control.

## Current Language Features

### Permission System

The language implements several permission types that control how variables can be accessed:

1. **reads write**
```rust
reads write counter = 4    // Others can read, only owner can write
counter += 5              // Valid: owner can modify
```

2. **read write** (Exclusive Access)
```rust
read write counter = 4    // Exclusive access - no other aliases allowed
counter += 5             // Valid: owner has full access
read c = counter         // Error: counter has exclusive access
```

3. **read with peak**
```rust
reads write counter = 4
read c = peak counter    // Valid: creates a view into counter
counter += 6            // Changes visible through the view
print c                // Prints updated value (10)
```

### Variable Operations

1. **Cloning Values**
```rust
reads write counter = 55
reads cloned = clone counter   // Creates independent copy
counter += 5                  // Doesn't affect cloned
print cloned                 // Prints original value (55)
```

2. **Temporary Views (peak)**
```rust
reads write counter = 100
read view = peak counter     // Creates a view into counter
counter += 5                // View sees this change
print view                 // Prints updated value (105)
```

### Key Features

- Explicit permission declarations
- Safe variable access control
- View-based referencing (peak)
- Value cloning system
- Clear ownership rules
- Static permission checking

## Language Philosophy

- Explicit over implicit
- Clear permission boundaries
- Safe memory access
- Predictable behavior

More features coming soon!