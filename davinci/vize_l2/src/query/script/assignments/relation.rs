//! Assignability of primitive literal types to their respective broad types.
//!
//! Creusot proves `accepts` against membership in the three disjoint
//! primitive value sets (`tests/formal/creusot/assignability`, root
//! `creusot.rs`). Under `cfg(creusot)` only `Clone` and `Copy` are derived;
//! the product build keeps the original derive list.
#[cfg(creusot)]
use creusot_std::prelude::{Clone, ensures};

#[cfg_attr(not(creusot), derive(Debug, Clone, Copy, PartialEq, Eq))]
#[cfg_attr(creusot, derive(Clone, Copy))]
pub enum Primitive {
    Boolean,
    Number,
    String,
}

#[cfg_attr(not(creusot), derive(Debug, Clone, Copy, PartialEq, Eq))]
#[cfg_attr(creusot, derive(Clone, Copy))]
pub enum Verdict {
    Assignable,
    Mismatch,
}

impl Primitive {
    /// A literal type is assignable to its own broad type and to no other.
    #[cfg_attr(creusot, ensures((result == Verdict::Assignable) == (self == value)))]
    pub const fn accepts(self, value: Self) -> Verdict {
        match (self, value) {
            (Self::Boolean, Self::Boolean)
            | (Self::Number, Self::Number)
            | (Self::String, Self::String) => Verdict::Assignable,
            _ => Verdict::Mismatch,
        }
    }

    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Number => "number",
            Self::String => "string",
        }
    }
}
