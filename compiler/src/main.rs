use front_end::parser::Parser;
use middle_end::hir::convert_to_hir;  // Changed from lower_ast
use middle_end::mir::lower_hir;
use middle_end::interpreter::Interpreter;

fn main() -> Result<(), String> {
    // Sample program
    let source = r#"
        reads write counter = 1
        counter += 1
        print counter
    "#;

    // Parse source to AST
    let mut parser = Parser::new(source);
    let ast = parser.parse()?;
    println!("AST: {:#?}", ast);  // Use pretty print

    // Convert AST to HIR (changed from lower_ast)
    let hir = convert_to_hir(ast);
    println!("HIR: {:#?}", hir);  // Use pretty print

    // Lower HIR to MIR
    let mir = lower_hir(&hir);
    println!("MIR: {:#?}", mir);  // Use pretty print

    // Execute MIR
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&mir)?;
    println!("Result: {}", result);

    Ok(())
}