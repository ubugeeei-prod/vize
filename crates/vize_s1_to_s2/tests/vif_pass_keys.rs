//! The `v-if` pass, series-5 half: the key arms the element/binding
//! installment landed — dynamic `:key` extraction, wrapper-key folding,
//! authored-order precedence, and the outlet surface — split from
//! `vif_pass.rs` under the source budget. The series-1 pins stay there.

mod support;

use vize_davinci::id::NodeId;
use vize_s0::Span;
use vize_s1_to_s2::pass::{BranchKey, BranchKeyKind, vif};
use vize_s2::folio::{DisegnoFolio, FolioAttribute, FolioElement, FolioOp};

use support::{assert_transformed_sound, with_transformed};

/// The single root element of branch `index` of the `chain`-th root op
/// (the `vif_pass.rs` helper, repeated test-file-locally like the span
/// helper below).
fn branch_root(folio: &DisegnoFolio, chain: usize, index: usize) -> &FolioElement {
    let FolioOp::If(if_op) = &folio.ops[chain] else {
        panic!("root op {chain} is not ui.if");
    };
    match &if_op.branches[index].ops[..] {
        [FolioOp::Element(element)] => element,
        other => panic!("branch root is not one element: {other:?}"),
    }
}

/// The span of the `occurrence`-th `needle` in `source` (0-based), as
/// the lowering records attribute spans.
fn span_of(source: &str, needle: &str, occurrence: usize) -> Span {
    let mut from = 0usize;
    for _ in 0..occurrence {
        from = source[from..].find(needle).expect("occurrence exists") + from + needle.len();
    }
    let start = source[from..].find(needle).expect("occurrence exists") + from;
    Span::new(
        u32::try_from(start).expect("fixture fits u32"),
        u32::try_from(start + needle.len()).expect("fixture fits u32"),
    )
}

#[test]
fn a_template_wrapper_key_folds_into_the_branch_fact() {
    // Installment 1 recorded the wrapper key as a series gap; series 5
    // closed it — the lowering captures the wrapper's key before the
    // unwrap drops the rest, and this pass folds it into the fact.
    let source = r#"<template v-if="a" key="k"><p>x</p><p>y</p></template>"#;
    with_transformed(source, |lowered, _, facts, _| {
        assert_eq!(
            facts.if_facts.sorted_entries(),
            vec![(
                NodeId::from_index(0).expect("op 0 has an id"),
                &vif::IfFacts {
                    branches: vec![Some(BranchKey {
                        kind: BranchKeyKind::Static(Some("k".into())),
                        span: span_of(source, r#"key="k""#, 0),
                    })],
                }
            )]
        );
        let template_drops: Vec<&str> = lowered
            .provenance
            .iter()
            .filter(|record| record.rule.as_str() == "drop.template-attribute")
            .map(|record| record.before.as_str())
            .collect();
        assert_eq!(
            template_drops,
            Vec::<&str>::new(),
            "the captured key is not a drop (and the wrapper had nothing else)"
        );
        assert!(
            lowered
                .provenance
                .iter()
                .any(|record| record.rule.as_str() == "lower.branch-wrapper-key"),
            "the capture is a recorded lowering decision"
        );
    });
    assert_transformed_sound(source, "wrapper-key");
}

#[test]
fn a_dynamic_key_extracts_without_leaving_the_surface() {
    // With `ui.bind` on the surface (series 5) the dynamic arm of the
    // legacy `extract_key_prop` finally has an op to read: the fact
    // records the value text and the binding's index; the op itself
    // stays (a pass removing a binding would shift every later id).
    let source = r#"<p v-if="a" :key="ka">1</p><p v-else :key="kb">2</p>"#;
    with_transformed(source, |lowered, folio, facts, _| {
        assert_eq!(lowered.diagnostics, vec![]);
        let entries = facts.if_facts.sorted_entries();
        assert_eq!(entries.len(), 1);
        let branches = &entries[0].1.branches;
        assert_eq!(
            branches
                .iter()
                .map(|key| key.as_ref().expect("both keys extract").kind.clone())
                .collect::<Vec<_>>(),
            vec![
                BranchKeyKind::Dynamic {
                    source: "ka".into(),
                    bind_index: Some(0),
                },
                BranchKeyKind::Dynamic {
                    source: "kb".into(),
                    bind_index: Some(0),
                },
            ]
        );
        // The binding op stays on the carrier surface.
        assert_eq!(branch_root(folio, 0, 0).bindings.len(), 1);
    });
    assert_transformed_sound(source, "dynamic-keys");
}

#[test]
fn dynamic_key_collisions_are_kind_blind_text_equality() {
    // The legacy `extract_key_value_str` compares the expression text
    // under the default dialect, static and dynamic alike: `key="dup"`
    // and `:key="dup"` collide.
    let source = r#"<p v-if="a" key="dup">1</p><p v-else :key="dup">2</p>"#;
    with_transformed(source, |lowered, _, _, _| {
        assert_eq!(lowered.diagnostics.len(), 1);
        assert_eq!(
            lowered.diagnostics[0].message.as_str(),
            vif::SAME_KEY_MESSAGE
        );
        assert_eq!(
            lowered.diagnostics[0].span,
            span_of(source, r#":key="dup""#, 0)
        );
    });
    assert_transformed_sound(source, "kind-blind-collision");
}

#[test]
fn the_first_key_spelling_in_authored_order_wins() {
    // The legacy scan takes the first key prop in authored order; the
    // S2 surface splits attributes from bindings, so the rule is
    // re-derived from spans. `:key` first: the static `key` attribute
    // stays on the surface.
    let source = r#"<p v-if="a" :key="ka" key="s">1</p><p v-else>2</p>"#;
    with_transformed(source, |_, folio, facts, _| {
        let entries = facts.if_facts.sorted_entries();
        assert_eq!(
            entries[0].1.branches[0]
                .as_ref()
                .expect("the dynamic key wins")
                .kind,
            BranchKeyKind::Dynamic {
                source: "ka".into(),
                bind_index: Some(0),
            }
        );
        assert_eq!(
            branch_root(folio, 0, 0).attributes,
            vec![FolioAttribute {
                name: "key".into(),
                value: Some("s".into()),
                span: span_of(source, r#"key="s""#, 0),
            }],
            "the later static key stays element surface"
        );
    });
}

#[test]
fn a_slot_outlet_branch_key_extracts_from_the_outlet_surface() {
    // Installment 1 counted `keys_slot_root` because `ui.slot` had no
    // attribute surface; series 5 gave it one, and the extraction now
    // runs exactly as on elements.
    let source = r#"<slot v-if="a" key="s"></slot><p v-else>f</p>"#;
    with_transformed(source, |_, folio, facts, _| {
        let entries = facts.if_facts.sorted_entries();
        assert_eq!(
            entries[0].1.branches[0]
                .as_ref()
                .expect("the outlet key extracts")
                .kind,
            BranchKeyKind::Static(Some("s".into()))
        );
        let FolioOp::If(if_op) = &folio.ops[0] else {
            panic!("root is ui.if");
        };
        let [FolioOp::Slot(outlet)] = &if_op.branches[0].ops[..] else {
            panic!("branch root is the outlet");
        };
        assert_eq!(outlet.attributes, vec![], "the key left the surface");
    });
    assert_transformed_sound(source, "outlet-key");
}
