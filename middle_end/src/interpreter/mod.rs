//! Interpreter for MIR code
//! 
//! This module provides functionality to execute MIR code directly,
//! primarily for testing and experimental purposes.

mod core;
mod values;
mod memory;
mod functions;

#[cfg(test)]
mod tests;

// Re-export main components
pub use core::Interpreter;
pub use values::InterpreterValue;
pub use functions::FunctionContext;