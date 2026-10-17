use super::*;
use vize_l1_to_l2::SurfacePreflight;

fn template(source: &str) -> SfcTemplateBlock<'_> {
    SfcTemplateBlock {
        content: std::borrow::Cow::Borrowed(source),
        loc: Default::default(),
        lang: None,
        src: None,
        attrs: Default::default(),
    }
}

#[test]
fn complete_native_error_and_repeated_requests_reuse_the_original_check_once() {
    for source in [
        "<div></div>",
        "<div v-if='a' v-if='b'><p CLASS='x' class='y'></p></div>",
    ] {
        let template = template(source);
        let arena = vize_l0::Allocator::default();
        let (tree, errors) = vize_l1::parse(&arena, source);
        let mut check = Preflight::new(&template, true, false);
        let expected = reject_duplicate_props(&template);
        assert_eq!(check.check(Some((&tree, &errors))), expected.is_ok());
        assert_eq!(check.check(None), expected.is_ok());
        assert_eq!(check.native_calls, 1);
        assert_eq!(check.retained_calls, 0);
        assert_eq!(
            serde_json::to_value(check.finish()).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
    }
}

#[test]
fn native_whole_refusal_keeps_all_old_spans_and_runs_the_callback_once() {
    let source = "<div id='a' id='b'><p v-if[a]='x' v-if[a]='y'/></div>";
    let template = template(source);
    let arena = vize_l0::Allocator::default();
    let (tree, errors) = vize_l1::parse(&arena, source);
    let mut check = Preflight::new(&template, true, false);
    let expected = reject_duplicate_props(&template);
    assert!(expected.is_err());
    assert_eq!(check.check(Some((&tree, &errors))), expected.is_ok());
    assert_eq!(check.check(None), expected.is_ok());
    assert_eq!(check.native_calls, 1);
    assert_eq!(check.retained_calls, 1);
    assert_eq!(
        serde_json::to_value(check.finish()).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
}

#[test]
fn actual_compatibility_entry_checks_once_and_disabled_dialect_stays_disabled() {
    let template = template("<div v-if='a' v-if='b'></div>");
    let mut check = Preflight::new(&template, true, false);
    assert!(!check.check(None));
    assert!(!check.check(None));
    assert_eq!(check.retained_calls, 1);
    assert_eq!(
        serde_json::to_value(check.finish()).unwrap(),
        serde_json::to_value(reject_duplicate_props(&template)).unwrap()
    );
    let mut check = Preflight::new(&template, false, false);
    assert!(check.check(None));
    assert_eq!(check.retained_calls, 0);
    assert!(check.finish().is_ok());
}

#[test]
fn actual_dom_dispatch_observes_native_parse_or_original_compatibility_once() {
    use vize_atelier_core::{
        CodegenExperimentalOptions, CodegenOptions,
        options::{CodegenMode, CustomElementMatcher, TemplateSyntaxMode},
    };
    for (source, native, retained) in [
        ("<div></div>", 1, 0),
        ("<div v-x:[a]foo='x' v-x:foo='y'></div>", 1, 0),
        ("<div/>", 0, 1),
        ("<div @[event].stop='handler'></div>", 1, 0),
        ("<div id='a' id='b'></div>", 0, 1),
    ] {
        for captured in [false, true] {
            let template = template(source);
            let arena = vize_l0::Allocator::default();
            let mut check = Preflight::new(&template, true, false);
            let mut capture = vize_l0::dump::capture::StageCapture::new("dom");
            let _ = vize_atelier_dom::compile_sfc_template_with_preflight(
                &arena,
                source,
                vize_atelier_dom::DomCompilerOptions {
                    mode: CodegenMode::Module,
                    prefix_identifiers: true,
                    ..Default::default()
                },
                TemplateSyntaxMode::Standard,
                None,
                CustomElementMatcher::default(),
                CodegenOptions::default(),
                CodegenExperimentalOptions::default(),
                false,
                Some(&mut check),
                captured.then_some(&mut capture),
            );
            assert!(
                check.result.is_some(),
                "actual entry must validate {source}"
            );
            assert_eq!(
                (check.native_calls, check.retained_calls),
                (native, retained),
                "{source} captured={captured}"
            );
            assert_eq!(
                serde_json::to_value(check.finish()).unwrap(),
                serde_json::to_value(reject_duplicate_props(&template)).unwrap()
            );
        }
    }
}

#[test]
fn equal_text_from_an_unrelated_tree_cannot_skip_the_original_callback() {
    let original = vize_l0::String::from("<div id='a' id='b'></div>");
    let foreign = original.clone();
    assert!(!core::ptr::eq(original.as_str(), foreign.as_str()));
    let template = template(&original);
    let arena = vize_l0::Allocator::default();
    let (tree, errors) = vize_l1::parse(&arena, &foreign);
    let mut check = Preflight::new(&template, true, false);
    assert!(!check.check(Some((&tree, &errors))));
    assert_eq!(check.retained_calls, 1);
    assert_eq!(
        serde_json::to_value(check.finish()).unwrap(),
        serde_json::to_value(reject_duplicate_props(&template)).unwrap()
    );
}

#[test]
fn rejected_preflight_preserves_complete_error_and_the_original_sidecar_boundary() {
    use crate::{
        TemplateCompileOptions,
        compile_template::{TemplateBlockCompileContext, compile_template_block_with_capture},
    };
    use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
    for source in [
        "<div v-if='a' v-if='b'></div>",
        "<div v-x:[a]foo='x' v-x:foo='y'></div>",
    ] {
        let template = template(source);
        let arena = vize_l0::Allocator::default();
        let mut capture = vize_l0::dump::capture::StageCapture::new("original");
        capture.option("original", "retained");
        vize_l0::dump::capture::CaptureSink::page(
            &mut capture,
            vize_l0::level::Level::L1,
            "original",
            || "retained".into(),
        );
        let before = capture.clone();
        let result = compile_template_block_with_capture(
            &arena,
            &template,
            &TemplateCompileOptions::default(),
            &CustomElementMatcher::default(),
            TemplateBlockCompileContext {
                scope_id: "",
                apply_scope_id: false,
                has_scoped: false,
                slotted: false,
                is_ts: false,
                inline: false,
                component_name: None,
                experimental_self_component: false,
                bindings: None,
                croquis: None,
            },
            TemplateSyntaxMode::Standard,
            &CodegenOptions::default(),
            Some(&mut capture),
        );
        assert_eq!(
            serde_json::to_value(result.err()).unwrap(),
            serde_json::to_value(reject_duplicate_props(&template).err()).unwrap()
        );
        assert_eq!(capture, before);
    }
}

#[test]
fn differing_title_and_comment_policies_keep_the_complete_callback_result_once() {
    for (source, comments, expected_error) in [
        ("<title><g id='a' id='b'></g></title>", false, false),
        (
            "<svg><title><g id='a' id='b'></g></title></svg>",
            false,
            false,
        ),
        (
            "<svg><foreignObject><title><g id='a' id='b'></g></title></foreignObject></svg>",
            false,
            false,
        ),
        (
            "<div id='a' id='b'><svg><title><g id='x' id='y'></g></title></svg></div>",
            false,
            true,
        ),
        ("<div // v-x:[a]foo='x' v-x:foo='y'\n></div>", true, true),
    ] {
        let template = template(source);
        let arena = vize_l0::Allocator::default();
        let (tree, errors) = vize_l1::parse_with_options(
            &arena,
            source,
            vize_l1::SurfaceParseOptions {
                experimental_in_tag_comments: comments,
            },
        );
        let mut check = Preflight::new(&template, true, comments);
        let expected = reject_duplicate_props(&template);
        assert_eq!(
            expected.is_err(),
            expected_error,
            "original callback policy: {source}"
        );
        assert_eq!(check.check(Some((&tree, &errors))), expected.is_ok());
        assert_eq!(check.check(None), expected.is_ok());
        assert_eq!((check.native_calls, check.retained_calls), (1, 1));
        assert_eq!(
            serde_json::to_value(check.finish()).unwrap(),
            serde_json::to_value(expected).unwrap(),
            "{source}"
        );
    }
}
