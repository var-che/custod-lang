//! Medium-level Intermediate Representation (MIR)
//!
//! The MIR is a lower-level representation than HIR, closer to machine code
//! but still platform-independent. It represents computations as a sequence
//! of simple instructions that operate on values and variables.

pub mod types;
pub mod lowering;
pub mod values;
pub mod instructions;
pub mod functions;

// Re-export key types for convenience
pub use types::{MirValue, MirInstruction, MirFunction};
pub use lowering::lower_hir;
pub use values::convert_hir_value;

mod tests;