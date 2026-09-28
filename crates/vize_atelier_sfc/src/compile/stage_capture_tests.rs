use super::{
    SfcScriptOutputMode, compile_sfc_for_adapter_with_experimental_options,
    compile_sfc_for_adapter_with_stage_capture,
};
use crate::types::{SfcCompileExperimentalOptions, SfcCompileOptions, SfcParseOptions};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_l0::dump::capture::CaptureOutcome;

fn compile_both(
    source: &str,
    syntax: TemplateSyntaxMode,
) -> (
    crate::types::SfcCompileResult,
    vize_l0::dump::capture::StageCapture,
) {
    compile_both_with_options(source, syntax, SfcCompileOptions::default())
}

fn compile_both_with_options(
    source: &str,
    syntax: TemplateSyntaxMode,
    options: SfcCompileOptions,
) -> (
    crate::types::SfcCompileResult,
    vize_l0::dump::capture::StageCapture,
) {
    let descriptor = crate::parse::parse_sfc(source, SfcParseOptions::default()).unwrap();
    let normal = compile_sfc_for_adapter_with_experimental_options(
        &descriptor,
        options.clone(),
        syntax,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
        SfcCompileExperimentalOptions::default(),
    )
    .unwrap();
    let (observed, capture) = compile_sfc_for_adapter_with_stage_capture(
        &descriptor,
        options,
        syntax,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
        SfcCompileExperimentalOptions::default(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(&observed).unwrap(),
        serde_json::to_value(&normal).unwrap(),
        "capturing stages must not change the compiled SFC",
    );
    (observed, capture)
}

#[test]
fn accepted_ssr_sfc_carries_its_own_backend_pages() {
    let mut options = SfcCompileOptions::default();
    options.template.ssr = true;
    let (_, capture) = compile_both_with_options(
        "<template><div>hello</div></template>",
        TemplateSyntaxMode::Standard,
        options,
    );
    assert_eq!(capture.target, "ssr");
    assert_eq!(capture.outcome, CaptureOutcome::Accepted);
    assert!(
        capture
            .options
            .iter()
            .any(|option| option.name == "ssr" && option.value == "true")
    );
    assert!(!capture.pages.is_empty());
}

#[test]
fn accepted_vapor_sfc_carries_its_own_backend_pages() {
    let mut options = SfcCompileOptions::default();
    options.vapor = true;
    let (_, capture) = compile_both_with_options(
        "<template><div>hello</div></template>",
        TemplateSyntaxMode::Standard,
        options,
    );
    assert_eq!(capture.target, "vapor");
    assert_eq!(capture.outcome, CaptureOutcome::Accepted);
    assert!(
        capture
            .options
            .iter()
            .any(|option| option.name == "vapor" && option.value == "true")
    );
    assert!(!capture.pages.is_empty());
}

#[test]
fn accepted_dom_sfc_carries_only_the_stages_that_produced_its_module() {
    let (_, capture) = compile_both(
        "<template><div>{{ message }}</div></template>",
        TemplateSyntaxMode::Standard,
    );
    assert_eq!(capture.target, "dom");
    assert_eq!(capture.outcome, CaptureOutcome::Accepted);
    assert!(!capture.pages.is_empty());
}

#[test]
fn legacy_template_selection_has_no_native_pages() {
    let (_, capture) = compile_both(
        "<template><div>{{ message }}</div></template>",
        TemplateSyntaxMode::Quirks,
    );
    assert_eq!(capture.target, "dom");
    assert!(matches!(capture.outcome, CaptureOutcome::Legacy(_)));
    assert!(capture.pages.is_empty());
}

#[test]
fn script_only_sfc_has_no_template_stages() {
    let (_, capture) = compile_both(
        "<script>export default { name: 'OnlyScript' }</script>",
        TemplateSyntaxMode::Standard,
    );
    assert_eq!(capture.target, "dom");
    assert!(matches!(capture.outcome, CaptureOutcome::Unavailable(_)));
    assert!(capture.pages.is_empty());
}
