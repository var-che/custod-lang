use std::env;
use std::fs;
use std::path::Path;
use std::process;

use front_end::lexer::Lexer;
use front_end::parser::Parser;
use front_end::source_manager::SourceManager;
use front_end::diagnostics_reporter::DiagnosticReporter;
use middle_end::hir::converters::convert_to_hir;
use middle_end::type_system::TypeChecker;

fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: compiler <filename>");
        process::exit(1);
    }
    
    // Read source file
    let filename = &args[1];
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", filename, e);
            process::exit(1);
        }
    };
    
    // Initialize source manager for error reporting
    let mut source_manager = SourceManager::new();
    let file_id = source_manager.add_file(filename.clone(), source.clone());
    
    println!("Compiling {}...", filename);
    
    // FRONT END: Lexical & Syntactic Analysis
    println!("\nPerforming lexical analysis...");
    let mut lexer = Lexer::new(source.clone());
    let tokens = lexer.scan_tokens();
    println!("Generated {} tokens", tokens.len());
    
    println!("\nPerforming syntactic analysis...");
    let mut parser = Parser::from_source(&source);
    let ast = parser.parse_statements();
    println!("Generated AST with {} statements", ast.len());
    
    // Check for front-end errors
    let front_end_errors = parser.get_errors();
    if !front_end_errors.is_empty() {
        println!("\nFound {} front-end errors:", front_end_errors.len());
        let reporter = DiagnosticReporter::new(source_manager);
        for error in front_end_errors {
            if let front_end::error::CompileError::Resolution(res_error) = error {
                println!("{}", reporter.report_error(&res_error));
            } else {
                println!("Error: {:?}", error);
            }
        }
        process::exit(1);
    }
    
    // MIDDLE END: HIR Generation and Type Checking
    println!("\nConverting to HIR...");
    let mut hir_program = convert_to_hir(ast[0].clone());
    for stmt in &ast[1..] {
        let next_hir = convert_to_hir(stmt.clone());
        hir_program.statements.extend(next_hir.statements);
    }
    println!("Generated HIR with {} statements", hir_program.statements.len());
    
    println!("\nPerforming type checking...");
    let mut type_checker = TypeChecker::new();
    let type_errors = type_checker.check_program(&hir_program);
    
    // Report any type errors
    if !type_errors.is_empty() {
        println!("\nFound {} type errors:", type_errors.len());
        for error in type_errors {
            println!("Error: {:?}", error);
        }
        process::exit(1);
    }
    
    println!("\nCompilation successful!");
}
