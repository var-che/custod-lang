//! MIR (Middle Intermediate Representation)
//!
//! This module provides the MIR implementation for the compiler.

pub mod types;
pub mod converter;
pub mod pretty_print;

// Re-export key functions and types
pub use types::{MirProgram, MirFunction, BasicBlock, Instruction, Operand};
pub use converter::convert_hir_to_mir;
pub use pretty_print::pretty_print_program;
