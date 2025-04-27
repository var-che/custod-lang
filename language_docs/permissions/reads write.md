# reads write Permission

`reads write` permission allows:
- Others can read the value (reads)
- Only the owner can write to it (write)
- Can be consumed to become either read-only or exclusive access

## Comparison to Pony
While Pony's `trn` (transitional) allows:
- Only the owner can write
- Others can read through a `box` reference
- Can be consumed to become `iso` or `val`

Our `reads write` is similar but more explicit:
```rust
// Declaration
reads write counter = 100

// Others can read
read other = counter    // Valid
reads shared = counter  // Valid

// Only owner can write
counter = 200          // Valid
other = 300           // Error: other only has read permission

// Consumption
read write owned = consume counter   // Like Pony's iso
reads shared = consume counter       // Like Pony's val
```
We can also have a multi-line statement list that returns the last expression:
```rust
reads write config = {{
    read write temp = load_config()
    validate(temp)
    temp
}}

// Now config is readable by others but only writable by owner, and can be consumed when all data mutability is done.
// ...
reads done = consume config // done is now immutable and can be sent to other actor
```