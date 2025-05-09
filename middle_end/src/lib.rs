pub mod hir;
pub mod mir;
pub mod optimize;
pub mod analysis;
pub mod interpreter;
pub mod type_checker;

#[cfg(test)]
mod tests;