//! Spelling canonicalization for the `v-slot` pass: one authored
//! `ui.slot-content` into its canonical name (the legacy
//! `get_slot_name` — modifier folding, the synthesized default) and its
//! consumed params scope (the vfor consumption pattern at a slot
//! boundary).

use alloc::vec::Vec as StdVec;

use vize_davinci::id::NodeId;
use vize_s0::{String, cstr};
use vize_s2::op::{DynamicName, SlotContentOp};
use vize_s2::provenance::ProvenanceRecord;
use vize_s2::scope::{ScopeBinding, ScopeOrigin};

use super::consume::{Channels, SlotSpelling};
use super::{RULE_DEFAULT_NAME, SlotBound, SlotName, SlotParams};
use crate::lower::simple_identifier;

/// Canonicalize one spelling and consume its scope.
pub(super) fn spelling(
    channels: &mut Channels<'_>,
    id: Option<NodeId>,
    content: &SlotContentOp<'_>,
) -> SlotSpelling {
    let (name, static_name) = canonical_name(channels, id, content);
    let params = consume_scope(channels, id, content);
    SlotSpelling {
        name,
        params,
        span: content.span,
        static_name,
    }
}

/// The canonical name: the legacy `get_slot_name` — static names fold
/// the dialect's modifiers with dots, an unauthored position synthesizes
/// `default` under [`RULE_DEFAULT_NAME`], a dynamic argument ignores
/// modifiers and stays pessimal.
fn canonical_name(
    channels: &mut Channels<'_>,
    id: Option<NodeId>,
    content: &SlotContentOp<'_>,
) -> (SlotName, bool) {
    match &content.name {
        None => {
            let text = fold("default", &content.modifiers);
            channels.provenance.push(ProvenanceRecord {
                rule: String::from(RULE_DEFAULT_NAME),
                node: id,
                // The name position was unauthored — the empty before is
                // the honest spelling of that.
                before: String::default(),
                after: cstr!("name \"{text}\""),
                span: content.span,
            });
            (
                SlotName::Static {
                    text,
                    origin: ScopeOrigin::Synthesized {
                        rule: String::from(RULE_DEFAULT_NAME),
                    },
                },
                true,
            )
        }
        Some(DynamicName::Static(base)) => (
            SlotName::Static {
                text: fold(base, &content.modifiers),
                origin: ScopeOrigin::Authored { span: content.span },
            },
            true,
        ),
        Some(DynamicName::Dynamic(expr)) => (
            SlotName::Dynamic {
                text: String::from(expr.source()),
            },
            false,
        ),
    }
}

/// `base` with the modifiers dot-appended (the shipped
/// `static_slot_name_with_modifiers`).
fn fold(base: &str, modifiers: &[&str]) -> String {
    let mut text = String::from(base);
    for modifier in modifiers {
        text.push('.');
        text.push_str(modifier);
    }
    text
}

/// Consume one spelling's params scope: entry present exactly when
/// params are authored, tag fresh across the artifact, recorded
/// bindings byte-equal with what the params surface derives through the
/// same one scanner (`simple_identifier`, the #4365 discipline).
fn consume_scope(
    channels: &mut Channels<'_>,
    id: Option<NodeId>,
    content: &SlotContentOp<'_>,
) -> SlotParams {
    let Some(expr) = &content.params else {
        // A paramless spelling introduces no scope — the lowering must
        // not have recorded one (precision half of the hygiene law).
        if let Some(id) = id {
            assert!(
                channels.scopes.get(id).is_none(),
                "hygiene law broken: paramless ui.slot-content {id} has a scope entry",
            );
        }
        return SlotParams::Absent;
    };
    // Past id exhaustion the lowering attached no facts; the consumed
    // view is unkeyable anyway (its component's fact is too).
    let Some(id) = id else {
        return SlotParams::Absent;
    };
    let recorded = channels.scopes.get(id).unwrap_or_else(|| {
        panic!(
            "hygiene law broken: ui.slot-content {id} has no scope entry — every params-bearing spelling is an introduction site"
        )
    });
    assert!(
        !channels.seen_tags.contains(&recorded.tag),
        "hygiene law broken: ui.slot-content {id} reuses scope tag {} — introduction sites mint fresh tags",
        recorded.tag,
    );
    channels.seen_tags.push(recorded.tag);

    let bound = simple_identifier(expr);
    let expected: StdVec<ScopeBinding> = bound
        .map(|name| ScopeBinding {
            name: String::from(name),
            origin: ScopeOrigin::Authored { span: expr.span() },
        })
        .into_iter()
        .collect();
    assert!(
        recorded.bindings == expected,
        "hygiene law broken: ui.slot-content {id} recorded bindings {:?} but its params surface derives {:?}",
        recorded.bindings,
        expected,
    );

    let name = match bound {
        Some(name) => SlotBound::Named(String::from(name)),
        None => SlotBound::Pending,
    };
    channels.provenance.push(ProvenanceRecord {
        rule: String::from("pass.v-slot.scope"),
        node: Some(id),
        before: cstr!(
            "scope {} bindings={}",
            recorded.tag,
            recorded.bindings.len()
        ),
        after: cstr!(
            "fact params={}",
            match &name {
                SlotBound::Named(name) => name.as_str(),
                SlotBound::Pending => "?",
            }
        ),
        span: content.span,
    });
    SlotParams::Scoped {
        text: String::from(expr.source()),
        tag: recorded.tag,
        name,
    }
}
