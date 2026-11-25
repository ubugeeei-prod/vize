//! Creusot proves that a primitive annotation accepts a literal exactly when
//! the literal's value lies in that primitive's value set, for every literal.
//! This file is the library root of `tests/formal/creusot/assignability` and
//! is not part of `vize_l2`'s module tree: `relation` below is the module the
//! product compiles, found by ordinary module discovery.
mod relation;

pub use relation::{Primitive, Verdict};

use creusot_std::prelude::*;

/// A literal by its value. The three primitive value sets are disjoint:
/// every boolean, every float bit pattern (NaNs and signed zero included)
/// and every string, here represented by one UTF-16 unit.
pub enum Literal {
    /// `true` or `false`.
    Boolean(bool),
    /// The bit pattern of an IEEE 754 double.
    Number(u64),
    /// A string, represented by one of its units.
    String(u16),
}

/// The literal's value lies in the value set of `annotation`.
#[logic(open)]
pub fn member(annotation: Primitive, literal: Literal) -> bool {
    match literal {
        Literal::Boolean(_) => annotation == Primitive::Boolean,
        Literal::Number(_) => annotation == Primitive::Number,
        Literal::String(_) => annotation == Primitive::String,
    }
}

/// The literal type the assignment query reads off a literal expression
/// (`verify.rs`: boolean, numeric and string literals): the primitive whose
/// value set holds the literal's value.
#[ensures(member(result, *literal))]
pub fn literal_type(literal: &Literal) -> Primitive {
    match literal {
        Literal::Boolean(_) => Primitive::Boolean,
        Literal::Number(_) => Primitive::Number,
        Literal::String(_) => Primitive::String,
    }
}

/// Membership is necessary and sufficient: `accepts` returns `Assignable`
/// exactly when the literal's value is in the annotation's value set.
pub fn literal_membership_is_necessary_and_sufficient(annotation: Primitive, literal: Literal) {
    let value_type = literal_type(&literal);
    let verdict = annotation.accepts(value_type);
    proof_assert!((verdict == Verdict::Assignable) == member(annotation, literal));
}
