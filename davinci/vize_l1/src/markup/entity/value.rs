//! Decoded value of one authored character reference.

/// Decoded scalars of one character reference without allocating per token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodedEntity {
    /// Named references use the static WHATWG table and may expand to multiple scalars.
    Named(&'static str),
    /// Numeric references expand to one corrected Unicode scalar.
    Numeric(char),
}

impl DecodedEntity {
    /// Visit every scalar in the expansion in source order.
    pub fn for_each(self, mut visit: impl FnMut(char)) {
        match self {
            Self::Named(value) => value.chars().for_each(visit),
            Self::Numeric(value) => visit(value),
        }
    }
}
