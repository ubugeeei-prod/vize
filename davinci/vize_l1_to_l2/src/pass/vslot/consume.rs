//! Page-order `v-slot` recursion: canonicalize spellings, consume scopes,
//! and summarize child views for [`super::group`]. Numbering follows the
//! shared [`PageWalk`](super::super::walk::PageWalk): op, attached bindings,
//! then children. [`super::run`] checks the lowering's minted id accounting.

use alloc::vec::Vec as StdVec;

use vize_l0::diag::{Diagnostic, Stage};
use vize_l0::id::NodeId;
use vize_l0::side_table::SideTable;
use vize_l0::{Span, String, ensure_sufficient_stack};
use vize_l2::op::{BindingOp, Op};
use vize_l2::provenance::ProvenanceRecord;
use vize_l2::scope::{ScopeFacts, ScopeTag};

use super::super::walk::PageWalk;
use super::spell::spelling;
use super::{MISPLACED_MESSAGE, SlotFacts, SlotName, SlotParams, group};
use crate::lower::{ForWrapper, WrapperKeys};

/// The pass's channels: the read-only scope table, the unified output
/// channels, and the pass state.
pub(super) struct Channels<'l> {
    pub scopes: &'l SideTable<ScopeFacts>,
    pub wrappers: &'l SideTable<WrapperKeys>,
    pub for_wrappers: &'l SideTable<ForWrapper>,
    pub diagnostics: &'l mut StdVec<Diagnostic>,
    pub provenance: &'l mut StdVec<ProvenanceRecord>,
    /// Every consumed slot-scope tag, for the freshness law.
    pub seen_tags: StdVec<ScopeTag>,
    /// The published facts, keyed by `ui.component` id.
    pub facts: SideTable<SlotFacts>,
}

impl Channels<'_> {
    /// One user error plus its provenance record, the vif shape.
    pub(super) fn error(
        &mut self,
        span: Span,
        message: &'static str,
        rule: &'static str,
        node: Option<NodeId>,
        before: String,
    ) {
        self.diagnostics.push(Diagnostic::legacy_error(
            &crate::exemptions::V_SLOT,
            Stage::Semantic,
            span,
            String::from(message),
        ));
        self.provenance.push(ProvenanceRecord {
            rule: String::from(rule),
            node,
            before,
            after: String::default(),
            span,
        });
    }
}

/// One canonicalized `v-slot` spelling, as the grouping consumes it.
#[derive(Debug)]
pub(super) struct SlotSpelling {
    /// The canonical name (modifiers folded, origin recorded).
    pub name: SlotName,
    /// The consumed params view.
    pub params: SlotParams,
    /// The spelling's span (the diagnostic anchor).
    pub span: Span,
    /// Whether the name position is static-or-implicit (the legacy
    /// `slot_name_is_static`, gating the duplicate check).
    pub static_name: bool,
}

/// What one region op contributes to its parent's slot view.
#[derive(Debug)]
pub(super) struct ChildView {
    /// The op's page-order id.
    pub id: Option<NodeId>,
    /// The op's span.
    pub span: Span,
    /// The classification.
    pub kind: ChildKind,
}

/// The slot-relevant classification of one child op.
#[derive(Debug)]
pub(super) enum ChildKind {
    /// `ui.element template` carrying at least one `ui.slot-content`.
    SlotTemplate(StdVec<SlotSpelling>),
    /// Whitespace-only text — never implicit-default content.
    Filler,
    /// Other content. `implicit` is false for slot-carrying non-template
    /// owners, true otherwise, and recursive for `ui.if`/`ui.for` (the
    /// legacy `has_implicit_child`). `default_slot` includes components
    /// carrying their own `v-slot`, but excludes structural slot templates.
    Content {
        /// Whether the child counts for the extraneous diagnostic.
        implicit: bool,
        /// Whether the child synthesizes the parent default slot group.
        default_slot: bool,
    },
}

fn implicit_default_content(id: Option<NodeId>, span: Span) -> ChildView {
    ChildView {
        id,
        span,
        kind: ChildKind::Content {
            implicit: true,
            default_slot: true,
        },
    }
}

/// The legacy `any_implicit_child`, including unwrapped nested `#slot`
/// wrappers. Kept structural slot carriers remain explicit `createSlots`
/// inputs; unwrapped wrappers flatten onto the default slot.
fn region_implicit(views: &[ChildView]) -> bool {
    let mut slot_templates = 0usize;
    for view in views {
        match &view.kind {
            ChildKind::Content { implicit: true, .. } => return true,
            ChildKind::SlotTemplate(_) => slot_templates += 1,
            ChildKind::Content {
                implicit: false, ..
            }
            | ChildKind::Filler => {}
        }
    }
    slot_templates > 1
}

fn has_slot_template(views: &[ChildView]) -> bool {
    views
        .iter()
        .any(|view| matches!(view.kind, ChildKind::SlotTemplate(_)))
}

fn region_default_slot(views: &[ChildView]) -> bool {
    views.iter().any(|view| {
        matches!(
            view.kind,
            ChildKind::Content {
                default_slot: true,
                ..
            }
        )
    })
}

fn branch_from_template(wrapper: Option<&WrapperKeys>, branch_index: usize) -> bool {
    wrapper
        .and_then(|keys| keys.from_template.get(branch_index).copied())
        .unwrap_or(false)
}

fn if_branch_implicit(
    wrapper: Option<&WrapperKeys>,
    branch_index: usize,
    views: &[ChildView],
) -> bool {
    region_implicit(views)
        || (branch_from_template(wrapper, branch_index) && has_slot_template(views))
}

fn if_branch_default_slot(
    wrapper: Option<&WrapperKeys>,
    branch_index: usize,
    views: &[ChildView],
) -> bool {
    region_default_slot(views)
        || (branch_from_template(wrapper, branch_index) && has_slot_template(views))
}

fn for_region_implicit(from_template: bool, views: &[ChildView]) -> bool {
    region_implicit(views) || (from_template && has_slot_template(views))
}

fn for_region_default_slot(from_template: bool, views: &[ChildView]) -> bool {
    region_default_slot(views) || (from_template && has_slot_template(views))
}

/// Visit one region's ops in page order, returning their views.
pub(super) fn region<'a>(
    walk: &mut PageWalk,
    channels: &mut Channels<'_>,
    ops: &[Op<'a>],
) -> StdVec<ChildView> {
    ensure_sufficient_stack(|| {
        let mut views = StdVec::with_capacity(ops.len());
        for op in ops {
            views.push(visit(walk, channels, op));
        }
        views
    })
}

fn visit<'a>(walk: &mut PageWalk, channels: &mut Channels<'_>, op: &Op<'a>) -> ChildView {
    let id = walk.mint();
    match op {
        Op::Element(element) => {
            let slots = bindings(walk, channels, &element.bindings);
            let children = region(walk, channels, &element.children.ops);
            let _ = children;
            let kind = if element.tag == "template" && !slots.is_empty() {
                ChildKind::SlotTemplate(slots)
            } else {
                if let Some(first) = slots.first() {
                    // The legacy `VSlotMisplaced`, fired once per element
                    // at the first spelling, exactly as `find_v_slot`
                    // anchored it.
                    channels.error(
                        first.span,
                        MISPLACED_MESSAGE,
                        "error.v-slot-misplaced",
                        id,
                        String::default(),
                    );
                }
                // A slot-carrying non-template element never anchors the
                // extraneous diagnostic (the legacy loop skips it), but
                // it is content for the implicit default.
                ChildKind::Content {
                    implicit: slots.is_empty(),
                    default_slot: true,
                }
            };
            ChildView {
                id,
                span: element.span,
                kind,
            }
        }
        Op::Component(component) => {
            let own = bindings(walk, channels, &component.bindings);
            let children = region(walk, channels, &component.children.ops);
            let implicit = own.is_empty();
            group::component(
                channels,
                id,
                component.name,
                component.span,
                &own,
                &children,
            );
            ChildView {
                id,
                span: component.span,
                kind: ChildKind::Content {
                    implicit,
                    default_slot: true,
                },
            }
        }
        Op::Text(text) if crate::lower::legacy_slot_filler_text(text.content) => ChildView {
            id,
            span: text.span,
            kind: ChildKind::Filler,
        },
        Op::Text(text) => implicit_default_content(id, text.span),
        Op::Interpolation(interpolation) => implicit_default_content(id, interpolation.span),
        Op::Comment(comment) => implicit_default_content(id, comment.span),
        Op::If(if_op) => {
            let wrapper = id.and_then(|id| channels.wrappers.get(id));
            let mut implicit = false;
            let mut default_slot = false;
            for (branch_index, branch) in if_op.branches.iter().enumerate() {
                let views = region(walk, channels, &branch.region.ops);
                implicit |= if_branch_implicit(wrapper, branch_index, &views);
                default_slot |= if_branch_default_slot(wrapper, branch_index, &views);
            }
            ChildView {
                id,
                span: if_op.span,
                kind: ChildKind::Content {
                    implicit,
                    default_slot,
                },
            }
        }
        Op::For(for_op) => {
            let views = region(walk, channels, &for_op.region.ops);
            let from_template = id.and_then(|id| channels.for_wrappers.get(id)).is_some();
            ChildView {
                id,
                span: for_op.span,
                kind: ChildKind::Content {
                    implicit: for_region_implicit(from_template, &views),
                    default_slot: for_region_default_slot(from_template, &views),
                },
            }
        }
        Op::OriginalFor(for_op) => {
            let views = region(walk, channels, &for_op.region.ops);
            ChildView {
                id,
                span: for_op.span,
                kind: ChildKind::Content {
                    implicit: for_region_implicit(false, &views),
                    default_slot: for_region_default_slot(false, &views),
                },
            }
        }
        Op::Slot(slot) => {
            // The outlet's binding surface (P2-9 series 5): binding ids
            // mint in page order, and a `v-slot` spelled on the outlet
            // is the legacy `VSlotMisplaced` — a `<slot>` is neither a
            // component nor a `<template>` carrier.
            let slots = bindings(walk, channels, &slot.bindings);
            if let Some(first) = slots.first() {
                channels.error(
                    first.span,
                    MISPLACED_MESSAGE,
                    "error.v-slot-misplaced",
                    id,
                    String::default(),
                );
            }
            let _views = region(walk, channels, &slot.fallback.ops);
            ChildView {
                id,
                span: slot.span,
                kind: ChildKind::Content {
                    implicit: true,
                    default_slot: true,
                },
            }
        }
    }
}

/// Mint every attached binding's id in order, canonicalizing the
/// `ui.slot-content` ones.
fn bindings<'a>(
    walk: &mut PageWalk,
    channels: &mut Channels<'_>,
    list: &[BindingOp<'a>],
) -> StdVec<SlotSpelling> {
    let mut slots = StdVec::new();
    for binding in list {
        let id = walk.mint();
        if let BindingOp::SlotContent(content) = binding {
            slots.push(spelling(channels, id, content));
        }
    }
    slots
}
