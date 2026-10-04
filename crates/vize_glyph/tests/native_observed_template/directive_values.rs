//! Strict values share the actual attribute visit while lower opaque contracts stay exact.

use super::{options, preobserved, selected};
use vize_glyph::native_doc::{
    LineEnding, NativeTemplateRefusal, NativeTemplateValuePolicy, ObservedNativeTemplateRefusal,
    native_template_document, observed_native_template_document,
    observed_native_template_document_with_policy, print,
};
use vize_l0::{Allocator, Span};

#[test]
fn lower_receivers_keep_complete_opaque_directive_values_and_default_policy_outputs() {
    for (head, value) in [
        (":id", "a+b"),
        ("@click", "f(...a)"),
        ("v-if", " a+ "),
        ("v-custom", "raw &amp; bytes"),
        ("#default", "{a}"),
        ("v-bind:[key]", "a&#43;b"),
        (":id", ""),
    ] {
        let source =
            format!("<!--前--><template><p {head}=\"{value}\">{{{{1n}}}}</p></template><!--尾-->");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let lower = observed_native_template_document(&owner, &arena).unwrap();
        let explicit = observed_native_template_document_with_policy(
            &owner,
            &arena,
            NativeTemplateValuePolicy::PreserveOpaque,
        )
        .unwrap();
        let original = preobserved(&owner);
        let refs = original.iter().collect::<std::vec::Vec<_>>();
        let borrowed = native_template_document(&owner, &refs, &arena).unwrap();
        let expected = format!("<p {head}=\"{value}\">{{{{ 1n }}}}</p>");
        for doc in [lower.document(), explicit.document(), borrowed.document()] {
            assert_eq!(print(doc, &options(200, LineEnding::Lf)), expected);
        }
        assert_eq!(lower.operands().len(), 1);
        assert_eq!(lower.operands()[0].raw_content(), "1n");
        assert!(
            lower.operands()[0]
                .admitted_for(
                    &owner,
                    owner
                        .children()
                        .next()
                        .unwrap()
                        .into_element()
                        .unwrap()
                        .children()
                        .next()
                        .unwrap()
                )
                .is_some()
        );
    }
}

#[test]
fn strict_recognized_value_visit_refuses_exact_original_authored_span_before_body_observation() {
    for (head, value) in [
        (":id", "a+b"),
        ("@click", "f(...a)"),
        ("v-if", " a+ "),
        ("v-custom", "raw &amp; bytes"),
        ("#default", "{a}"),
        ("v-bind:[key]", "a&#43;b"),
        (":id", ""),
    ] {
        let source =
            format!("<!--前--><template><p {head}=\"{value}\">{{{{1n}}}}</p></template><!--尾-->");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let start = source.find(&format!("\"{value}\"")).unwrap() + 1;
        let span = Span::new(start as u32, (start + value.len()) as u32);
        let failure = observed_native_template_document_with_policy(
            &owner,
            &arena,
            NativeTemplateValuePolicy::RefuseValuedDirectives,
        )
        .unwrap_err();
        assert_eq!(
            failure.refusal(),
            ObservedNativeTemplateRefusal::Document {
                index: 0,
                refusal: NativeTemplateRefusal::DirectiveValue { span }
            }
        );
        assert!(core::ptr::eq(failure.original(), &owner));
        assert!(failure.operands().is_empty());
        assert!(failure.interpolation_failure().is_none());
        assert_eq!(span.slice(&source), value);
        assert!(core::ptr::eq(
            owner.component().block().root_source(),
            source.as_str()
        ));
        assert_eq!(
            owner.component().block().source(),
            format!("<p {head}=\"{value}\">{{{{1n}}}}</p>")
        );
    }
}

#[test]
fn strict_late_value_keeps_original_completed_prefix_and_static_attribute_values_still_format() {
    let source = "<template>{{/*x*/1&#110;}}<p :id=\"a&#43;b\">{{later}}</p>{{tail}}</template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let failure = observed_native_template_document_with_policy(
        &owner,
        &arena,
        NativeTemplateValuePolicy::RefuseValuedDirectives,
    )
    .unwrap_err();
    let start = source.find("a&#43;b").unwrap() as u32;
    let span = Span::new(start, start + 7);
    assert_eq!(
        failure.refusal(),
        ObservedNativeTemplateRefusal::Document {
            index: 1,
            refusal: NativeTemplateRefusal::DirectiveValue { span }
        }
    );
    assert_eq!(failure.operands().len(), 1);
    let first = &failure.operands()[0];
    assert_eq!(first.raw_content(), "/*x*/1&#110;");
    assert_eq!(first.syntax().source().text(), "/*x*/1n");
    assert_eq!(
        first.syntax().comments().next().unwrap().text().unwrap(),
        "/*x*/"
    );
    assert!(first.syntax().source().decode_map().is_some());
    assert!(
        first
            .admitted_for(&owner, owner.children().next().unwrap())
            .is_some()
    );
    assert!(failure.interpolation_failure().is_none());
    assert_eq!(span.slice(source), "a&#43;b");
    assert!(
        owner
            .component()
            .block()
            .source()
            .ends_with("{{later}}</p>{{tail}}")
    );
    let static_source = "<template><p id=\"a+b &amp; c\">{{1n}}</p></template>";
    let static_owner = selected(&arena, static_source);
    let complete = observed_native_template_document_with_policy(
        &static_owner,
        &arena,
        NativeTemplateValuePolicy::RefuseValuedDirectives,
    )
    .unwrap();
    assert_eq!(
        print(complete.document(), &options(200, LineEnding::Lf)),
        "<p id=\"a+b &amp; c\">{{ 1n }}</p>"
    );
    assert_eq!(complete.operands().len(), 1);
}

#[test]
fn actual_unquoted_value_visit_and_original_head_or_recovery_priority_remain_distinct() {
    let arena = Allocator::default();
    let source = "<template><p :id=a&#43;b>{{1n}}</p></template>";
    let owner = selected(&arena, source);
    let lower = observed_native_template_document(&owner, &arena).unwrap();
    assert_eq!(
        print(lower.document(), &options(200, LineEnding::Lf)),
        "<p :id=a&#43;b>{{ 1n }}</p>"
    );
    let strict = observed_native_template_document_with_policy(
        &owner,
        &arena,
        NativeTemplateValuePolicy::RefuseValuedDirectives,
    )
    .unwrap_err();
    let span = Span::new(17, 24);
    assert_eq!(span.slice(source), "a&#43;b");
    assert_eq!(
        strict.refusal(),
        ObservedNativeTemplateRefusal::Document {
            index: 0,
            refusal: NativeTemplateRefusal::DirectiveValue { span }
        }
    );
    assert!(strict.operands().is_empty());
    assert!(strict.interpolation_failure().is_none());
    let source = "<template><p id=a&#43;b v-pre>{{raw}}</p>{{1n}}</template>";
    let owner = selected(&arena, source);
    let strict = observed_native_template_document_with_policy(
        &owner,
        &arena,
        NativeTemplateValuePolicy::RefuseValuedDirectives,
    )
    .unwrap();
    assert_eq!(
        print(strict.document(), &options(200, LineEnding::Lf)),
        "<p id=a&#43;b v-pre>{{raw}}</p>{{ 1n }}"
    );
    assert_eq!(strict.operands().len(), 1);
    for source in [
        "<template><p v-=\"x\">{{1n}}</p></template>",
        "<template><p :id= >{{1n}}</p></template>",
    ] {
        let owner = selected(&arena, source);
        let lower = observed_native_template_document(&owner, &arena).unwrap_err();
        let strict = observed_native_template_document_with_policy(
            &owner,
            &arena,
            NativeTemplateValuePolicy::RefuseValuedDirectives,
        )
        .unwrap_err();
        assert!(matches!(
            lower.refusal(),
            ObservedNativeTemplateRefusal::Document {
                index: 0,
                refusal: NativeTemplateRefusal::Template(_)
            }
        ));
        assert_eq!(strict.refusal(), lower.refusal());
        assert!(lower.operands().is_empty());
        assert!(strict.operands().is_empty());
        assert!(core::ptr::eq(strict.original(), &owner));
        assert_eq!(
            strict.original().component().block().source(),
            owner.component().block().source()
        );
    }
}
