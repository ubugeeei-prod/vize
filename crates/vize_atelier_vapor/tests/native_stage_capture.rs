use vize_atelier_core::{TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_vapor::{
    VaporCompilerExperimentalOptions, VaporCompilerOptions, compile_vapor_with_sfc_context,
    compile_vapor_with_sfc_context_and_capture,
};
use vize_carton::Allocator;
use vize_l0::{
    dump::capture::{CaptureOutcome, StageCapture},
    level::Level,
};

fn compile(source: &str, options: VaporCompilerOptions) -> (vize_carton::String, StageCapture) {
    let allocator = Allocator::new();
    let mut capture = StageCapture::new("vapor");
    let (result, errors) = compile_vapor_with_sfc_context_and_capture(
        &allocator,
        source,
        options.clone(),
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        VaporCompilerExperimentalOptions::default(),
        None,
        &mut capture,
    );
    assert!(errors.is_empty(), "{errors:?}");
    assert!(
        result.error_messages.is_empty(),
        "{:?}",
        result.error_messages
    );
    let (baseline, baseline_errors) = compile_vapor_with_sfc_context(
        &allocator,
        source,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        VaporCompilerExperimentalOptions::default(),
        None,
    );
    assert!(baseline_errors.is_empty(), "{baseline_errors:?}");
    assert_eq!(result.code, baseline.code);
    (result.code, capture)
}

#[test]
fn accepted_vapor_pages_describe_the_checked_emitter_input() {
    let (code, capture) = compile("<div>hello</div>", VaporCompilerOptions::default());
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
            Level::L3,
            Level::L4
        ]
    );
    assert_eq!(capture.pages[0].text.as_str(), "<div>hello</div>");
    assert!(capture.pages[1].text.contains("[l2-"));
    assert!(capture.pages[3].text.contains("[l3-"));
    // Vapor returns a complete backend module. The SFC adapter may later
    // rewrite its imports and render function as it composes the SFC script.
    assert_eq!(capture.pages[6].step, "emit");
    assert_eq!(capture.pages[6].text, code);
}

#[test]
fn retained_vapor_module_has_no_native_pages() {
    let (code, capture) = compile(
        "<div>hello</div>",
        VaporCompilerOptions {
            davinci_retained_lane: true,
            ..Default::default()
        },
    );
    assert!(!code.is_empty());
    assert!(matches!(capture.outcome, CaptureOutcome::Legacy(_)));
    assert!(capture.pages.is_empty());
}
