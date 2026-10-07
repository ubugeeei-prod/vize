//! Whole-module controls for props left empty by template-loop key suppression.
//! Stock Vue rejects misplaced child keys; that refusal stays in the receipts.
#![expect(
    clippy::expect_used,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "authored controls compare complete compiler modules"
)]

use serde_json::Value;

use crate::davinci_dom_corpus_support::{Lane, Report, compare_sfc_template_lane};

macro_rules! official_control {
    ($name:literal) => {
        include_str!(concat!(
            "../../../../tests/_fixtures/differential/compiler/n8n-empty-suppressed-props/official/",
            $name,
            ".json"
        ))
    };
}

const CONTROLS: &[&str] = &[
    official_control!("named_slot_key_only"),
    official_control!("pure_key_only"),
    official_control!("pure_empty"),
    official_control!("surviving_bound_prop"),
    official_control!("surviving_static_prop"),
    official_control!("template_injected_key"),
    official_control!("spread_with_key"),
    official_control!("multiple_children_keep_key"),
    official_control!("grandchild_keeps_key"),
    official_control!("element_loop_keeps_key"),
    official_control!("static_key_only"),
];

#[test]
fn suppressed_child_key_uses_null_only_when_no_props_survive() {
    for lane in [Lane::Default, Lane::Prefixed, Lane::Bindings] {
        let mut report = Report::default();
        for receipt in CONTROLS {
            let receipt: Value = serde_json::from_str(receipt).expect("complete official receipt");
            let name = receipt.get("name").and_then(Value::as_str).expect("name");
            let source = receipt
                .get("source")
                .and_then(Value::as_str)
                .expect("whole authored SFC");
            compare_sfc_template_lane(name, source, &mut report, lane);
        }
        assert_eq!(report.files, 11);
        assert_eq!(report.parsed, 11);
        assert_eq!(report.templates, 11);
        assert_eq!(report.compared, 11);
        assert_eq!(report.patch_fact_compared, 11);
        assert_eq!(report.old_error_skips, 0);
        assert_eq!(report.s2_refusal_count, 0, "{:?}", report.s2_refusals);
        assert_eq!(report.divergence_count, 0, "{:?}", report.divergences);
        assert_eq!(report.s2_refusals, Vec::<String>::new());
        assert_eq!(report.divergences, Vec::<String>::new());
    }
}

#[cfg(feature = "legacy-differential")]
#[test]
fn retained_scope_id_prevents_the_empty_props_null_shortcut() {
    use vize_atelier_core::options::CodegenMode;
    use vize_atelier_dom::{DomCompilerOptions, compile_template_legacy_with_options};
    use vize_l0::Allocator;
    use vize_l1_to_l2::{
        DomEmitMode, DomEmitOptions, LegacyCaps, emit_dom_source_patch_facts_observed_with_options,
    };

    let receipt: Value = serde_json::from_str(CONTROLS.first().expect("named-slot control"))
        .expect("complete official receipt");
    let template = receipt
        .get("template")
        .and_then(Value::as_str)
        .expect("whole authored template");
    for (prefix_identifiers, module) in [(false, false), (true, false), (true, true)] {
        let old_allocator = Allocator::new();
        let (_, errors, old) = compile_template_legacy_with_options(
            &old_allocator,
            template,
            DomCompilerOptions {
                prefix_identifiers,
                mode: if module {
                    CodegenMode::Module
                } else {
                    CodegenMode::Function
                },
                scope_id: Some(vize_l0::String::from("data-v-empty-props")),
                ..Default::default()
            },
        );
        assert_eq!(errors.len(), 0);
        let new_allocator = Allocator::new();
        let new = emit_dom_source_patch_facts_observed_with_options(
            &new_allocator,
            template,
            LegacyCaps::VUE3,
            &DomEmitOptions {
                prefix_identifiers,
                mode: if module {
                    DomEmitMode::Module
                } else {
                    DomEmitMode::Function
                },
                scope_id: Some("data-v-empty-props"),
                ..DomEmitOptions::DEFAULT
            },
        )
        .expect("native scoped control");
        assert_eq!(
            new.emit.assembled(),
            format!("{}\n{}", old.preamble, old.code)
        );
    }
}
