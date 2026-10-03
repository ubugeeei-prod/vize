use super::support::{
    LOCALES, NeverLookup, complete, expected, parity, parser, registered, selected, span,
};
use vize_l0::{Allocator, Span, cstr};
use vize_l1::markup::NativeLintTagRefusal;
use vize_patina::native::{NativeLintRefusal, NativeSyntaxLint, NativeVHtmlLintError};

#[test]
fn exact_own_pre_and_exact_plus_modified_pre_freeze_all_sink_spellings() {
    for header in [
        "v-pre v-html='x' :innerHTML='x' .outerHTML='x' v-bind:innerHTML='x'",
        "v-html='x' :innerHTML='x' v-pre",
        "v-pre.foo v-pre v-html='x'",
        "v-pre v-pre.foo v-html='x'",
    ] {
        let source = cstr!("<template><Foo {header} /></template>");
        for locale in LOCALES {
            assert_eq!(parity(&source, locale), expected(vec![]));
        }
    }
}

#[test]
fn modified_argument_and_dynamic_pre_refuse_without_erasing_original_clean_vectors() {
    for pre in ["v-pre.foo", "v-pre:arg", "v-pre:[key]"] {
        let source =
            cstr!("<template><div {pre} v-html='own'><Foo :innerHTML='child' /></div></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let parent = owner.children().next().unwrap().into_element().unwrap();
        let child = parent.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        for element in [&parent, &child] {
            assert_eq!(
                lint.no_v_html(element, &NeverLookup).err(),
                Some(NativeVHtmlLintError::Header(NativeLintRefusal::LintTag {
                    reason: NativeLintTagRefusal::AmbiguousVerbatim
                }))
            );
        }
        // This registered rule uses actual Relief directives, not the L2 facade.
        for locale in LOCALES {
            assert_eq!(complete(&registered(&source, locale)), expected(vec![]));
        }
    }
}

#[test]
fn inherited_literal_context_refuses_before_clean_credit_and_catalog_lookup() {
    for (source, depth) in [
        (
            "<template><div v-pre><Foo v-html='x' :innerHTML='y' /></div></template>",
            1,
        ),
        (
            "<template><slot v-pre><Foo v-html='x' /></slot></template>",
            1,
        ),
        (
            "<template><slot v-pre><div><Foo v-html='x' /></div></slot></template>",
            2,
        ),
        (
            "<template><div v-pre><slot><Foo v-html='x' /></slot></div></template>",
            2,
        ),
        (
            "<template><div v-pre><Foo v-unknown='x' :title.='x' :[key]='x' /></div></template>",
            1,
        ),
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let mut element = owner.children().next().unwrap().into_element().unwrap();
        for _ in 0..depth {
            element = element.children().next().unwrap().into_element().unwrap();
        }
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let head = if source.contains("v-unknown") {
            "<Foo v-unknown='x' :title.='x' :[key]='x' />"
        } else if source.contains(":innerHTML") {
            "<Foo v-html='x' :innerHTML='y' />"
        } else {
            "<Foo v-html='x' />"
        };
        assert!(element.lint_tag().unwrap().header_is_literal());
        assert_eq!(
            lint.no_v_html(&element, &NeverLookup).err(),
            Some(NativeVHtmlLintError::InheritedLiteralContext {
                span: span(source, head)
            })
        );
        for locale in LOCALES {
            assert_eq!(complete(&registered(source, locale)), expected(vec![]));
        }
    }
}

#[test]
fn own_exact_pre_authenticates_frozen_sink_headers_inside_inherited_context() {
    for source in [
        "<template><div v-pre><Foo v-pre v-html='x' :innerHTML='y' /></div></template>",
        "<template><slot v-pre><Foo v-html='x' v-pre /></slot></template>",
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let parent = owner.children().next().unwrap().into_element().unwrap();
        let element = parent.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        assert!(element.lint_tag().unwrap().header_is_literal());
        assert!(lint.no_v_html(&element, &NeverLookup).unwrap().is_empty());
        for locale in LOCALES {
            assert_eq!(complete(&registered(source, locale)), expected(vec![]));
        }
    }
}

#[test]
fn malformed_own_pre_headers_refuse_and_keep_every_original_parser_error() {
    for source in [
        "<template><Foo v-pre v-html.='x' /></template>",
        "<template><Foo v-html.='x' v-pre /></template>",
        "<template><Foo v-pre :innerHTML.='x' /></template>",
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let head = span(
            source,
            if source.contains("v-html.") {
                "v-html."
            } else {
                ":innerHTML."
            },
        );
        assert_eq!(
            lint.no_v_html(&element, &NeverLookup).err(),
            Some(NativeVHtmlLintError::Header(
                NativeLintRefusal::UnsupportedDirective { span: head }
            ))
        );
        for locale in LOCALES {
            assert_eq!(
                complete(&registered(source, locale)),
                expected(vec![parser(
                    "error",
                    "Directive modifier is expected.",
                    Span::new(head.end, head.end + 1)
                )])
            );
        }
    }
}

#[test]
fn inherited_template_preserves_original_provider_refusal_and_clean_registered_vector() {
    let source = "<template><div v-pre><template v-html='x'></template></div></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let parent = owner.children().next().unwrap().into_element().unwrap();
    let element = parent.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    assert_eq!(
        lint.no_v_html(&element, &NeverLookup).err(),
        Some(NativeVHtmlLintError::Header(NativeLintRefusal::LintTag {
            reason: NativeLintTagRefusal::InheritedTemplate
        }))
    );
    for locale in LOCALES {
        assert_eq!(complete(&registered(source, locale)), expected(vec![]));
    }
}
