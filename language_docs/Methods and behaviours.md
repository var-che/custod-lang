### Regular Methods (fn)
- Synchronous execution
- Return values allowed
- Stack-bounded execution
- Automatically atomic within actor
- Only callable from inside actor
### Behaviors (on)
- Asynchronous execution
- No return values
- Message-queue based
- Requires explicit atomicity for multi-step operations
- Callable from outside actor
- Transaction Pattern

Behaviors can provide controlled access to actor state through transactions:
- Permission Elevation: Inside actor, transactions get elevated access to state
- Atomic Execution: Multiple operations execute as one unit
- State Safety: Actor state remains consistent

## Examples
### Basic Actor Methods
```rust
actor Counter {
    reads write count: i64

    // Regular method - internal use only
    fn increment_by(n: i64) -> i64 {
        count += n
        count
    }

    // Behavior - external interface
    on increment() {
        count += 1
    }
}
```
### Transaction Pattern Example
```rust
// A shared register actor that maintains multiple named counters
actor SharedRegisters {
    reads write data: Map<String, i64>

    // Constructor
    fn new() -> SharedRegisters {
        SharedRegisters {
            data: Map::new()
        }
    }

    // Internal functions - only callable inside actor
    fn read_now(name: String) -> i64 {
        data.get(name).unwrap_or(0)
    }

    fn write_now(name: String, value: i64) {
        data.insert(name, value)
    }

    // External behavior - provides atomic transaction access
    on access(transaction: fn(reads write registers: SharedRegisters)) {
        transaction(this)
    }
}

actor BankAccount {
    reads write balance: i64
    reads write registers: SharedRegisters

    // Regular method for construction
    fn new(initial: i64, reg: SharedRegisters) -> BankAccount {
        BankAccount {
            balance: initial,
            registers: reg
        }
    }

    // Behavior with transaction
    on transfer(amount: i64, to: String) {
        if balance >= amount {
            registers.access(|reg| {
                let recipient = reg.read_now(to)
                reg.write_now(to, recipient + amount)
                balance -= amount
            })
        }
    }
}
fn main() {
    // Create our shared registers
    reads write registers = SharedRegisters::new()

    // Example 1: Simple increment transaction
    registers.access(|reg| {
        let value = reg.read_now("counter")
        reg.write_now("counter", value + 1)
    })

    // Example 2: Complex transaction with multiple operations
    registers.access(|reg| {
        // Read two counters
        let x = reg.read_now("x")
        let y = reg.read_now("y")
        
        // Update both atomically
        reg.write_now("x", x + y)
        reg.write_now("y", x * 2)
    })

    // Example 3: Conditional transaction
    registers.access(|reg| {
        let balance = reg.read_now("balance")
        let withdrawal = reg.read_now("pending")

        if balance >= withdrawal {
            reg.write_now("balance", balance - withdrawal)
            reg.write_now("pending", 0)
        }
    })
}
```
