//! Middle-end of the compiler
//!
//! This module contains the middle-end components of the compiler pipeline,
//! including HIR (High-level Intermediate Representation) and MIR (Mid-level
//! Intermediate Representation).

pub mod hir;

#[cfg(test)]
mod tests;