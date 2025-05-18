//! Complete compilation pipeline
//!
//! This module integrates the front-end, middle-end, and (eventually) back-end
//! stages of compilation with proper error handling at each stage.

use front_end::lexer::Lexer;
use front_end::parser::Parser;
use front_end::source_manager::SourceManager;
use front_end::diagnostics_reporter::DiagnosticReporter;
use front_end::error::CompileError;

use middle_end::hir::converters::convert_to_hir;
use middle_end::type_system::{TypeChecker, TypeError};
use middle_end::error_handler::{ErrorHandler, MiddleEndError};
use middle_end::hir::permissions::PermissionChecker;
use middle_end::hir::HirProgram;

/// The result of a compilation stage
pub enum CompilationResult {
    /// Compilation succeeded
    Success,
    /// Compilation failed
    Failure(Vec<String>),
}

impl CompilationResult {
    /// Check if compilation was successful
    pub fn is_success(&self) -> bool {
        matches!(self, CompilationResult::Success)
    }
    
    /// Get error messages if compilation failed
    pub fn error_messages(&self) -> Vec<String> {
        match self {
            CompilationResult::Success => vec![],
            CompilationResult::Failure(msgs) => msgs.clone(),
        }
    }
}

/// The complete compilation pipeline
pub struct CompilationPipeline {
    source_manager: SourceManager,
    reporter: DiagnosticReporter,
    error_handler: ErrorHandler,
    verbose: bool,
}

impl CompilationPipeline {
    /// Create a new compilation pipeline
    pub fn new(verbose: bool) -> Self {
        let source_manager = SourceManager::new();
        let reporter = DiagnosticReporter::new(source_manager.clone());
        let error_handler = ErrorHandler::new();
        
        Self {
            source_manager,
            reporter,
            error_handler,
            verbose,
        }
    }
    
    /// Set source code to compile
    pub fn with_source(&mut self, filename: &str, source: String) -> &mut Self {
        self.source_manager.add_file(filename.to_string(), source);
        self
    }
    
    /// Run the entire compilation pipeline
    pub fn compile(&mut self) -> CompilationResult {
        // Step 1: Front-end (syntax analysis)
        if self.verbose {
            println!("Starting front-end compilation phase...");
        }
        
        let front_end_result = self.run_front_end();
        if !front_end_result.is_success() {
            return front_end_result; // Early return on front-end errors
        }
        
        let ast_statements = match front_end_result {
            CompilationResult::Success => {
                // This is a bit hacky - we're re-parsing to get the AST
                // In a real implementation we'd return the AST from run_front_end
                let source = self.source_manager.get_default_source();
                let mut parser = Parser::from_source(&source);
                parser.parse_statements()
            },
            _ => unreachable!(), // We already checked for success
        };
        
        // Step 2: Middle-end (semantic analysis)
        if self.verbose {
            println!("Starting middle-end compilation phase...");
        }
        
        // 2a: Convert AST to HIR
        let mut hir_program = HirProgram {
            statements: Vec::new(),
            type_info: Default::default(),
        };
        
        for stmt in ast_statements {
            let hir = convert_to_hir(stmt);
            hir_program.statements.extend(hir.statements);
        }
        
        if self.verbose {
            println!("Successfully converted AST to HIR");
        }
        
        // 2b: Check permissions
        if self.verbose {
            println!("Checking permissions...");
        }
        
        if let Err(msg) = PermissionChecker::check_program(&hir_program) {
            self.error_handler.add_permission_error(msg);
        }
        
        // 2c: Type checking
        if self.verbose {
            println!("Performing type checking...");
        }
        
        let mut type_checker = TypeChecker::new();
        let type_errors = type_checker.check_program(&hir_program);
        
        for error in type_errors {
            self.error_handler.add_type_error(error);
        }
        
        // Check for middle-end errors
        if self.error_handler.has_errors() {
            if self.verbose {
                println!("Middle-end analysis found errors:");
            }
            return CompilationResult::Failure(
                self.error_handler.format_errors()
            );
        }
        
        // All compilation stages passed successfully
        if self.verbose {
            println!("Compilation completed successfully!");
        }
        
        CompilationResult::Success
    }
    
    /// Run the front-end phase of compilation
    fn run_front_end(&self) -> CompilationResult {
        let source = self.source_manager.get_default_source();
        
        // Step 1a: Lexical analysis 
        if self.verbose {
            println!("Performing lexical analysis...");
        }
        
        let mut lexer = Lexer::new(source.clone());
        let tokens = lexer.scan_tokens();
        
        if self.verbose {
            println!("Generated {} tokens", tokens.len());
        }
        
        // Step 1b: Parsing
        if self.verbose {
            println!("Performing syntax parsing...");
        }
        
        let mut parser = Parser::from_source(&source);
        let statements = parser.parse_statements();
        
        if self.verbose {
            println!("Parsed {} statements", statements.len());
        }
        
        // Check for front-end errors
        let front_end_errors = parser.get_errors();
        if !front_end_errors.is_empty() {
            if self.verbose {
                println!("Front-end found {} errors:", front_end_errors.len());
            }
            
            let mut error_messages = Vec::new();
            for error in front_end_errors {
                match error {
                    CompileError::Resolution(res_error) => {
                        let formatted = self.reporter.report_error(&res_error);
                        error_messages.push(formatted);
                    },
                    _ => {
                        error_messages.push(format!("Error: {:?}", error));
                    }
                }
            }
            
            return CompilationResult::Failure(error_messages);
        }
        
        CompilationResult::Success
    }
}
