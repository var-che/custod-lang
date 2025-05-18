pub mod token;
pub mod ast;
pub mod types;
pub mod parser;
pub mod lexer;
pub mod symbol_table;
pub mod source_manager;
pub mod diagnostics_reporter;
pub mod error; // Add new error module
pub mod source_location; // Add new source location module
pub mod type_inference; // Add the new type inference module
pub mod type_checker; // Add the new type checker module

#[cfg(test)]
mod tests;

