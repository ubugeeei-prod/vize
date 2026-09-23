//! Partial S2 facts for a template that still has S1 holes (P5-10).
//!
//! P2-8 already lowers through `Unexpected` and `Missing` holes and keeps
//! the surrounding ops. This module hands those kept fragments to the fact
//! manager. Facts are computed for every fragment a hole does not cover.
//! A cursor sitting in an `Unexpected` hole reads nothing. Expression
//! interiors are not recovered: a `v-for` alias is a fact only when the
//! lowering already recorded it as a simple identifier.

mod facts;
mod page;

use alloc::vec::Vec;

use vize_davinci::fact::FactManager;
use vize_s0::{Allocator, Span, String};
use vize_s1::{SurfaceTree, parse};

use crate::lower::{Lowered, lower};

use facts::{PartialHoles, PartialRegions, PartialScopes, REGISTRY, TemplateFacts};
use page::collect;

/// An S1 hole that a kept fragment was lowered beside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoleFact {
    /// `Unexpected` bytes, or a `Missing` token / end tag.
    pub kind: HoleKind,
    /// The hole's range in the lowering's source. `Missing` is often empty.
    pub span: Span,
}

/// Which hole policy clause a [`HoleFact`] records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoleKind {
    /// `SurfaceChild::Unexpected` — bytes with no structural home.
    Unexpected,
    /// A `Missing` token, or an `ElementClose::Missing` hole.
    Missing,
}

/// One kept S2 op the fact manager accepted as a well-formed region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegionFact {
    /// The op's source range.
    pub span: Span,
    /// What kind of op was kept.
    pub kind: FragmentKind,
}

/// The op kinds a partial page distinguishes. Bindings are the attached
/// ops the page-order walk numbers between an owner and its children.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FragmentKind {
    /// `ui.element`.
    Element,
    /// `ui.component`.
    Component,
    /// `ui.text`.
    Text,
    /// `ui.interpolation`.
    Interpolation,
    /// `ui.comment`.
    Comment,
    /// `ui.if`.
    If,
    /// `ui.for`.
    For,
    /// `ui.slot`.
    Slot,
    /// An attached binding op (`ui.bind`, `ui.slot-content`, …).
    Binding,
}

/// Where a scope name sits in its introducing fragment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeRole {
    /// The `v-for` value alias.
    Value,
    /// The `v-for` key alias.
    Key,
    /// The `v-for` index alias.
    Index,
    /// A slot-prop or `slot-scope` simple identifier.
    Slot,
}

/// Key of one scope name: the introducing op, then the name's position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScopeKey {
    /// Page-order id of the introducing op.
    pub node: u32,
    /// Position in that op's scope (value, then key, then index).
    pub index: u16,
}

/// One scope name computed for a well-formed fragment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeNameFact {
    /// The bound name.
    pub name: String,
    /// Where the name was authored. Empty for a synthesized name.
    pub name_span: Span,
    /// The introducing fragment's span — where the name is visible.
    pub visible: Span,
    /// Value, key, index, or slot prop.
    pub role: ScopeRole,
}

/// The holes, regions and scope names of one partial page.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PartialFacts {
    holes: Vec<(u32, HoleFact)>,
    regions: Vec<(u32, RegionFact)>,
    scopes: Vec<(ScopeKey, ScopeNameFact)>,
}

/// Facts for `source`, including when it contains S1 holes.
#[must_use]
pub fn partial_facts_of(source: &str) -> PartialFacts {
    let allocator = Allocator::new();
    let (tree, errors) = parse(&allocator, source);
    let mut lowered = lower(&allocator, &tree, &errors);
    partial_facts(&mut lowered, &tree)
}

/// Facts for an already-lowered page. `tree` is the S1 tree `lowered` came
/// from; hole spans are measured against `lowered.source`.
#[must_use]
pub fn partial_facts(lowered: &mut Lowered<'_>, tree: &SurfaceTree<'_>) -> PartialFacts {
    let page = collect(lowered, tree);
    let mut manager = FactManager::new(&REGISTRY);
    // The registry is closed over its groups, so these lookups succeed;
    // should they ever not, the page reports no facts instead of aborting.
    let Ok(view) = manager.prepare::<TemplateFacts>(&page) else {
        return PartialFacts::default();
    };
    let (Ok(holes), Ok(regions), Ok(scopes)) = (
        view.get::<PartialHoles>(),
        view.get::<PartialRegions>(),
        view.get::<PartialScopes>(),
    ) else {
        return PartialFacts::default();
    };
    PartialFacts {
        holes: holes.iter().map(|(key, hole)| (*key, *hole)).collect(),
        regions: regions
            .iter()
            .map(|(key, region)| (*key, *region))
            .collect(),
        scopes: scopes
            .iter()
            .map(|(key, scope)| (*key, scope.clone()))
            .collect(),
    }
}

impl PartialFacts {
    /// Holes in source order.
    #[must_use]
    pub fn holes(&self) -> &[(u32, HoleFact)] {
        &self.holes
    }

    /// Well-formed fragments, in page-order id order.
    #[must_use]
    pub fn regions(&self) -> &[(u32, RegionFact)] {
        &self.regions
    }

    /// Scope names of well-formed fragments, in `(node, index)` order.
    #[must_use]
    pub fn scopes(&self) -> &[(ScopeKey, ScopeNameFact)] {
        &self.scopes
    }

    /// Whether `offset` sits in a non-empty `Unexpected` hole.
    #[must_use]
    pub fn in_unexpected(&self, offset: u32) -> bool {
        self.holes
            .iter()
            .any(|(_, hole)| hole.kind == HoleKind::Unexpected && contains(hole.span, offset))
    }

    /// The innermost scope name `name` visible at `offset`.
    #[must_use]
    pub fn binding_at(&self, offset: u32, name: &str) -> Option<&ScopeNameFact> {
        if self.in_unexpected(offset) {
            return None;
        }
        self.scopes
            .iter()
            .filter(|(_, fact)| fact.name.as_str() == name && contains(fact.visible, offset))
            .min_by_key(|(_, fact)| fact.visible.len())
            .map(|(_, fact)| fact)
    }

    /// Scope names visible at `offset`, innermost shadowing an outer one.
    #[must_use]
    pub fn visible_names(&self, offset: u32) -> Vec<&str> {
        if self.in_unexpected(offset) {
            return Vec::new();
        }
        let mut rows: Vec<&ScopeNameFact> = self
            .scopes
            .iter()
            .filter(|(_, fact)| contains(fact.visible, offset))
            .map(|(_, fact)| fact)
            .collect();
        rows.sort_by_key(|fact| (fact.visible.len(), fact.name_span.start));
        let mut names = Vec::new();
        for fact in rows {
            if names.iter().any(|seen| *seen == fact.name.as_str()) {
                continue;
            }
            names.push(fact.name.as_str());
        }
        names
    }
}

fn contains(span: Span, offset: u32) -> bool {
    offset >= span.start && offset < span.end
}
