//! Hygiene scope facts: the side-table vocabulary for binding-introducing
//! ops (P2-8).
//!
//! # The imported practice
//!
//! `davinci-road/prior-art-toolchains.md` (Lean 4, metaprogramming
//! anti-lesson): dialect lowerings that synthesize identifiers (slot
//! props, `v-for` scopes) need **hygiene-style scope tagging so
//! synthesized names can't capture author bindings**. The mechanism here
//! is the classic one reduced to what S2 needs:
//!
//! - Every binding-introducing op (a `ui.for`, a slot-props position)
//!   gets a fresh [`ScopeTag`], minted densely by the lowering artifact.
//!   Names are compared as `(name, tag)` pairs, never as bare text, so
//!   two scopes spelling the same name never collide.
//! - Every introduced binding carries a [`ScopeOrigin`]: `Authored` with
//!   the author's span, or `Synthesized` with the rule that minted it. A
//!   later pass can therefore never confuse a synthesized name with an
//!   author's - the origin is a recorded fact, not a naming convention.
//! - A pass that synthesizes a new name **must** mint a fresh tag for it
//!   and record it as [`ScopeOrigin::Synthesized`]; re-using an authored
//!   scope's tag for a synthesized name is the capture bug the tag
//!   exists to prevent.
//!
//! # Residency
//!
//! Facts live beside the tree in a
//! [`SideTable<ScopeFacts>`](vize_davinci::side_table::SideTable) keyed by
//! the introducing op's [`NodeId`](vize_davinci::id::NodeId) (dense
//! page-order ids, `folio-format.md` "Node numbering"). The table is the
//! sparse non-arena form, so names are owned [`String`]s and the facts are
//! `'static` - they survive `Allocator::reset` the way diagnostics do
//! (P1-11), which is what keeps the LSP live on a broken SFC.
//!
//! # What P2-8 does and does not enumerate
//!
//! The Vue lowering records a binding entry only for a position whose
//! expression is a **simple identifier**. Names bound inside destructuring
//! patterns (`(({ a, b })) in xs`) are deliberately not enumerated here:
//! identifier extraction has exactly one home, the
//! `ExprDialect::enumerate_bindings` seam ([#4365]), and a second ad-hoc
//! scanner in this module would recreate the split P1-8 deleted. A position
//! without an entry still has its scope tag - the *scope* is a fact even where
//! the *names* wait.
//!
//! [#4365]: https://github.com/ubugeeei-prod/vize/issues/4365

use alloc::vec::Vec;

use vize_s0::{Span, String};

/// One binding-introduction site, minted densely per artifact.
///
/// The tag is what makes hygiene mechanical: identity is `(name, tag)`,
/// so a name synthesized under a fresh tag can never capture an author
/// binding, whatever it is spelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeTag(u32);

impl ScopeTag {
    /// The tag for the `index`-th introduction site of an artifact.
    #[inline]
    #[must_use]
    pub const fn from_index(index: u32) -> Self {
        Self(index)
    }

    /// The 0-based mint index.
    #[inline]
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

impl core::fmt::Display for ScopeTag {
    /// Prints `#N`, the folio spelling of a scope tag.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// Where a binding's name came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeOrigin {
    /// The author wrote the name; `span` is where.
    Authored {
        /// The authored range of the name.
        span: Span,
    },
    /// A lowering or pass minted the name; `rule` names the decision (the
    /// same rule-name vocabulary provenance records use).
    Synthesized {
        /// The rule that synthesized the name.
        rule: String,
    },
}

/// One binding a scope introduces into its op's region(s).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeBinding {
    /// The bound name.
    pub name: String,
    /// Authored or synthesized (the hygiene fact).
    pub origin: ScopeOrigin,
}

/// The scope facts of one binding-introducing op.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeFacts {
    /// The op's introduction-site tag.
    pub tag: ScopeTag,
    /// The bindings it introduces, in position order (for `ui.for`:
    /// value, key, index). Positions holding patterns contribute no entry
    /// (see the module docs for the [#4365] boundary).
    ///
    /// [#4365]: https://github.com/ubugeeei-prod/vize/issues/4365
    pub bindings: Vec<ScopeBinding>,
}

/// Scope facts cross compile boundaries with their artifact (the P1-11
/// contract, the same enforcement `Diagnostic` carries).
const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<ScopeFacts>();
    assert_owned::<ScopeBinding>();
    assert_owned::<ScopeOrigin>();
};

/// The tag is a bare index on every target; the owning types are 64-bit
/// figures of pointer-containing structs, so they carry the `vize_relief`
/// guard (see `crate::op`).
const _: () = assert!(core::mem::size_of::<ScopeTag>() == 4);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<ScopeOrigin>() == 24);
    assert!(core::mem::size_of::<ScopeBinding>() == 48);
    assert!(core::mem::size_of::<ScopeFacts>() == 32);
};

#[cfg(test)]
mod tests {
    use super::{ScopeBinding, ScopeFacts, ScopeOrigin, ScopeTag};
    use alloc::vec;
    use vize_s0::{Span, String, cstr};

    #[test]
    fn tags_mint_densely_and_order_by_index() {
        let first = ScopeTag::from_index(0);
        let second = ScopeTag::from_index(1);
        assert_eq!(first.index(), 0);
        assert_eq!(second.index(), 1);
        assert!(first < second);
        assert_eq!(cstr!("{first}").as_str(), "#0");
    }

    #[test]
    fn a_synthesized_name_never_equals_an_authored_one() {
        // The hygiene law in miniature: identical spelling, different
        // facts. Consumers compare (name, tag) and read the origin; the
        // types make both differences representable and recorded.
        let authored = ScopeBinding {
            name: String::from("item"),
            origin: ScopeOrigin::Authored {
                span: Span::new(14, 18),
            },
        };
        let synthesized = ScopeBinding {
            name: String::from("item"),
            origin: ScopeOrigin::Synthesized {
                rule: String::from("normalize.slot-props"),
            },
        };
        assert_eq!(authored.name, synthesized.name);
        assert_ne!(authored, synthesized);
    }

    #[test]
    fn facts_hold_bindings_in_position_order() {
        let facts = ScopeFacts {
            tag: ScopeTag::from_index(2),
            bindings: vec![
                ScopeBinding {
                    name: String::from("item"),
                    origin: ScopeOrigin::Authored {
                        span: Span::new(10, 14),
                    },
                },
                ScopeBinding {
                    name: String::from("index"),
                    origin: ScopeOrigin::Authored {
                        span: Span::new(16, 21),
                    },
                },
            ],
        };
        assert_eq!(facts.tag, ScopeTag::from_index(2));
        assert_eq!(facts.bindings.len(), 2);
        assert_eq!(facts.bindings[0].name.as_str(), "item");
        assert_eq!(facts.bindings[1].name.as_str(), "index");
    }
}
