//! Check the shipped relation against membership in the three disjoint
//! primitive value sets. Float bit patterns include NaNs and signed zero; a
//! representative string unit witnesses every primitive type-tag decision.
mod relation;
use relation::{Primitive, Verdict};

#[derive(kani::Arbitrary)]
enum Literal {
    Boolean(bool),
    Number(u64),
    String(u16),
}

#[kani::proof]
fn literal_membership_is_necessary_and_sufficient() {
    let selector: u8 = kani::any();
    kani::assume(selector < 3);
    let annotation = match selector {
        0 => Primitive::Boolean,
        1 => Primitive::Number,
        _ => Primitive::String,
    };
    let literal: Literal = kani::any();
    let (value_type, member) = match literal {
        Literal::Boolean(value) => {
            let _ = value;
            (Primitive::Boolean, selector == 0)
        }
        Literal::Number(bits) => {
            let _ = bits;
            (Primitive::Number, selector == 1)
        }
        Literal::String(unit) => {
            let _ = unit;
            (Primitive::String, selector == 2)
        }
    };
    assert_eq!(
        annotation.accepts(value_type) == Verdict::Assignable,
        member
    );
}
