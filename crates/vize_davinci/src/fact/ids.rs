//! The production fact-group identity table.
//!
//! A group's [`AnalysisId`] is also its bit in every pass's
//! [`Preserved`](crate::pass::Preserved) mask, so ids must be unique across
//! every crate that registers a producer. This table is the single
//! allocation point: a P4-3 wave (or any later provider) takes its group's id
//! from here and adds new ids here, never mints a literal of its own.
//!
//! Ids `0..32` are production groups; ids `32..64` stay free for test and
//! bench fixtures, which never share a registry with production groups.

use crate::pass::AnalysisId;

/// `Bindings` — script bindings and their spans (P4-3a).
pub const BINDINGS: AnalysisId = AnalysisId::new(1);
/// `UndefinedRefs` — template identifiers no binding resolves (P4-3a).
pub const UNDEFINED_REFS: AnalysisId = AnalysisId::new(2);
/// `UnusedBindings` — setup bindings nothing reads (P4-3c).
pub const UNUSED_BINDINGS: AnalysisId = AnalysisId::new(3);
/// `ComponentUsages` — component usages keyed by resolved identity (P4-3b).
pub const COMPONENT_USAGES: AnalysisId = AnalysisId::new(4);
/// `RenderTree` — the cross-file render-tree edges (P4-3b).
pub const RENDER_TREE: AnalysisId = AnalysisId::new(5);
/// `Reactivity` — the S3 reactivity lattice with its verdict axis (P4-3d).
pub const REACTIVITY: AnalysisId = AnalysisId::new(6);
/// `EffectGraph` — effect dependency sets (P4-3e).
pub const EFFECT_GRAPH: AnalysisId = AnalysisId::new(7);
/// `ProvideInject` — provide/inject pairs (P4-3f).
pub const PROVIDE_INJECT: AnalysisId = AnalysisId::new(8);
/// `RaceConditions` — async-setup races (P4-3f).
pub const RACE_CONDITIONS: AnalysisId = AnalysisId::new(9);
/// `HtmlElements` — authored template elements a nesting witness cites (P4-11b).
pub const HTML_ELEMENTS: AnalysisId = AnalysisId::new(10);
/// `HtmlComposedNesting` — nesting proven only in a parent's context (P4-11b).
pub const HTML_COMPOSED_NESTING: AnalysisId = AnalysisId::new(11);
/// `ExpressionFacts` — foreign-dialect expression facts over the P6-1b
/// contract world.
pub const EXPRESSION_FACTS: AnalysisId = AnalysisId::new(12);

/// The first id tests and bench fixtures may use.
pub const FIXTURE_BASE: u8 = 32;

/// Every allocated production id, in id order.
pub const PRODUCTION: [AnalysisId; 12] = [
    BINDINGS,
    UNDEFINED_REFS,
    UNUSED_BINDINGS,
    COMPONENT_USAGES,
    RENDER_TREE,
    REACTIVITY,
    EFFECT_GRAPH,
    PROVIDE_INJECT,
    RACE_CONDITIONS,
    HTML_ELEMENTS,
    HTML_COMPOSED_NESTING,
    EXPRESSION_FACTS,
];

// Unique, ascending, and below the fixture range.
const _: () = {
    let mut i = 0;
    while i < PRODUCTION.len() {
        assert!(PRODUCTION[i].index() < FIXTURE_BASE);
        if i > 0 {
            assert!(PRODUCTION[i - 1].index() < PRODUCTION[i].index());
        }
        i += 1;
    }
};
