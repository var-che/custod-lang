# Language Design Conversation

Q: I understand your language has a permission-based system with keywords like `read`, `write`, `reads`, and `writes`. Could you clarify the main goals of this permission system? Is it primarily aimed at memory safety, concurrency safety, or both?

A: This language is inspired by the Pony language, and what I dint like about Erlang and Elixir, is that it is deals with immutable data only. Not really a fan of Functional programming only. Imperative is what I like. So, Pony solves this issue for concurency, and not making douplicated data when sending the messages to other actors. Here, we have a reference to the data, and if only one alias has reference to some data, it is safe to send it to another actor so that that actor can own that data.

Q:From what I've seen, permissions can be combined (like `read write` or `reads writes`). Are all combinations valid? For example, does `read writes` make sense as a permission?

A: Yes, all those combinations make sense. When there is singular and not plural, `read` not `reads`, simply means that only that one alias can read from it. `reads` means that other aliases can be made that can read that data. Same thing means for `write` and `writes`. It means that only one alias can write to it, and `writes` means that other aliases can exist that can write to that value.

Q:I see the language has a concept of "recover blocks" using `({...})` syntax. How exactly do these differ from regular code blocks? Can you give an example of a practical use case where recover blocks shine?

A: I am thinking about this. We will declare an alias and just have no perms attached to it,
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

Q:The `peak` keyword seems to be used when creating read aliases to values with `reads write` permission. Is this the only way to create such aliases? What happens if someone tries to access without using `peak`?

A: The `peak` is there to express intention, to be explicit in what we are doing with the memory. It is like asking the alias that we want to peak into the value. If it is without peak, the error should be thrown back to the user.

Q: How extensive is the type inference in this language? Does it follow more of a Haskell/ML approach or something closer to Rust/Swift?

A: I havent worked with Haskell/ML a lot to be honest, but I have with Rust, and recently I got into Swift as well. I really like Swift syntax and how they designed the whole language. So, the answer is: closer to Swift.

Q: What about debugging? How do you treat errors that user make?

A: I believe that debugging and throwing errors and suggestions like Rust, is what languages are the top most priority. Having clear error system, where error occured, make suggestion how to fix the error, say all the potential ways to fix the errors is really a great thing in the language. 

Q: Could you explain how function calls interact with the permission system? For example, if I pass an `rw` variable to a function, what happens to its permissions during and after the function call?

A: `rw` or `read write` must be consumed for the function with the keyword `consume`. After that, the original alias that was pointing to that value is invalid. This is an example of how Pony does it:
```pony
fun replace_iso(x: iso U32): iso U32 =>
  consume x  // Explicitly consume the input (optional in Pony 0.54+)
  recover iso 42 end  // Return a new 'iso'

fun main() =>
  let original: iso U32 = recover iso 6 end
  let new_iso = replace_iso(consume original)  // 'original' is gone, 'new_iso' takes over
```
and the pseudo code for our language would be:
```
rw original = 5
fn replace_rw(rw x: Int) -> Int {
    6
}
rw replaced = replace_rw(consume original)
print original // Error, we already consumed it
```

Q: How does your permission system handle data structures like arrays or maps? Do permissions apply at the container level, element level, or both?

A: I havent thought about data structures yet, but probably it would apply to the container level.

Q: What's your approach for handling null or optional values? Will the language have a concept similar to Rust's Option/Result or Swift's optionals?

A: there should be no `null` values at all. Option/Result should be implemented and intuitive as well. I like the Inko lang approach, Zig lang, and Rust for this one.

Q: For the actors you mentioned, how will message passing be implemented syntactically? Will there be special syntax for sending messages between actors?

A: TBD

Q: How will your language handle concurrency primitives beyond actors? Will there be support for futures/promises, async/await, or other concurrency models?

A: I like what Pony does with `be` keyword. However, I would like to have some option or syntactic suggar for `async await` in the future.

Q: Could you elaborate on the memory management approach in your language? Will it use garbage collection, reference counting, or a more manual approach like Rust's ownership system?

A: Well, there is no shared mutable state between actors. Its either immutable (`read, reads`), isolated (`read write`), in essense. There is Automatic GC per actor, meaning that each actor has its own heap and garbage collector, that eliminates stop-the-world pauses. GC is run independantly per actor. Also, every reference has a compile time permission, `read`,`write`, `reads`, `writes` and combination of them, that is dictating how it can be used across actors. So, no data race on the static level. Examples:
`read write` or for short `rw` - unique mutable access, and you can pass data between actors safely.
`reads` - is immutable and globally sharable, read-only data accross actors.
`read` - read-only, alias of `reads writes` / `reads` of for short: `rsws`/`rs`. Function parameters for safety

About actor heaps: each actor manages its own separate heap. Objects sent between actors are deep copied unless its `reads`. GC is non blocking.

About garbage collection: per actor GC, using generational GC. Objects are reclaimed when no longer reachable. And there are no cycles. We need a system that prevents cyclic references.

Q: How will generics or parametric polymorphism work in your language? Are there any interesting constraints or features tied to the permission system?

A: TBD

Q: What is your approach to standard library design? Which core functionalities do you consider most important to have in the standard library versus letting users implement themselves?

A: I actually want standard library to be similar to what JS has.

Q: How will your language approach interoperability with other languages, particularly C/C++ for system integration?

A: TBD

Q: Regarding the error handling model - will it be exceptions, Result types, or something else? How does this interact with the permission system?

A: I think that for now, what i want is something like this:
```
func readFile(rw path: String) -> Result<rw String, rs Error> {
  if path.isEmpty {
    return .error("Empty path")  // Immutable error (`val`)
  } else {
    return .ok(consume path)     // Ownership transferred (`iso`)
  }
}
```

Q: What's your philosophy on operator overloading and custom operators? Will these be supported?

A: No operator overload

Q: Will the language support pattern matching? If so, how extensive will it be?

A: TBD

Q: How do you plan to handle lifetimes and references? Will these be explicit like in Rust or handled differently?

A: For how I have this example to illustrate what I want:
```
fn processFile(rw path: String) -> rs String {
    rw content = path.readAsText() // content is unique, or `read write` 
    content.lowercased() // implicitly converted to `reads` or `rs` because this is the last statement, and since in the fn declaration there is `rs String` we are converting this to `reads` of the type String
}

rw file: String = openFile("data.txt") // `read write` = mutable, owned
rs processed: String = processFile(consume file) // file is consumed
```

Q: How will you handle modules and namespacing in your language? Will there be a specific file structure or import system?

A: 

Q: What's your approach to metaprogramming and compile-time code generation? Will your language support macros or something similar?

A: 

Q: Can you describe how you envision the type system handling interfaces, traits, or protocols? How will polymorphism work?

A: 

Q: How will the language approach mutability beyond the permission system? For example, will there be concepts like constants, final variables, or readonly collections?

A: 

Q: What's your approach to string handling and internationalization? Will there be special support for Unicode or localization features?

A: 

Q: Will your language support closures or anonymous functions? How do they interact with the permission system?

A: 

Q: What is your plan for the build system and package management? How will dependencies be handled?

A: 

Q: How will the language handle testing? Will there be built-in test frameworks or conventions?

A: