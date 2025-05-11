use front_end::parser::Parser;
use front_end::lexer::Lexer;
use front_end::ast::{self, Statement}; // Import Statement type explicitly
use middle_end::hir::convert_to_hir;
use middle_end::mir::lowering::lower_hir;
use middle_end::interpreter::Interpreter;
use middle_end::type_checker::TypePermissionChecker;

// Define the Program struct here since it's not available in the imported modules
#[derive(Debug)]
struct Program {
    statements: Vec<Statement>,
}

fn main() -> Result<(), String> {
    // Modified program to avoid using += operator
    let source = r#"
        reads write c = 5
        read r = peak c
        c = c + 5
        print r
    "#;

    println!("\n=== Starting compilation pipeline ===");
    println!("Source code:\n{}", source);

    // Step 1: Lexical analysis - Convert source to tokens
    println!("\n--- Lexical Analysis ---");
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    println!("Generated {} tokens", tokens.len());

    // Step 2: Parsing - Convert tokens to AST
    println!("\n--- Syntax Parsing ---");
    let mut parser = Parser::from_source(source);
    
    // Parse multiple statements
    let statements = parser.parse_statements();
    // Create program using our local Program struct
    let ast = Program { statements };
    
    println!("AST Generated:\n{:#?}", ast);

    // Step 3: HIR Generation - Convert AST to HIR
    println!("\n--- HIR Generation ---");
    let hir = convert_to_hir(ast::Statement::Block(ast.statements));
    println!("HIR Generated:\n{:#?}", hir);

    // Step 4: Type and Permission Checking
    println!("\n--- Type & Permission Checking ---");
    let mut checker = TypePermissionChecker::new();
    checker.check_program(&hir)?;
    println!("Program passed type and permission checks");

    // Step 5: MIR Generation - Lower HIR to MIR
    println!("\n--- MIR Generation ---");
    let mir = lower_hir(&hir);
    println!("MIR Generated:\n{:#?}", mir);

    // Step 6: Execution - Run the MIR code
    println!("\n--- Program Execution ---");
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&mir)?;
    
    println!("\nExecution complete. Final state:");
    println!("Result: {}", result);
    
    // Print the final values of variables
    println!("\nFinal variable states:");
    interpreter.print_variables();

    Ok(())
}