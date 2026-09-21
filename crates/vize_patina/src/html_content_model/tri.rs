//! Three-valued truth: the `proven / refuted / unknown` shape every
//! conformance fact takes (assurance.md, "Three-valued facts, fire-on-proof").

/// A three-valued fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// Lift a two-valued fact.
    #[inline]
    pub const fn from_bool(value: bool) -> Self {
        if value { Self::Yes } else { Self::No }
    }

    /// Kleene negation.
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
