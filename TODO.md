5/18/25
[ ] Added symbol table for errors and tracing declarations within the proper scopes.
[ ] Add tests for write writes read reads peak clone.

5/12/25
[+] Peak reference behavior fully implemented and tested in the compiler pipeline
[+] Binary expressions in MIR are properly evaluated, allowing operations like `c = c + 5`
[+] Compiler pipeline integrated from lexing through execution

Investigate Zig features for server development:
[+] Explicit error handling with error union types (try/catch equivalent)
[ ] Defer statements for guaranteed resource cleanup
[ ] Explicit allocator passing for memory management
[ ] Comptime for zero-cost abstractions
[ ] Optional types vs nullable pointers for safer null handling
[ ] Slices instead of pointer+length for buffer management

5/11/25
[+] `fn` is implemented but without any scope ideas. Aliases are flatten in the global environment. 
[+] variables declared inside of the functions need to be isolated from the outer environment. 
[ ] name reuse across different functions.
[+] function parameters exist only during function execution. we need to clean the variables after the function is done. Closures TBD
[+] parameters need to be properly initialized with argument values
[+] parameters need to be cleaned up when the function returns
Resource Management
[ ] memory and other resources allocated in a function should be released when it completes
[ ] prevents memory leaks and resource exhaustion
Clean State between calls
[+] each function call starts with a fresh environment (except for globals)
[ ] ensure predictable behavior across multiple invocations

5/10/25
[+] `read` and `peak`. `read` is a permission that allows user to create an alias to `read` / `peak` from their value. Similar to `box` in Pony lang.

Appropriate tests were created for this feature.
