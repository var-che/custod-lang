use crate::symbol_table::Span;
use crate::token::Token;

/// Trait for types that can provide source location information
pub trait HasSourceLocation {
    fn get_span(&self) -> Span;
}

impl HasSourceLocation for Token {
    fn get_span(&self) -> Span {
        Span::new(
            self.line,
            self.column,
            self.line,
            self.column + self.length - 1
        )
    }
}

/// Extension trait for types with source locations
pub trait SourceLocationExt {
    fn combine_spans<T: HasSourceLocation>(&self, other: &T) -> Span;
}

impl<T: HasSourceLocation> SourceLocationExt for T {
    fn combine_spans<U: HasSourceLocation>(&self, other: &U) -> Span {
        self.get_span().combine(&other.get_span())
    }
}
