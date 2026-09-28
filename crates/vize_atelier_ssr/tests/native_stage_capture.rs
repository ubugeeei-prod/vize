use vize_atelier_core::{TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_ssr::{
    Allocator, SsrCompilerExperimentalOptions, SsrCompilerOptions,
    compile_ssr_with_sfc_slotted_context, compile_ssr_with_sfc_slotted_context_and_capture,
};
use vize_l0::{
    dump::capture::{CaptureOutcome, StageCapture},
    level::Level,
};

fn compile(source: &str, options: SsrCompilerOptions) -> (vize_l0::String, StageCapture) {
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
    (result.code, capture)
}

#[test]
fn accepted_ssr_pages_describe_the_module_emitter_input() {
    let (code, capture) = compile("<div>hello</div>", SsrCompilerOptions::default());
    assert!(!code.is_empty());
    assert_eq!(capture.outcome, CaptureOutcome::Accepted);
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
            Level::L3
        ]
    );
    assert_eq!(capture.pages[0].text.as_str(), "<div>hello</div>");
    assert!(capture.pages[1].text.contains("[l2-"));
    assert!(capture.pages[3].text.contains("[l3-"));
}

#[test]
fn retained_ssr_module_has_no_native_pages() {
    let (code, capture) = compile(
        "<div>hello</div>",
        SsrCompilerOptions {
            comments: true,
            ..Default::default()
        },
    );
    assert!(!code.is_empty());
    assert!(matches!(capture.outcome, CaptureOutcome::Legacy(_)));
    assert!(capture.pages.is_empty());
}
