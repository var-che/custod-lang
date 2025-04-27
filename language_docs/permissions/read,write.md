The `read,write` permission gives an alias exclusive ability to both `read` from and `write` to a value. No other alias can access this value unless it is explicitly transferred or consumed. This is the most exclusive capability and behaves similarly to iso or box in other actor-based languages, depending on how it is used.

Only one alias may have `read,write` access to a value at any given time.
## Syntax
```rust
read,write counter = 0
```
## Behavior

- The alias counter can `read` and `write` the value.
- No other alias can read or write to this value unless it is moved via `consume` or transferred using a specific language feature (e.g., to another actor).
- This guarantees thread-safe mutation and encapsulated ownership.
## Examples

### Basic mutation
```rust
read,write counter = 0

counter = counter + 1
console.log(counter) // 1
```
### Attempting to create another alias (❌ Not allowed)
```rust
read,write counter = 0
read another = counter // ❌ Error: counter is exclusively owned
```
### Transferring ownership via consume
```rust
read,write counter = 10
read,write next = consume counter

next = 42
console.log(next) // 42
```