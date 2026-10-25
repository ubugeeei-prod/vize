use super::support::{jsx, options};
use vize_carton::i18n::{Locale, translator};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1_to_l2::native_file::lower_sfc_native;
use vize_l2::lang::js::{JsxFileProducer, JsxMarkupIssueKind};
use vize_patina::native::canonical::{NativeCanonicalLint, NativeCanonicalLintRefusal};

#[test]
fn later_vue_markup_comment_refuses_the_whole_query_without_mutating_the_owner() {
    let a = Allocator::default();
    let source = "<template><p tabindex='1'/><!-- vize-disable a11y/tabindex-no-positive --><p tabindex='2'/></template>";
    let owner = lower_sfc_native(&a, source, options());
    let admitted = owner.admitted().unwrap();
    let before = owner.file().unwrap().file().artifact().node_count();
    let lint = NativeCanonicalLint::from_vue(admitted);
    let messages = translator().for_locale(Locale::En);
    let start = source.find("<!--").unwrap() as u32;
    let end = source.find("-->").unwrap() as u32 + 3;
    assert_eq!(
        lint.tabindex_no_positive(&messages).unwrap_err(),
        NativeCanonicalLintRefusal::MarkupComment {
            span: Span::new(start, end)
        }
    );
    assert_eq!(owner.file().unwrap().file().artifact().node_count(), before);
    assert!(owner.admitted().is_some());
}

#[test]
fn original_jsx_comments_cannot_hide_behind_empty_canonical_containers() {
    for source in [
        "/* prefix */ const view = <div tabindex='1'/>;",
        "const view = <div tabindex='1'>{/* vize-disable */}</div>;",
    ] {
        let a = Allocator::default();
        let owner = jsx(&a, source, true);
        let start = source.find("/*").unwrap() as u32;
        let end = source.find("*/").unwrap() as u32 + 2;
        let Err(error) = NativeCanonicalLint::from_jsx(&owner) else {
            panic!("original comments require host suppression authority")
        };
        assert_eq!(
            error,
            NativeCanonicalLintRefusal::ScriptComment {
                span: Span::new(start, end)
            }
        );
        assert!(owner.file().is_complete());
    }
}

#[test]
fn jsx_original_comment_coordinates_retain_nonzero_utf8_source_block_offsets() {
    let a = Allocator::default();
    let source = "😀\r\n/* original */ const view = <p tabindex='1'/>;";
    let start = source.find("/*").unwrap();
    let text = source.get(start..).unwrap();
    let block = SourceRoot::new(source)
        .unwrap()
        .block(text, start as u32)
        .unwrap();
    let original = oxc_parser::Parser::new(&a, text, oxc_span::SourceType::tsx().with_module(true))
        .parse_observed();
    let mut producer =
        JsxFileProducer::new(&a, original, block, 0).unwrap_or_else(|_| panic!("native block"));
    producer.walk().unwrap();
    let owner = producer
        .finish_markup()
        .unwrap_or_else(|_| panic!("native canonical view"));
    let Err(error) = NativeCanonicalLint::from_jsx(&owner) else {
        panic!("original comment")
    };
    assert_eq!(
        error,
        NativeCanonicalLintRefusal::ScriptComment {
            span: Span::new(start as u32, source.find("*/").unwrap() as u32 + 2)
        }
    );
}

#[test]
fn later_unprovided_jsx_shapes_never_create_a_partial_native_lint_context() {
    for (source, kind) in [
        (
            "const view = <div tabindex='1' {...attrs}/>;",
            JsxMarkupIssueKind::SpreadAttribute,
        ),
        (
            "const view = <div tabindex='1' title='&amp;'/>;",
            JsxMarkupIssueKind::ValueNormalization,
        ),
        (
            "const view = <div tabindex='1'>{value}</div>;",
            JsxMarkupIssueKind::DynamicChild,
        ),
    ] {
        let a = Allocator::default();
        let original =
            oxc_parser::Parser::new(&a, source, oxc_span::SourceType::tsx().with_module(true))
                .parse_observed();
        let block = SourceRoot::new(source).unwrap().whole_block();
        let mut producer = JsxFileProducer::new(&a, original, block, 0)
            .unwrap_or_else(|_| panic!("native original"));
        producer.walk().unwrap();
        let Err(refused) = producer.finish_markup() else {
            panic!("unprovided whole view")
        };
        assert_eq!(refused.issues().first().unwrap().kind, kind);
        if matches!(
            kind,
            JsxMarkupIssueKind::SpreadAttribute | JsxMarkupIssueKind::DynamicChild
        ) {
            assert!(refused.owner().is_none());
            let original = refused.rejected_owner().unwrap();
            assert_eq!(
                original.error(),
                vize_l2::lang::js::JsxFileError::IncompleteFile
            );
            assert!(original.observation().admitted().is_some());
            let file = original.file().unwrap();
            assert!(!file.is_complete());
            assert_eq!(file.artifact().node_count(), 0);
            continue;
        }
        let owner = refused.owner().unwrap();
        assert!(owner.file().is_complete());
        assert_eq!(owner.file().artifact().node_count(), 0);
        // Only JsxMarkupFile can construct the lint context. This retained
        // original JsxFile is deliberately not such a capability.
        assert!(owner.observation().admitted().is_some());
    }
}

#[test]
fn original_vue_expression_holes_remain_whole_file_refusals() {
    let a = Allocator::default();
    let source = "<template><p tabindex='1'>{{ broken + }}</p></template>";
    let owner = lower_sfc_native(&a, source, options());
    assert!(owner.admitted().is_none());
    assert!(!owner.issues().is_empty());
    assert!(owner.template().unwrap().produced().is_some());
}
