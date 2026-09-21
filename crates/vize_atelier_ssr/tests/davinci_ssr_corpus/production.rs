//! The production sweep: every SFC compiled through the adapter entry point
//! exactly as the Vite plugin calls it in SSR mode (`SeparateTemplate`, the
//! filename as template id, `is_ts` from the script lang, scoped styles), once
//! on the selected lane and once with the legacy walker pinned. The whole SFC
//! output must be byte-identical, and each production SSR compile's lane
//! verdict is the honest production-reach number: unlike the emitter sweep,
//! nothing here is derived by hand, so a Croquis summary or any other option
//! the SFC compiler adds reaches the selector as it does in production.

use std::collections::BTreeMap;

use vize_atelier_core::CodegenOptions;
use vize_atelier_core::options::{CustomElementMatcher, TemplateSyntaxMode};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileExperimentalOptions, SfcCompileOptions, SfcCompileResult,
    SfcDescriptor, SfcParseOptions, SfcScriptOutputMode, StyleCompileOptions,
    TemplateCompileOptions, compile_sfc_for_adapter_with_experimental_options,
};
use vize_atelier_ssr::differential::{record_lanes, with_legacy_lane};

#[derive(Default)]
pub(crate) struct ProductionReport {
    pub(crate) templates: u64,
    pub(crate) compared: u64,
    pub(crate) error_skips: u64,
    /// Lane verdicts of every production SSR compile (`s4`, `legacy.<reason>`,
    /// `rejected`); `none` when an SFC with a template never reached SSR.
    pub(crate) lanes: BTreeMap<&'static str, u64>,
    pub(crate) rejected: Vec<String>,
    pub(crate) divergences: Vec<String>,
}

impl ProductionReport {
    pub(crate) fn reach(&self) -> u64 {
        self.lanes.get("s4").copied().unwrap_or_default()
    }
}

fn is_ts(descriptor: &SfcDescriptor) -> bool {
    let ts = |lang: Option<&str>| matches!(lang, Some("ts" | "tsx"));
    descriptor
        .script
        .as_ref()
        .is_some_and(|block| ts(block.lang.as_deref()))
        || descriptor
            .script_setup
            .as_ref()
            .is_some_and(|block| ts(block.lang.as_deref()))
}

/// The options `vize_vitrine`'s SFC adapter builds for an SSR compile.
fn adapter_options(name: &str, descriptor: &SfcDescriptor) -> SfcCompileOptions {
    let filename: vize_s0::String = name.into();
    let scoped = descriptor.styles.iter().any(|style| style.scoped);
    let is_ts = is_ts(descriptor);
    SfcCompileOptions {
        parse: SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        },
        script: ScriptCompileOptions {
            id: Some(filename.clone()),
            is_ts,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            id: Some(filename.clone()),
            scoped,
            ssr: true,
            is_ts,
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: filename,
            scoped,
            ..Default::default()
        },
        vapor: false,
        scope_id: None,
    }
}

fn compile(name: &str, descriptor: &SfcDescriptor) -> Result<SfcCompileResult, String> {
    compile_sfc_for_adapter_with_experimental_options(
        descriptor,
        adapter_options(name, descriptor),
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
        SfcCompileExperimentalOptions::default(),
    )
    .map_err(|error| format!("{error:?}"))
}

fn same(selected: &SfcCompileResult, legacy: &SfcCompileResult) -> bool {
    selected.code == legacy.code
        && selected.css == legacy.css
        && selected.map == legacy.map
        && format!("{:?}", selected.errors) == format!("{:?}", legacy.errors)
        && format!("{:?}", selected.warnings) == format!("{:?}", legacy.warnings)
}

pub(crate) fn compare(name: &str, source: &str, report: &mut ProductionReport) {
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(source, SfcParseOptions::default()) else {
        return;
    };
    if descriptor.template.is_none() {
        return;
    }
    report.templates += 1;
    let (selected, verdicts) = record_lanes(|| compile(name, &descriptor));
    let legacy = with_legacy_lane(|| compile(name, &descriptor));
    if verdicts.is_empty() {
        *report.lanes.entry("none").or_default() += 1;
    }
    for verdict in &verdicts {
        *report.lanes.entry(verdict).or_default() += 1;
        if *verdict == "rejected" {
            report.rejected.push(name.to_owned());
        }
    }
    match (&selected, &legacy) {
        (_, Err(_)) => report.error_skips += 1,
        (Ok(selected), Ok(legacy)) => {
            report.compared += 1;
            if !same(selected, legacy) {
                let detail = if report.divergences.len() < 10 {
                    format!(
                        "\n--- selected\n{}\n--- legacy\n{}",
                        selected.code, legacy.code
                    )
                } else {
                    String::new()
                };
                report
                    .divergences
                    .push(format!("{name} {verdicts:?}{detail}"));
            }
        }
        (Err(error), Ok(_)) => {
            report.compared += 1;
            report.divergences.push(format!(
                "{name} {verdicts:?}: selected lane failed: {error}"
            ));
        }
    }
}

pub(crate) fn assert_clean(label: &str, report: &ProductionReport) {
    assert!(
        report.rejected.is_empty(),
        "{label}: broken S4 plan invariants on the production path: {:#?}",
        report.rejected
    );
    assert!(
        report.divergences.is_empty(),
        "{label}: {} production SFC divergences:\n{}",
        report.divergences.len(),
        report.divergences.join("\n")
    );
}
