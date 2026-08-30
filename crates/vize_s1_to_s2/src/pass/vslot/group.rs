//! Per-component grouping and validation: the legacy `collect_slots`
//! and `validate_v_slot_usage`, re-expressed over the child views the
//! recursion hands each `ui.component`.
//!
//! The two halves keep the legacy lane's two distinct predicates
//! deliberately: **grouping** synthesizes the implicit default from
//! the shared default-slot predicate (a component that owns its own
//! `v-slot` is still content for the parent default slot; a structural
//! slot-template carrier is not), while the **extraneous** diagnostic
//! uses the narrower descending answer the legacy `has_implicit_child`
//! computed. The structural-directive duplicate guard is vacuous here:
//! the carrier sits under `ui.if` / `ui.for` and never reaches this
//! grouping as a `SlotTemplate` child.

use alloc::vec::Vec as StdVec;

use vize_davinci::id::NodeId;
use vize_s0::{Span, String, cstr};
use vize_s2::provenance::ProvenanceRecord;
use vize_s2::scope::ScopeOrigin;

use super::consume::{Channels, ChildKind, ChildView, SlotSpelling};
use super::{
    DUPLICATE_MESSAGE, EXTRANEOUS_MESSAGE, MIXED_MESSAGE, RULE_IMPLICIT_DEFAULT, SlotCarrier,
    SlotFacts, SlotGroup, SlotName, SlotParams,
};

/// Group and validate one `ui.component`.
pub(super) fn component(
    channels: &mut Channels<'_>,
    id: Option<NodeId>,
    name: &str,
    span: Span,
    own: &[SlotSpelling],
    children: &[ChildView],
) {
    validate(channels, id, own, children);
    let groups = collect(channels, id, span, own, children);
    if groups.is_empty() {
        return;
    }
    let mut listing = String::default();
    for group in &groups {
        if !listing.is_empty() {
            listing.push(' ');
        }
        match &group.name {
            SlotName::Static {
                text,
                origin: ScopeOrigin::Synthesized { .. },
            } => listing.push_str(cstr!("+\"{text}\"").as_str()),
            SlotName::Static { text, .. } => listing.push_str(cstr!("\"{text}\"").as_str()),
            SlotName::Dynamic { text } => listing.push_str(cstr!("[{text}]").as_str()),
        }
    }
    channels.provenance.push(ProvenanceRecord {
        rule: String::from("pass.v-slot.group"),
        node: id,
        before: String::from(name),
        after: cstr!("groups {listing}"),
        span,
    });
    if let Some(id) = id {
        channels.facts.insert(id, SlotFacts { groups });
    }
}

/// The legacy `collect_slots`, extended with the component's own
/// spellings (the legacy lane reads those separately at codegen): own
/// groups in spelling order, template groups in child order with
/// duplicate static names dropped silently, then the implicit default
/// when the component has content, no own spelling, and no group named
/// `default`.
fn collect(
    channels: &mut Channels<'_>,
    id: Option<NodeId>,
    span: Span,
    own: &[SlotSpelling],
    children: &[ChildView],
) -> StdVec<SlotGroup> {
    let mut groups: StdVec<SlotGroup> = own
        .iter()
        .map(|spelling| SlotGroup {
            name: spelling.name.clone(),
            params: spelling.params.clone(),
            carrier: SlotCarrier::Component,
        })
        .collect();
    let mut seen_static: StdVec<String> = StdVec::new();
    for child in children {
        let ChildKind::SlotTemplate(slots) = &child.kind else {
            continue;
        };
        for spelling in slots {
            if let SlotName::Static { text, .. } = &spelling.name {
                if seen_static.iter().any(|seen| seen == text) {
                    continue;
                }
                seen_static.push(text.clone());
            }
            groups.push(SlotGroup {
                name: spelling.name.clone(),
                params: spelling.params.clone(),
                carrier: SlotCarrier::Template(child.id),
            });
        }
    }

    let has_content = children.iter().any(|child| {
        matches!(
            child.kind,
            ChildKind::Content {
                default_slot: true,
                ..
            }
        )
    });
    if own.is_empty() && has_content && !groups.iter().any(|group| group.name.text() == "default") {
        channels.provenance.push(ProvenanceRecord {
            rule: String::from(RULE_IMPLICIT_DEFAULT),
            node: id,
            before: String::default(),
            after: String::from("slot \"default\""),
            span,
        });
        groups.push(SlotGroup {
            name: SlotName::Static {
                text: String::from("default"),
                origin: ScopeOrigin::Synthesized {
                    rule: String::from(RULE_IMPLICIT_DEFAULT),
                },
            },
            params: SlotParams::Absent,
            carrier: SlotCarrier::Implicit,
        });
    }
    groups
}

/// The legacy `validate_v_slot_usage`, child views in place of the raw
/// tree. The misplaced check fires at each element's own visit
/// ([`super::consume`]); this half owns the component-level three.
fn validate(
    channels: &mut Channels<'_>,
    id: Option<NodeId>,
    own: &[SlotSpelling],
    children: &[ChildView],
) {
    if !own.is_empty() {
        // Mixed usage: one error at the first template spelling, then
        // stop — the legacy loop breaks there, duplicate and extraneous
        // checks never run.
        for child in children {
            if let ChildKind::SlotTemplate(slots) = &child.kind {
                channels.error(
                    slots[0].span,
                    MIXED_MESSAGE,
                    "error.v-slot-mixed",
                    id,
                    String::default(),
                );
                break;
            }
        }
        return;
    }

    let mut seen: StdVec<&str> = StdVec::new();
    let mut has_template_slots = false;
    let mut has_named_default = false;
    let mut first_implicit: Option<Span> = None;
    for child in children {
        match &child.kind {
            ChildKind::SlotTemplate(slots) => {
                has_template_slots = true;
                // The first spelling per template, exactly the legacy
                // `find_v_slot`; dynamic names skip the check.
                let first = &slots[0];
                if !first.static_name {
                    continue;
                }
                let text = first.name.text();
                if seen.contains(&text) {
                    channels.error(
                        first.span,
                        DUPLICATE_MESSAGE,
                        "error.v-slot-duplicate",
                        id,
                        cstr!("\"{text}\""),
                    );
                    continue;
                }
                if text == "default" {
                    has_named_default = true;
                }
                seen.push(text);
            }
            ChildKind::Content { implicit: true, .. } => {
                if first_implicit.is_none() {
                    first_implicit = Some(child.span);
                }
            }
            ChildKind::Content {
                implicit: false, ..
            }
            | ChildKind::Filler => {}
        }
    }
    if has_template_slots
        && has_named_default
        && let Some(loc) = first_implicit
    {
        channels.error(
            loc,
            EXTRANEOUS_MESSAGE,
            "error.v-slot-extraneous-children",
            id,
            String::default(),
        );
    }
}
