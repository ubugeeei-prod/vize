use vize_atelier_core::{TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_ssr::{
    Allocator, SsrCodegenResult, SsrCompilerExperimentalOptions, SsrCompilerOptions,
    compile_ssr_with_sfc_slotted_context, compile_ssr_with_sfc_slotted_context_and_capture,
};
use vize_l0::{
    dump::capture::{CaptureOutcome, StageCapture},
    level::Level,
};

fn compile(source: &str, options: SsrCompilerOptions) -> (SsrCodegenResult, StageCapture) {
    let allocator = Allocator::new();
    let mut capture = StageCapture::new("ssr");
    let (_, errors, result) = compile_ssr_with_sfc_slotted_context_and_capture(
        &allocator,
        source,
        options.clone(),
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        SsrCompilerExperimentalOptions::default(),
        true,
        &mut capture,
    );
    assert!(errors.is_empty(), "{errors:?}");
    let (_, baseline_errors, baseline) = compile_ssr_with_sfc_slotted_context(
        &allocator,
        source,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        SsrCompilerExperimentalOptions::default(),
        true,
    );
    assert!(baseline_errors.is_empty(), "{baseline_errors:?}");
    assert_eq!(result.code, baseline.code);
    assert_eq!(result.preamble, baseline.preamble);
    (result, capture)
}

#[test]
fn accepted_ssr_pages_describe_the_module_emitter_input() {
    let (result, capture) = compile("<div>hello</div>", SsrCompilerOptions::default());
    assert!(!result.code.is_empty());
    assert_eq!(capture.outcome, CaptureOutcome::Accepted);
    assert_eq!(
        capture
            .options
            .iter()
            .find(|option| option.name == "template_syntax")
            .map(|option| option.value.as_str()),
        Some("Standard")
    );
    assert_eq!(
        capture
            .options
            .iter()
            .find(|option| option.name == "dialect")
            .map(|option| option.value.as_str()),
        Some("V3")
    );
    assert_eq!(
        capture
            .options
            .iter()
            .find(|option| option.name == "slotted")
            .map(|option| option.value.as_str()),
        Some("true")
    );
    assert_eq!(
        capture
            .pages
            .iter()
            .map(|page| page.level)
            .collect::<Vec<_>>(),
        [
            Level::L1,
            Level::L2,
            Level::L2,
            Level::L3,
            Level::L3,
            Level::L3,
            Level::L4
        ]
    );
    assert_eq!(capture.pages[0].text.as_str(), "<div>hello</div>");
    assert!(capture.pages[1].text.contains("[l2-"));
    assert!(capture.pages[3].text.contains("[l3-"));
    // The SSR backend returns two chunks; this is the exact SFC render
    // module assembly, before the SFC script is composed around it.
    let mut render_module = vize_l0::String::default();
    render_module.push_str(&result.preamble);
    render_module.push('\n');
    render_module.push_str(&result.code);
    render_module.push('\n');
    assert_eq!(capture.pages[6].step, "emit");
    assert_eq!(capture.pages[6].text, render_module);
}

#[test]
fn retained_ssr_module_has_no_native_pages() {
    let (result, capture) = compile(
        "<div>hello</div>",
        SsrCompilerOptions {
            comments: true,
            ..Default::default()
        },
    );
    assert!(!result.code.is_empty());
    assert!(matches!(capture.outcome, CaptureOutcome::Legacy(_)));
    assert!(capture.pages.is_empty());
}
