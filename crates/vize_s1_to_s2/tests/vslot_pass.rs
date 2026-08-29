//! The `v-slot` pass (P2-9 series 3): semantics pins.
//!
//! Exact-equality oracles over the pass's products — the canonical
//! grouping facts, the first `ScopeOrigin::Synthesized` producer (both
//! directions), the slot-boundary hygiene, the four diagnostics with
//! relief's exact wording, and the tree preservation. The TS-17 folio
//! snapshots live in `vslot_pass_snapshot.rs`.

mod support;

use vize_davinci::id::NodeId;
use vize_s1_to_s2::pass::vslot::{
    DUPLICATE_MESSAGE, EXTRANEOUS_MESSAGE, MISPLACED_MESSAGE, MIXED_MESSAGE, RULE_DEFAULT_NAME,
    RULE_IMPLICIT_DEFAULT, SlotBound, SlotCarrier, SlotFacts, SlotGroup, SlotName, SlotParams,
};
use vize_s2::scope::{ScopeOrigin, ScopeTag};

use support::{assert_transformed_sound, with_lowered, with_transformed};

fn id(index: u32) -> NodeId {
    NodeId::from_index(index).expect("test ids fit")
}

#[test]
fn a_bare_v_slot_synthesizes_the_default_name_never_authored() {
    // The first `ScopeOrigin::Synthesized` producer, direction one: the
    // compiler invents the name, and the origin records exactly that.
    let source = r#"<Panel v-slot="props"><em>{{ props.x }}</em></Panel>"#;
    with_transformed(source, |lowered, _, facts, _| {
        assert_eq!(lowered.diagnostics, vec![]);
        let entries = facts.slot_facts.sorted_entries();
        assert_eq!(
            entries,
            vec![(
                id(0),
                &SlotFacts {
                    groups: vec![SlotGroup {
                        name: SlotName::Static {
                            text: "default".into(),
                            origin: ScopeOrigin::Synthesized {
                                rule: RULE_DEFAULT_NAME.into(),
                            },
                        },
                        params: SlotParams::Scoped {
                            text: "props".into(),
                            tag: ScopeTag::from_index(0),
                            name: SlotBound::Named("props".into()),
                        },
                        carrier: SlotCarrier::Component,
                    }],
                }
            )]
        );
    });
    assert_transformed_sound(source, "bare-v-slot");
}

#[test]
fn an_authored_default_name_is_authored_never_synthesized() {
    // Direction two: the same spelling authored by hand keeps the
    // author's origin — identical text, different recorded fact.
    let source = r#"<Panel v-slot:default="props">x</Panel>"#;
    with_transformed(source, |_, _, facts, _| {
        let entries = facts.slot_facts.sorted_entries();
        assert_eq!(entries.len(), 1);
        let group = &entries[0].1.groups[0];
        let SlotName::Static { text, origin } = &group.name else {
            panic!("authored static name expected: {group:?}");
        };
        assert_eq!(text.as_str(), "default");
        assert!(
            matches!(origin, ScopeOrigin::Authored { .. }),
            "an authored name must never carry the synthesized origin: {origin:?}"
        );
    });
    assert_transformed_sound(source, "authored-default");
}

#[test]
fn the_canonical_name_folds_modifiers_exactly_as_the_legacy_lane() {
    // `get_slot_name`'s dot-folding, on both the authored and the
    // synthesized base; a dynamic argument ignores modifiers.
    let source = r#"<Panel v-slot.raw="p">x</Panel>"#;
    with_transformed(source, |_, _, facts, _| {
        let entries = facts.slot_facts.sorted_entries();
        let SlotName::Static { text, origin } = &entries[0].1.groups[0].name else {
            panic!("static name expected");
        };
        assert_eq!(text.as_str(), "default.raw");
        assert!(matches!(origin, ScopeOrigin::Synthesized { .. }));
    });
    let source = r#"<Card><template v-slot:body.raw="row">y</template><i>z</i></Card>"#;
    with_transformed(source, |_, _, facts, _| {
        let entries = facts.slot_facts.sorted_entries();
        let groups = &entries[0].1.groups;
        let SlotName::Static { text, origin } = &groups[0].name else {
            panic!("static name expected");
        };
        assert_eq!(text.as_str(), "body.raw");
        assert!(matches!(origin, ScopeOrigin::Authored { .. }));
    });
}

#[test]
fn the_implicit_default_group_is_synthesized_from_non_slot_content() {
    // `collect_slots`' implicit default: named template groups plus
    // trailing content synthesize the `default` group; whitespace-only
    // text alone never does (the pass's one canonical deviation from
    // the raw legacy predicate, measured in the differential lane).
    let source =
        "<Card>\n  <template #head=\"h\"><b>{{ h }}</b></template>\n  <p>body</p>\n</Card>";
    with_transformed(source, |lowered, _, facts, _| {
        assert_eq!(lowered.diagnostics, vec![]);
        let entries = facts.slot_facts.sorted_entries();
        assert_eq!(entries.len(), 1);
        let groups = &entries[0].1.groups;
        assert_eq!(groups.len(), 2);
        // Card=0, the template=1 (the leading whitespace text is gone —
        // the installment-4 condense removes it before ids mint).
        assert_eq!(groups[0].name.text(), "head");
        assert_eq!(groups[0].carrier, SlotCarrier::Template(Some(id(1))));
        assert_eq!(
            groups[1],
            SlotGroup {
                name: SlotName::Static {
                    text: "default".into(),
                    origin: ScopeOrigin::Synthesized {
                        rule: RULE_IMPLICIT_DEFAULT.into(),
                    },
                },
                params: SlotParams::Absent,
                carrier: SlotCarrier::Implicit,
            }
        );
    });

    // Whitespace-only siblings: no implicit default.
    let padded = "<Card>\n  <template #head=\"h\">x</template>\n</Card>";
    with_transformed(padded, |_, _, facts, _| {
        let entries = facts.slot_facts.sorted_entries();
        assert_eq!(entries[0].1.groups.len(), 1, "filler is not content");
    });
}

#[test]
fn a_slot_prop_never_captures_an_outer_authored_binding() {
    // The hygiene pin across the slot boundary: the same spelling in a
    // v-for scope outside and a slot-props scope inside stays two
    // distinct (name, tag) pairs — capture is impossible by identity,
    // exactly the P2-8 design.
    let source = r#"<li v-for="item in items"><Card v-slot="item"><i>{{ item }}</i></Card></li>"#;
    with_transformed(source, |lowered, _, facts, _| {
        assert_eq!(lowered.diagnostics, vec![]);
        let for_entries = facts.for_facts.sorted_entries();
        let slot_entries = facts.slot_facts.sorted_entries();
        assert_eq!(for_entries.len(), 1);
        assert_eq!(slot_entries.len(), 1);
        let for_tag = for_entries[0].1.tag;
        let SlotParams::Scoped { text, tag, name } = &slot_entries[0].1.groups[0].params else {
            panic!("scoped params expected");
        };
        assert_eq!(text.as_str(), "item");
        assert_eq!(name, &SlotBound::Named("item".into()));
        assert_ne!(for_tag, *tag, "one spelling, two introduction sites");
        // Both names are authored — the pass synthesized no binding.
        for (_, scope) in lowered.scopes.sorted_entries() {
            for binding in &scope.bindings {
                assert!(matches!(binding.origin, ScopeOrigin::Authored { .. }));
            }
        }
    });
    assert_transformed_sound(source, "slot-shadowing");
}

#[test]
fn two_spellings_on_one_carrier_are_two_introduction_sites() {
    // The shared-carrier case P2-8's element-keyed scope could not
    // represent: each spelling's scope keys its own binding op now.
    let source = r#"<Card><template #a="x" #b="y">z</template></Card>"#;
    with_transformed(source, |lowered, _, facts, _| {
        let entries = facts.slot_facts.sorted_entries();
        let groups = &entries[0].1.groups;
        assert_eq!(groups.len(), 2);
        let tags: Vec<u32> = groups
            .iter()
            .map(|group| match &group.params {
                SlotParams::Scoped { tag, .. } => tag.index(),
                SlotParams::Absent => panic!("both spellings have params"),
            })
            .collect();
        assert_eq!(tags, vec![0, 1], "fresh tag per spelling");
        assert_eq!(lowered.scopes.len(), 2);
    });
    assert_transformed_sound(source, "shared-carrier");
}

#[test]
fn a_destructuring_params_position_is_pending_with_its_scope_standing() {
    // The #4365 boundary consumed as recorded: pattern names are not
    // enumerated anywhere, the position pessimizes, the tag stands.
    let source = r#"<Card v-slot="{ a, b }">x</Card>"#;
    with_transformed(source, |lowered, _, facts, _| {
        let entries = facts.slot_facts.sorted_entries();
        let SlotParams::Scoped { text, name, .. } = &entries[0].1.groups[0].params else {
            panic!("scoped params expected");
        };
        assert_eq!(text.as_str(), "{ a, b }");
        assert_eq!(name, &SlotBound::Pending);
        let (_, scope) = lowered.scopes.sorted_entries()[0];
        assert_eq!(scope.bindings.len(), 0, "pattern names wait for #4365");
    });
    assert_transformed_sound(source, "pattern-params");
}

#[test]
fn the_four_diagnostics_carry_reliefs_exact_wording() {
    // Misplaced: a native non-template element.
    with_transformed(r#"<span v-slot="p">x</span>"#, |lowered, _, _, _| {
        assert_eq!(lowered.diagnostics.len(), 1);
        assert_eq!(lowered.diagnostics[0].message.as_str(), MISPLACED_MESSAGE);
    });
    // Mixed: own v-slot plus a template slot, one error, first template.
    with_transformed(
        r#"<Modal v-slot="own"><template #late>y</template></Modal>"#,
        |lowered, _, _, _| {
            assert_eq!(lowered.diagnostics.len(), 1);
            assert_eq!(lowered.diagnostics[0].message.as_str(), MIXED_MESSAGE);
        },
    );
    // Duplicate: the later static name, grouping drops it silently.
    with_transformed(
        r#"<Grid><template #cell>a</template><template #cell>b</template></Grid>"#,
        |lowered, _, facts, _| {
            assert_eq!(lowered.diagnostics.len(), 1);
            assert_eq!(lowered.diagnostics[0].message.as_str(), DUPLICATE_MESSAGE);
            assert_eq!(facts.slot_facts.sorted_entries()[0].1.groups.len(), 1);
        },
    );
    // Extraneous: an authored default slot plus stray implicit content.
    with_transformed(
        r#"<Grid><template #default>c</template><hr></Grid>"#,
        |lowered, _, _, _| {
            assert_eq!(lowered.diagnostics.len(), 1);
            assert_eq!(lowered.diagnostics[0].message.as_str(), EXTRANEOUS_MESSAGE);
        },
    );
}

#[test]
fn a_template_carrier_outside_a_component_neither_groups_nor_errors() {
    // The legacy lane validates slot templates only under components; a
    // stray one stays untouched — but its scope is still consumed (the
    // hygiene law covers every introduction site).
    let source = r#"<div><template #a="p">x</template></div>"#;
    with_transformed(source, |lowered, _, facts, _| {
        assert_eq!(lowered.diagnostics, vec![]);
        assert!(facts.slot_facts.is_empty());
        assert_eq!(lowered.scopes.len(), 1);
        let scope_records: Vec<&str> = lowered
            .provenance
            .iter()
            .filter(|record| record.rule.as_str() == "pass.v-slot.scope")
            .map(|record| record.after.as_str())
            .collect();
        assert_eq!(scope_records, vec!["fact params=p"]);
    });
    assert_transformed_sound(source, "stray-template");
}

#[test]
fn the_pass_preserves_the_tree() {
    // Like v-for, a preserving mandatory pass: grouping is a published
    // fact — a binding op cannot leave the surface without shifting
    // every page-order id after it. Diagnostics DO append (unlike
    // v-for), so only the folio is pinned byte-identical.
    let source = r#"<Card v-slot="p"><b>{{ p }}</b></Card><span v-slot="q">x</span>"#;
    let before = with_lowered(source, |_, folio| folio.clone());
    with_transformed(source, |_, folio, _, _| {
        assert_eq!(folio, &before, "the v-slot pass must not mutate");
    });
}

#[test]
fn a_v_slot_on_an_outlet_is_misplaced_with_reliefs_wording() {
    // Installment 3 recorded the outlet's missing `VSlotMisplaced` twin
    // as waiting on the outlet binding surface; series 5 landed it: the
    // spelling lowers to `ui.slot-content` on the `ui.slot`, and this
    // pass fires the legacy diagnostic where the legacy lane fires it
    // (`validate_v_slot_usage` — a `<slot>` is neither a component nor
    // a `<template>` carrier).
    let source = r#"<slot v-slot="p"></slot>"#;
    with_transformed(source, |lowered, _, facts, _| {
        assert_eq!(lowered.diagnostics.len(), 1);
        assert_eq!(lowered.diagnostics[0].message.as_str(), MISPLACED_MESSAGE);
        assert!(facts.slot_facts.is_empty(), "no grouping exists");
    });
    assert_transformed_sound(source, "misplaced-on-outlet");
}

#[test]
fn the_pass_is_total_over_malformed_slots() {
    for (name, source) in [
        ("empty-params", r#"<Card v-slot="">x</Card>"#),
        ("value-less", r#"<Card v-slot>x</Card>"#),
        (
            "dynamic-name-hole",
            r#"<Card><template #[]>x</template></Card>"#,
        ),
        ("unclosed", r#"<Card><template #a>x</Card>"#),
        ("on-outlet", r#"<slot v-slot="p"></slot>"#),
        ("empty", ""),
    ] {
        assert_transformed_sound(source, name);
    }
}
