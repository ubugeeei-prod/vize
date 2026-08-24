//! The `v-model` pass (P2-9 series 5): semantics pins.
//!
//! Exact-equality oracles over the pass's two products — the two legacy
//! validations with relief's exact wording, and the sparse fault table
//! that mirrors the legacy lane's removal of invalid models — plus the
//! scope-environment rules the port mirrors from the live transform
//! (`crates/vize_atelier_core/src/lane/element.rs` +
//! `crates/vize_atelier_core/src/lane/traverse.rs`). The TS-17 folio
//! snapshots live in `vmodel_pass_snapshot.rs`.

mod support;

use vize_davinci::diagnostic::{Severity, Stage};
use vize_ricalco::pass::{ModelFacts, ModelFault, vmodel};

use support::{assert_transformed_sound, with_transformed};

#[test]
fn a_model_on_a_v_for_alias_is_flagged_with_reliefs_wording() {
    let source = r#"<li v-for="item in xs"><input v-model="item"></li>"#;
    with_transformed(source, |lowered, _, facts, _| {
        assert_eq!(lowered.diagnostics.len(), 1);
        let diagnostic = &lowered.diagnostics[0];
        assert_eq!(diagnostic.severity, Severity::Error);
        assert_eq!(diagnostic.stage, Stage::Semantic);
        assert_eq!(diagnostic.message.as_str(), vmodel::ON_SCOPE_MESSAGE);
        // The fault fact is the legacy removal's preserving twin: one
        // sparse entry, keyed by the model binding op's page-order id
        // (%0 ui.for, %1 li, %2 input, %3 the model binding).
        let entries = facts.model_faults.sorted_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0.index(), 3);
        assert_eq!(
            entries[0].1,
            &ModelFacts {
                fault: ModelFault::OnScope,
            }
        );
    });
    assert_transformed_sound(source, "for-alias");
}

#[test]
fn the_key_and_index_aliases_are_scope_names_too() {
    let source = r#"<td v-for="(value, key, index) in obj"><input v-model="index"></td>"#;
    with_transformed(source, |lowered, _, _, _| {
        assert_eq!(lowered.diagnostics.len(), 1);
        assert_eq!(
            lowered.diagnostics[0].message.as_str(),
            vmodel::ON_SCOPE_MESSAGE
        );
    });
}

#[test]
fn a_model_on_a_slot_prop_is_flagged() {
    let source = r#"<Card v-slot="row"><input v-model="row"></Card>"#;
    with_transformed(source, |lowered, _, facts, _| {
        assert_eq!(lowered.diagnostics.len(), 1);
        assert_eq!(
            lowered.diagnostics[0].message.as_str(),
            vmodel::ON_SCOPE_MESSAGE
        );
        assert_eq!(facts.model_faults.len(), 1);
    });
    assert_transformed_sound(source, "slot-prop");
}

#[test]
fn the_carriers_own_model_is_outside_its_slot_scope() {
    // The live lane processes an element's own props before entering its
    // v-slot scope (`traverse_node` order), so the carrier's own model
    // never sees its own slot prop.
    let source = r#"<Card v-slot="x" v-model="x">y</Card>"#;
    with_transformed(source, |lowered, _, facts, _| {
        assert_eq!(lowered.diagnostics, vec![]);
        assert!(facts.model_faults.is_empty());
    });
}

#[test]
fn a_pattern_params_scope_contributes_no_names() {
    // The one-scanner rule (#4365): the S2 lane enumerates pattern
    // bindings nowhere yet, so — unlike the legacy lane, which
    // enumerates `{ row }` — no scope name exists to match. The
    // differential lane counts this class (`models_pattern_scope`)
    // instead of comparing inside it; this pin is the recorded weaker
    // behaviour, kept loud on purpose.
    let source = r#"<Card v-slot="{ row }"><input v-model="row"></Card>"#;
    with_transformed(source, |lowered, _, facts, _| {
        assert_eq!(lowered.diagnostics, vec![]);
        assert!(facts.model_faults.is_empty());
    });
}

#[test]
fn an_argument_on_a_plain_element_is_flagged() {
    let source = r#"<input v-model:value="name">"#;
    with_transformed(source, |lowered, _, facts, _| {
        assert_eq!(lowered.diagnostics.len(), 1);
        assert_eq!(
            lowered.diagnostics[0].message.as_str(),
            vmodel::ARG_ON_ELEMENT_MESSAGE
        );
        let entries = facts.model_faults.sorted_entries();
        assert_eq!(
            entries[0].1,
            &ModelFacts {
                fault: ModelFault::ArgOnElement,
            }
        );
    });
    assert_transformed_sound(source, "arg-on-element");
}

#[test]
fn a_component_argument_is_fine() {
    let source = r#"<Field v-model:title="doc.title"></Field>"#;
    with_transformed(source, |lowered, _, facts, _| {
        assert_eq!(lowered.diagnostics, vec![]);
        assert!(facts.model_faults.is_empty());
    });
}

#[test]
fn the_on_scope_check_wins_over_the_argument_check() {
    // The legacy order: on-scope first, one fault per model.
    let source = r#"<li v-for="item in xs"><input v-model:value="item"></li>"#;
    with_transformed(source, |lowered, _, facts, _| {
        assert_eq!(lowered.diagnostics.len(), 1);
        assert_eq!(
            lowered.diagnostics[0].message.as_str(),
            vmodel::ON_SCOPE_MESSAGE
        );
        let entries = facts.model_faults.sorted_entries();
        assert_eq!(
            entries[0].1,
            &ModelFacts {
                fault: ModelFault::OnScope,
            }
        );
    });
}

#[test]
fn a_valid_model_leaves_no_fact_entry() {
    // Sparse-table discipline: entries only for models the legacy lane
    // would remove.
    let source = r#"<input v-model.trim="draft.note">"#;
    with_transformed(source, |lowered, _, facts, _| {
        assert_eq!(lowered.diagnostics, vec![]);
        assert!(facts.model_faults.is_empty());
    });
}

#[test]
fn the_scope_closes_with_its_region() {
    // A model after the loop is outside the loop's scope.
    let source = r#"<div><p v-for="item in xs">{{ item }}</p><input v-model="item"></div>"#;
    with_transformed(source, |lowered, _, _, _| {
        assert_eq!(lowered.diagnostics, vec![]);
    });
    assert_transformed_sound(source, "scope-closes");
}

#[test]
fn the_pass_is_total_over_malformed_inputs() {
    for (name, source) in [
        ("no-expression", r#"<input v-model>"#),
        ("model-on-outlet", r#"<slot v-model="x"></slot>"#),
        ("dynamic-argument", r#"<Comp v-model:[k]="x"></Comp>"#),
        ("empty", ""),
    ] {
        assert_transformed_sound(source, name);
    }
}
