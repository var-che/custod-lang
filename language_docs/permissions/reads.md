# `reads` Permission

## Overview
The `reads` permission creates an immutable alias that remains constant throughout the program's lifetime. This permission guarantees that the value cannot be modified, making it safe for sharing across actors and concurrent operations.

## Syntax
```rust
reads config = 44
```

## Key Characteristics
- Value is immutable for the entire program lifetime
- Safe to share across multiple actors
- Supports deep cloning when creating new `reads` aliases
- Allows temporary access via `read` permission

## Examples

### Basic Usage
```rust
reads config = 44
read temp = config        // Temporary read access
console.log(temp)        // 44
```

### Deep Cloning
```rust
reads original = [1, 2, 3]
reads cloned = original   // Creates a deep clone
```

### Temporary Access
```rust
reads shared_data = {
    port: 8080,
    host: "localhost"
}
read temp = shared_data   // Creates temporary read access
```

## Error Cases

### Invalid Write Access
```rust
reads config = 44
write temp = config       // Error: Cannot grant write permission to reads
```

### Direct Mutation
```rust
reads config = { port: 8080 }
config.port = 3000       // Error: Cannot mutate reads value
```

## Creating Mutable Copies
The `copy` keyword allows creating new aliases with different permissions:

```rust
reads template = {
    version: "1.0",
    timestamp: now()
}
read,write mutable = copy template   // Creates mutable copy
```

## Best Practices
1. Use for configuration and constant values
2. Prefer `read` for temporary access
3. Use `copy` when mutation is needed
4. Document deep clone operations

## Common Use Cases
- Configuration management
- Shared constants
- Immutable data structures
- Cross-actor communication
