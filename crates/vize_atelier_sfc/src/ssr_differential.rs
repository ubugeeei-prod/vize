//! Production-entry SSR lane comparison (`ssr-production-comparison`).
//!
//! One SFC compiles through the adapter entry point exactly as the Vite plugin
//! calls it in SSR mode (`SeparateTemplate`, the filename as template id,
//! `is_ts` from the script lang, scoped styles), once on the selected lane and
//! once with the legacy walker pinned. Nothing is derived by hand, so a Croquis
//! summary or any other option the SFC compiler adds reaches the lane selector
//! as it does in production. The SSR corpus gate and the reducer's divergence
//! predicate share this comparison, so a reduced witness differs exactly when
//! the corpus counts a divergence. Lane pinning comes from the caller
//! (`vize_atelier_ssr::differential::on_both_lanes`), so this module never arms
//! the SSR differential itself.

use vize_atelier_core::CodegenOptions;
use vize_atelier_core::options::{CustomElementMatcher, TemplateSyntaxMode};
use vize_carton::{String, cstr};

use crate::{
    ScriptCompileOptions, SfcCompileExperimentalOptions, SfcCompileOptions, SfcCompileResult,
    SfcDescriptor, SfcError, SfcParseOptions, SfcScriptOutputMode, StyleCompileOptions,
    TemplateCompileOptions, compile_sfc_for_adapter_with_experimental_options, parse_sfc,
};

/// One production SSR compile's outcome.
pub type SsrProductionOutcome = Result<SfcCompileResult, SfcError>;

/// The selected outcome, its lane verdicts, and the legacy-pinned outcome.
pub type SsrLaneOutcomes = (
    SsrProductionOutcome,
    std::vec::Vec<&'static str>,
    SsrProductionOutcome,
);

/// One SFC compiled on both SSR lanes through the production entry point.
#[derive(Debug)]
pub struct SsrProductionPair {
    /// Lane verdicts of every production SSR compile on the selected run, in
    /// compile order (`s4`, `legacy.<reason>`, `rejected`); empty when the SFC
    /// never reached SSR.
    pub verdicts: std::vec::Vec<&'static str>,
    pub selected: SsrProductionOutcome,
    pub legacy: SsrProductionOutcome,
}

impl SsrProductionPair {
    /// `None` outside the comparable boundary (the legacy lane failed);
    /// otherwise whether the selected lane's complete output differs.
    #[must_use]
    pub fn differs(&self) -> Option<bool> {
        match (&self.selected, &self.legacy) {
            (_, Err(_)) => None,
            (Ok(selected), Ok(legacy)) => Some(!same(selected, legacy)),
            (Err(_), Ok(_)) => Some(true),
        }
    }
}

fn same(selected: &SfcCompileResult, legacy: &SfcCompileResult) -> bool {
    selected.code == legacy.code
        && selected.css == legacy.css
        && selected.map == legacy.map
        && cstr!("{:?}", selected.errors) == cstr!("{:?}", legacy.errors)
        && cstr!("{:?}", selected.warnings) == cstr!("{:?}", legacy.warnings)
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
    let filename = String::from(name);
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

fn compile(name: &str, descriptor: &SfcDescriptor) -> SsrProductionOutcome {
    compile_sfc_for_adapter_with_experimental_options(
        descriptor,
        adapter_options(name, descriptor),
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
        SfcCompileExperimentalOptions::default(),
    )
}

/// Compile `source`, identified as `name` the way a bundler passes its id, on
/// both lanes: `lanes` runs the compile on the selected lane (returning its
/// verdicts) and again with the legacy walker pinned. `None` when the source
/// does not parse as an SFC or has no template.
pub fn compare_production(
    name: &str,
    source: &str,
    lanes: impl FnOnce(&dyn Fn() -> SsrProductionOutcome) -> SsrLaneOutcomes,
) -> Option<SsrProductionPair> {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).ok()?;
    descriptor.template.as_ref()?;
    let (selected, verdicts, legacy) = lanes(&|| compile(name, &descriptor));
    Some(SsrProductionPair {
        verdicts,
        selected,
        legacy,
    })
}
