pub mod hir;
pub mod mir;  // Keep your existing MIR module
pub mod optimize;
pub mod analysis;
pub mod interpreter;
pub mod type_checker;

#[cfg(test)]
mod tests;