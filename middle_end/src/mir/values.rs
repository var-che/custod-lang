//! MIR Value Handling
//!
//! This module contains functions for working with MIR values,
//! including conversions between different value representations.

use crate::hir::HirValue;
use crate::mir::types::MirValue;

/// Convert an HIR value to a MIR value
pub fn convert_hir_value(value: &HirValue) -> MirValue {
    match value {
        HirValue::Number(n, _) => MirValue::Number(*n),
        HirValue::Variable(name, _) => MirValue::Variable(name.clone()),
        HirValue::Clone(expr) => convert_hir_value(expr),
        HirValue::Peak(expr) => convert_hir_value(expr),
        HirValue::Binary { left, .. } => {
            // Simple case - return the left value
            // The caller should use process_hir_value for proper binary operation handling
            convert_hir_value(left)
        },
        HirValue::Call { arguments, .. } => {
            // For direct value conversion without generating code,
            // just return the first argument or zero
            arguments.first()
                .map(convert_hir_value)
                .unwrap_or(MirValue::Number(0))
        },
        HirValue::Consume(expr) => convert_hir_value(expr),
    }
}