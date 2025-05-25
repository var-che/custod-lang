```rust
rw OR rs d = ({
  // scoped block
  // we can use outside scope aliases that are `rs` only, meaning `reads`
})
```
# Statements
## Variable declaration statement
```rust
[permission] alias = expression
```
Examples:
```rust
read write single = 42
rw single = 42
rsws multi = 55
reads writes multi = 55
```
# Permissions
The language has to insure that there will be mutable aliases, but at the same time, we need to be sure that those aliases are safe to be sent over to another actor, without making the clone or duplication of the same data. 

Mutability is not bad, in fact, it has a tradeoff. It is easier to write a code that allows mutations, but can allow some unintentional behavior to occur. In a concurrent based systems, it is crucial to have a single reference to a value, in order to be safely transferred to another actor. There has to be a clear line of what you can do with a variable.
### read write or rw 
There are two types of permissions that are safe to toss around to actors:
```rust
read write 
reads
```
`read write` or `rw` for short, is a single reference to some value.
```rust
read write counter = 1
```
In there, you cannot assign another alias to `counter` value. `counter` has an exclusive read and write permission. It would throw an error if you would try to do it:
```rust
read write counter = 1
read c = counter
```
would cause an error like this:
```
Error: Single ownership of the value

2 | read c = counter
             ~~~~~~~ -> `counter` has an explicit ownership of that data and you cannot assign another alias to read nor write to it.

Solution:
Delete the whole statement and let `counter` be the single owner of that value.

2 | read c = counter
    ~~~~~~~~~~~~~~~~ -> Delete that stuff
```
### reads write, or rsw
A permission that allows many aliases to read into this value, but only single writer to the value.

TODO

# Types
It is important to have statically defined types and inference system to detect them. 
# Loops
loops in this? lets explore how loops would look like in our language.
```rust
rw c: Int = 4
while(c < 10) {
  print(c)
  c += 1
} 
```

# Recover
How about `recover` in our language? 
I am thinking about this. We will declare an alias and just have no perms attached to it,
enclosed within some brackets. I want {} to be reserved for structs, and we need to differentiate
those with our recover system.
```rust
un_permed = ({
  // series of statements, with last one being returned back
})
...
rs un_permed // would set the perms for that alias.
```

We can also declare perms as well:
```rust
rs premed = ({
  // statements, last one returned
  4
})
```
this is making the `permed` alias with `reads` meaning it is immutable, any attempt to change it later would throw an error.

`[no perms] alias = ({})` is defaulting to mutable structure `rsws alias = ({})`
but it is required later on during the program to declare what perms it would have:

`[r | w | rs | rw ] alias = ({})`

If we dont declare the permissions in the code for that alias, we need to throw a compile time error that will tell the user
what needs to be done. An example is:
```rust
un_permed = ({
  4
})
```
the debugger/compiler needs to be friendly and concrete about what needs to be done if no perms are set. Current idea is:
```
Error: Missing declaring permission with `un_permed`
1 | un_permed = ({

Solution 1: Add perms right next to alias declaration:
1 | rs un_permed = ({
    ~~ -> Add permission next to `un_permed`

Solution 2: Add perms later on, but put them.
1    | un_permed = ({
...
n'th | rs un_permed 
       ~~ -> Add them here.
```
This approach is similar to what Pony does with its `recover` block, which has loosened up capabilities restriction within that block.

# Functions
