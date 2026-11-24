//! Three-valued truth: the `proven / refuted / unknown` shape every
//! conformance fact takes (assurance.md, "Three-valued facts, fire-on-proof").
//!
//! Each connective carries its law as a Creusot contract
//! (`cfg_attr(creusot, ..)`, see `creusot.rs` beside this file): it is sound
//! for every concrete value its operands admit, and it is Kleene's strong
//! connective, the most definite sound answer.

#[cfg(creusot)]
use creusot_std::prelude::{Clone, DeepModel, PartialEq, ensures, logic};

/// A three-valued fact.
///
/// Under `cfg(creusot)` the derives that are not part of the laws (`Debug`
/// and `Hash`) are left out of the proof crate; the product build keeps the
/// original derive list, so it expands to exactly the code it did before the
/// proofs.
#[cfg_attr(not(creusot), derive(Debug, Clone, Copy, PartialEq, Eq, Hash))]
#[cfg_attr(creusot, derive(Clone, Copy, PartialEq, Eq, DeepModel))]
pub enum Tri {
    /// Refuted in every context consistent with what is known.
    No,
    /// Depends on something unknown (an unresolved component, a dynamic
    /// binding, the context a template is mounted into).
    Maybe,
    /// Proven in every context consistent with what is known.
    Yes,
}

impl Tri {
    /// Whether the fact is consistent with the concrete truth value
    /// `concrete`: a proven fact admits only `true`, a refuted one only
    /// `false`, and an unknown one both.
    #[cfg(creusot)]
    #[logic(open)]
    pub fn admits(self, concrete: bool) -> bool {
        match self {
            Self::Yes => concrete,
            Self::No => !concrete,
            Self::Maybe => true,
        }
    }

    /// Lift a two-valued fact.
    #[cfg_attr(creusot, ensures(forall<concrete: bool> result.admits(concrete) == (concrete == value)))]
    #[inline]
    pub const fn from_bool(value: bool) -> Self {
        if value { Self::Yes } else { Self::No }
    }

    /// Kleene negation.
    #[cfg_attr(creusot, ensures(forall<a: bool> self.admits(a) ==> result.admits(!a)))]
    #[cfg_attr(creusot, ensures((result == Tri::Yes) == (self == Tri::No)))]
    #[cfg_attr(creusot, ensures((result == Tri::No) == (self == Tri::Yes)))]
    #[inline]
    #[must_use]
    pub const fn not(self) -> Self {
        match self {
            Self::No => Self::Yes,
            Self::Maybe => Self::Maybe,
            Self::Yes => Self::No,
        }
    }

    /// Kleene conjunction.
    #[cfg_attr(creusot, ensures(forall<a: bool, b: bool>
        self.admits(a) && other.admits(b) ==> result.admits(a && b)))]
    #[cfg_attr(creusot, ensures((result == Tri::Yes) == (self == Tri::Yes && other == Tri::Yes)))]
    #[cfg_attr(creusot, ensures((result == Tri::No) == (self == Tri::No || other == Tri::No)))]
    #[inline]
    #[must_use]
    pub const fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::No, _) | (_, Self::No) => Self::No,
            (Self::Yes, Self::Yes) => Self::Yes,
            _ => Self::Maybe,
        }
    }

    /// Kleene disjunction.
    #[cfg_attr(creusot, ensures(forall<a: bool, b: bool>
        self.admits(a) && other.admits(b) ==> result.admits(a || b)))]
    #[cfg_attr(creusot, ensures((result == Tri::Yes) == (self == Tri::Yes || other == Tri::Yes)))]
    #[cfg_attr(creusot, ensures((result == Tri::No) == (self == Tri::No && other == Tri::No)))]
    #[inline]
    #[must_use]
    pub const fn or(self, other: Self) -> Self {
        match (self, other) {
            (Self::Yes, _) | (_, Self::Yes) => Self::Yes,
            (Self::No, Self::No) => Self::No,
            _ => Self::Maybe,
        }
    }

    /// The least upper bound in the information order: agreeing facts stay,
    /// disagreeing facts become unknown.
    #[cfg_attr(creusot, ensures(forall<a: bool>
        self.admits(a) || other.admits(a) ==> result.admits(a)))]
    #[cfg_attr(creusot, ensures(result == if self == other { self } else { Tri::Maybe }))]
    #[inline]
    #[must_use]
    pub const fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::Yes, Self::Yes) => Self::Yes,
            (Self::No, Self::No) => Self::No,
            _ => Self::Maybe,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Tri::{self, Maybe, No, Yes};

    const ALL: [Tri; 3] = [No, Maybe, Yes];

    /// Exact truth tables, rows and columns in `ALL` order.
    #[test]
    fn truth_tables_are_kleene() {
        let table = |op: fn(Tri, Tri) -> Tri| ALL.map(|l| ALL.map(|r| op(l, r)));
        assert_eq!(ALL.map(Tri::not), [Yes, Maybe, No]);
        assert_eq!(
            table(Tri::and),
            [[No, No, No], [No, Maybe, Maybe], [No, Maybe, Yes]]
        );
        assert_eq!(
            table(Tri::or),
            [[No, Maybe, Yes], [Maybe, Maybe, Yes], [Yes, Yes, Yes]]
        );
        assert_eq!(
            table(Tri::join),
            [
                [No, Maybe, Maybe],
                [Maybe, Maybe, Maybe],
                [Maybe, Maybe, Yes]
            ]
        );
        assert_eq!([false, true].map(Tri::from_bool), [No, Yes]);
    }
}
