//! The opt-in stage sidecar describes the emission that returned the module.

use vize_atelier_core::options::{
    CodegenExperimentalOptions, CodegenOptions, CustomElementMatcher, TemplateSyntaxMode,
};
use vize_atelier_dom::{
    DomCompilerOptions,
    compile_template_with_custom_elements_template_syntax_codegen_and_experimental_options,
    compile_template_with_custom_elements_template_syntax_codegen_and_experimental_options_with_stage_capture,
};
use vize_l0::Allocator;
use vize_l0::dump::capture::{CaptureOutcome, StageCapture};
use vize_l0::level::Level;

fn compare(source: &str, options: DomCompilerOptions) -> StageCapture {
    let allocator = Allocator::new();
    let (_, plain_errors, plain) =
        compile_template_with_custom_elements_template_syntax_codegen_and_experimental_options(
            &allocator,
            source,
            options.clone(),
            TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(),
            CodegenOptions::default(),
            CodegenExperimentalOptions::default(),
        );
    let mut capture = StageCapture::new("dom");
    let (_, observed_errors, observed) =
        compile_template_with_custom_elements_template_syntax_codegen_and_experimental_options_with_stage_capture(
            &allocator,
            source,
            options,
            TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(),
            CodegenOptions::default(),
            CodegenExperimentalOptions::default(),
            &mut capture,
        );
    assert_eq!(plain_errors.len(), observed_errors.len());
    assert_eq!(plain.code, observed.code);
    assert_eq!(plain.preamble, observed.preamble);
    assert_eq!(plain.map, observed.map);
    if capture.outcome == CaptureOutcome::Accepted {
        let emitted = capture.pages.last().expect("emitted page");
        assert_eq!(emitted.level, Level::L4);
        assert_eq!(emitted.step, "emit");
        assert_eq!(
            emitted.text,
            format!("{}\n{}", observed.preamble, observed.code)
        );
    }
    capture
}

#[test]
fn accepted_dom_capture_is_from_the_emitting_run() {
    let capture = compare("<div>{{ msg }}</div>", DomCompilerOptions::default());
    assert_eq!(capture.outcome, CaptureOutcome::Accepted);
    assert_eq!(
        capture
            .pages
            .iter()
            .map(|page| page.level)
            .collect::<Vec<_>>(),
        [Level::L1, Level::L2, Level::L2, Level::L4],
    );
    assert_eq!(capture.pages[2].step, "facts");
    assert!(capture.pages[2].text.contains("static_facts"));
    assert_ne!(capture.pages[1].text, capture.pages[2].text);
}

#[test]
fn compatibility_selection_discards_native_pages() {
    let capture = compare(
        "<div>{{ msg }}</div>",
        DomCompilerOptions {
            experimental_patterned_template: true,
            ..DomCompilerOptions::default()
        },
    );
    assert_eq!(
        capture.outcome,
        CaptureOutcome::Legacy("patterned-template".into())
    );
    assert!(capture.pages.is_empty());
}

#[test]
fn parse_failure_does_not_claim_a_production_stage() {
    let capture = compare("<div", DomCompilerOptions::default());
    assert_eq!(
        capture.outcome,
        CaptureOutcome::Rejected("parse-error".into())
    );
    assert!(capture.pages.is_empty());
}
