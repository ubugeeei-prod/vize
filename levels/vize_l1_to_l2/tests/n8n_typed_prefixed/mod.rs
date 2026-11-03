//! An explicit authored SFC-language recipe beside the unchanged JS corpus lane.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "whole independently frozen compiler control packets"
)]

mod packets;
mod refusal;
mod stock;
pub use refusal::assert_original_js_refusal;

use crate::davinci_dom_corpus_support::Report;
use serde_json::{Value, json};
use vize_atelier_core::{CodegenResult, CompilerError};
use vize_atelier_dom::{DomCompilerOptions, compile_template_legacy_with_options};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_l0::Allocator;
use vize_l1_to_l2::{
    DomEmitOptions, EmitError, LegacyCaps, ObservedPatchFactsEmit,
    emit_dom_source_patch_facts_observed_with_options,
};

const CONTRACT: &str = include_str!(
    "../../../../tests/_fixtures/differential/compiler/n8n-if-key-regression/prefixed-language-contracts.json"
);

fn contract(name: &str) -> Value {
    assert_eq!(
        packets::digest(CONTRACT.as_bytes()),
        "8f0a18cc2b6d085607e48e17f64356647f06ed2e237ae186a5cabe3336cff1bc"
    );
    let value: Value = serde_json::from_str(CONTRACT).expect("whole frozen language contracts");
    let cases = value["cases"].as_array().expect("exact contract cases");
    assert_eq!(
        cases
            .iter()
            .map(|row| row["name"].as_str().expect("frozen name"))
            .collect::<Vec<_>>(),
        vec!["n8n_original_instance_ai", "n8n-FormInput"]
    );
    cases
        .iter()
        .find(|row| row["name"] == name)
        .expect("explicit original language contract")
        .clone()
}

fn errors_wire(errors: &[CompilerError]) -> Value {
    json!(
        errors
            .iter()
            .map(|error| json!({
                "code":format!("{:?}",error.code), "message":error.message,"location":error.loc
            }))
            .collect::<Vec<_>>()
    )
}

fn compile(
    name: &str,
    source: &str,
    is_ts: bool,
) -> (
    CodegenResult,
    Vec<CompilerError>,
    Result<ObservedPatchFactsEmit, EmitError>,
) {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("complete original SFC");
    let template = descriptor.template.as_ref().expect("original template");
    let source_is_ts = descriptor
        .script
        .as_ref()
        .is_some_and(|block| matches!(block.lang.as_deref(), Some("ts" | "tsx")))
        || descriptor
            .script_setup
            .as_ref()
            .is_some_and(|block| matches!(block.lang.as_deref(), Some("ts" | "tsx")));
    let old_allocator = Allocator::new();
    let old_options = DomCompilerOptions {
        prefix_identifiers: true,
        is_ts,
        ..Default::default()
    };
    let old_options_debug = format!("{old_options:?}");
    let (_, errors, old) =
        compile_template_legacy_with_options(&old_allocator, &template.content, old_options);
    let new_allocator = Allocator::new();
    let options = DomEmitOptions {
        prefix_identifiers: true,
        is_ts,
        ..DomEmitOptions::DEFAULT
    };
    let native = emit_dom_source_patch_facts_observed_with_options(
        &new_allocator,
        &template.content,
        LegacyCaps::VUE3,
        &options,
    );
    let lane = if is_ts {
        "prefixed-sfc-ts"
    } else {
        "prefixed-js-refusal"
    };
    packets::retain(
        name,
        source,
        lane,
        json!({
        "template":template.content,"sourceIsTs":source_is_ts,
        "sfcDescriptorCompleteDebug":format!("{descriptor:?}"),
        "recipe":{"mode":"Function","prefixIdentifiers":true,"isTs":is_ts,"bindingMetadata":null},
        "legacyOptionsCompleteDebug":old_options_debug,
        "nativeOptionsCompleteDebug":format!("{options:?}"),
            "legacy":{"preamble":old.preamble,"code":old.code,"map":old.map,
            "assembled":format!("{}\n{}",old.preamble,old.code),"errors":errors_wire(&errors),
            "errorsCompleteDebug":format!("{errors:#?}")},
            "native":match &native {
                Ok(value) => json!({"tag":"Ok","completeDebug":format!("{value:?}"),
                    "preamble":value.emit.preamble,"code":value.emit.code,
                    "assembled":value.emit.assembled(),"materializedEntries":value.materialized_entries}),
                Err(error) => json!({"tag":"Err","completeDebug":format!("{error:#?}")}),
            }
        }),
    );
    assert!(
        source_is_ts,
        "the opt-in recipe is restricted to authored TypeScript SFCs"
    );
    assert_eq!(
        packets::digest(source.as_bytes()),
        contract(name)["sourceSha256"]
    );
    (old, errors, native)
}

/// This authored opt-in does not change any canonical `Lane::Prefixed` input.
pub fn compare_authored_sfc_ts_prefixed(name: &str, source: &str, report: &mut Report) {
    report.files += 1;
    let (old, errors, native) = compile(name, source, true);
    report.parsed += 1;
    report.templates += 1;
    assert_eq!(errors_wire(&errors), contract(name)["typedLegacyErrors"]);
    let native = native.expect("explicit SFC-language prefix recipe must admit the original");
    assert_eq!(
        native.emit.assembled(),
        format!("{}\n{}", old.preamble, old.code)
    );
    report.compared += 1;
    report.patch_fact_compared += 1;
    report.patch_fact_entries += native.materialized_entries as u64;
}
