use super::support::{jsx, options};
use vize_carton::i18n::{Locale, translator};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1_to_l2::native_file::{NativeSfcIssue, NativeSfcIssueKind, lower_sfc_native};
use vize_l2::lang::js::{JsxFileProducer, JsxMarkupIssueKind};
use vize_patina::native::canonical::{NativeCanonicalLint, NativeCanonicalLintRefusal};

#[test]
fn later_indeterminate_attributes_keep_whole_producer_refusals() {
    for header in ["alt='' :[key]='value'", "alt='' v-bind='attrs'"] {
        let a = Allocator::default();
        let source = vize_l0::cstr!("<template><div><img/><img {header}/></div></template>");
        let owner = lower_sfc_native(&a, &source, options());
        assert!(owner.admitted().is_none(), "{source}");
        assert!(!owner.issues().is_empty(), "{source}");
    }
}

#[test]
fn late_markup_comments_refuse_the_complete_query() {
    let a = Allocator::default();
    let source = "<template><img/><!-- vize-disable a11y/img-alt --><img/></template>";
    let owner = lower_sfc_native(&a, source, options());
    let lint = NativeCanonicalLint::from_vue(owner.admitted().unwrap());
    assert_eq!(
        lint.img_alt(&translator().for_locale(Locale::En))
            .unwrap_err(),
        NativeCanonicalLintRefusal::MarkupComment {
            span: Span::new(
                source.find("<!--").unwrap() as u32,
                source.find("-->").unwrap() as u32 + 3
            )
        }
    );
    assert!(owner.admitted().is_some());
}

#[test]
fn original_program_comments_cannot_be_lost_by_canonical_normalization() {
    for source in [
        "/* prefix */ const view = <img/>;",
        "const view = <div><img/>{/* vize-disable a11y/img-alt */}</div>;",
    ] {
        let a = Allocator::default();
        let owner = jsx(&a, source, true);
        let Err(error) = NativeCanonicalLint::from_jsx(&owner) else {
            panic!("original comment")
        };
        assert_eq!(
            error,
            NativeCanonicalLintRefusal::ScriptComment {
                span: Span::new(
                    source.find("/*").unwrap() as u32,
                    source.find("*/").unwrap() as u32 + 2
                )
            }
        );
        assert!(owner.file().is_complete());
    }
}

#[test]
fn semantic_components_keep_the_original_file_admission_refusal() {
    for source in [
        "<template><IMG src='photo'/></template>",
        "<template><IMG src='photo'></IMG></template>",
        "<template><img is='vue:Other'/></template>",
        "<template><div is='vue:img'/></template>",
    ] {
        let a = Allocator::default();
        let owner = lower_sfc_native(&a, source, options());
        assert!(owner.admitted().is_none());
        assert!(owner.template().is_some());
        assert_eq!(
            owner.issues(),
            &[NativeSfcIssue {
                container_index: None,
                span: Some(Span::new(0, source.len() as u32)),
                kind: NativeSfcIssueKind::FileRejected
            }]
        );
    }
}

#[test]
fn late_semantic_component_refuses_the_complete_original_file() {
    let a = Allocator::default();
    let source = "<template><section><img/><img is='vue:Other'/></section></template>";
    let owner = lower_sfc_native(&a, source, options());
    assert!(owner.admitted().is_none());
    assert!(owner.template().is_some());
    assert_eq!(
        owner.issues(),
        &[NativeSfcIssue {
            container_index: None,
            span: Some(Span::new(0, source.len() as u32)),
            kind: NativeSfcIssueKind::FileRejected
        }]
    );
}

#[test]
fn non_html_image_namespace_is_not_guessed_from_its_name() {
    let a = Allocator::default();
    let source = "<template><svg><img/></svg></template>";
    let owner = lower_sfc_native(&a, source, options());
    let lint = NativeCanonicalLint::from_vue(owner.admitted().unwrap());
    let start = source.find("<img").unwrap() as u32;
    assert_eq!(
        lint.img_alt(&translator().for_locale(Locale::En))
            .unwrap_err(),
        NativeCanonicalLintRefusal::UnsupportedImageElement {
            span: Span::new(start, start + 6)
        }
    );
}

#[test]
fn same_program_nonzero_blocks_keep_full_authored_bytes_and_utf16_positions() {
    for newline in ["\n", "\r\n"] {
        for typescript in [false, true] {
            let a = Allocator::default();
            let source = vize_l0::cstr!(
                "😀{newline}const prefix = '作者😀';{newline}const view = <img></img>;"
            );
            let offset = source.find("const prefix").unwrap();
            let text = source.get(offset..).unwrap();
            let block = SourceRoot::new(&source)
                .unwrap()
                .block(text, offset as u32)
                .unwrap();
            let kind = if typescript {
                oxc_span::SourceType::tsx()
            } else {
                oxc_span::SourceType::jsx()
            }
            .with_module(true);
            let original = oxc_parser::Parser::new(&a, text, kind).parse_observed();
            let mut producer = JsxFileProducer::new(&a, original, block, 0)
                .unwrap_or_else(|_| panic!("native block"));
            producer.walk().unwrap();
            let owner = producer
                .finish_markup()
                .unwrap_or_else(|_| panic!("canonical block"));
            let lint = NativeCanonicalLint::from_jsx(&owner).unwrap();
            let output = super::support::query(&lint, &translator().for_locale(Locale::Ja));
            let range = output.first().unwrap().diagnostic().span;
            let start = source.find("<img>").unwrap();
            assert_eq!(range, Span::new(start as u32, start as u32 + 11));
            assert_eq!(
                source.get(range.start as usize..range.end as usize),
                Some("<img></img>")
            );
            let expected_utf16_start = 2
                + newline.len()
                + "const prefix = '作者😀';".encode_utf16().count()
                + newline.len()
                + "const view = ".len();
            assert_eq!(
                source
                    .get(..range.start as usize)
                    .unwrap()
                    .encode_utf16()
                    .count(),
                expected_utf16_start
            );
        }
    }
}

#[test]
fn v_pre_unknown_directives_and_invalid_expressions_keep_producer_refusals() {
    for source in [
        "<template><img v-pre :alt='x'/></template>",
        "<template><div v-pre><img :[alt]='x'/></div></template>",
        "<template><img v-unknown='x'/></template>",
        "<template><img :alt='broken +'/></template>",
        "<template><img :alt='photo?.description'/></template>",
        "<template><img .alt='text'/></template>",
        "<template><img :ALT.camel='text'/></template>",
        "<template><img v-bind:alt.prop='text'/></template>",
        "<template><img :alt/></template>",
        "<template><img @alt='onAlt'/></template>",
    ] {
        let a = Allocator::default();
        let owner = lower_sfc_native(&a, source, options());
        assert!(owner.admitted().is_none(), "{source}");
        assert!(!owner.issues().is_empty(), "{source}");
    }
}

#[test]
fn jsx_dynamic_values_spreads_and_entities_refuse_before_query_construction() {
    for (body, kind) in [
        (
            "<div><img/><img alt={label}/></div>",
            JsxMarkupIssueKind::DynamicAttribute,
        ),
        (
            "<div><img/><img {...attrs}/></div>",
            JsxMarkupIssueKind::SpreadAttribute,
        ),
        ("<img alt='&amp;'/>", JsxMarkupIssueKind::ValueNormalization),
    ] {
        let a = Allocator::default();
        let source = vize_l0::cstr!("const view = {body};");
        let original =
            oxc_parser::Parser::new(&a, &source, oxc_span::SourceType::tsx().with_module(true))
                .parse_observed();
        let block = SourceRoot::new(&source).unwrap().whole_block();
        let mut producer = JsxFileProducer::new(&a, original, block, 0)
            .unwrap_or_else(|_| panic!("original Program"));
        producer.walk().unwrap();
        let Err(refused) = producer.finish_markup() else {
            panic!("unprovided view")
        };
        assert_eq!(refused.issues().first().unwrap().kind, kind);
        assert!(refused.owner().is_some() || refused.rejected_owner().is_some());
    }
}
