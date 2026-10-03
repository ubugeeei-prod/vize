use super::support::{
    LOCALES, NeverLookup, complete, expected, parity, parser, registered, selected, span, warning,
};
use vize_l0::{Allocator, Span, cstr};
use vize_l1::markup::NativeLintTagRefusal;
use vize_patina::native::{NativeLintRefusal, NativeSyntaxLint};

const ALIGN: &str = "CSS `text-align` or `margin: auto`";

#[test]
fn exact_own_and_inherited_pre_match_only_authored_static_policy_names() {
    for source in [
        "<template><div v-pre align='yes' :align='no' .align='no' v-bind:align='no' /></template>",
        "<template><div v-pre><span align='yes' :align='no' v-bind:align='no' /></div></template>",
        "<template><div v-pre.foo v-pre align='yes' /></template>",
        "<template><div v-pre v-pre.foo align='yes' /></template>",
    ] {
        let tag = if source.contains("<span") {
            "span"
        } else {
            "div"
        };
        for locale in LOCALES {
            assert_eq!(
                parity(source, locale),
                expected(vec![warning(
                    locale,
                    span(source, "align='yes'"),
                    tag,
                    "align",
                    ALIGN
                )])
            );
        }
    }
}

#[test]
fn inherited_literal_heads_keep_dynamic_custom_and_empty_modifiers_opaque() {
    let source = "<template><div v-pre><span align='yes' :align.='no' v-bind:align..camel='no' v-unknown='x' :[key]='x' /></div></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![warning(
                locale,
                span(source, "align='yes'"),
                "span",
                "align",
                ALIGN
            )])
        );
    }
}

#[test]
fn modified_and_argument_pre_refuse_parent_and_child_without_erasing_product_output() {
    for pre in ["v-pre.foo", "v-pre:arg", "v-pre:[key]"] {
        let source = cstr!(
            "<template><div {pre} align='one' :align='x'><span align='two' :align='y'></span></div></template>"
        );
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let parent = owner.children().next().unwrap().into_element().unwrap();
        let child = parent.children().next().unwrap().into_element().unwrap();
        for element in [&parent, &child] {
            assert_eq!(
                lint.deprecated_attr(element, &NeverLookup).err(),
                Some(NativeLintRefusal::LintTag {
                    reason: NativeLintTagRefusal::AmbiguousVerbatim
                })
            );
        }
        for locale in LOCALES {
            assert_eq!(
                complete(&registered(&source, locale)),
                expected(vec![
                    warning(locale, span(&source, "align='one'"), "div", "align", ALIGN),
                    warning(locale, span(&source, "align='two'"), "span", "align", ALIGN)
                ])
            );
        }
    }
}

#[test]
fn inherited_structural_template_keeps_typed_refusal_and_full_registered_warning() {
    let source =
        "<template><div v-pre><template v-if='ok' align='yes'></template></div></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let parent = owner.children().next().unwrap().into_element().unwrap();
    let element = parent.children().next().unwrap().into_element().unwrap();
    assert_eq!(
        lint.deprecated_attr(&element, &NeverLookup).err(),
        Some(NativeLintRefusal::LintTag {
            reason: NativeLintTagRefusal::InheritedTemplate
        })
    );
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![warning(
                locale,
                span(source, "align='yes'"),
                "template",
                "align",
                ALIGN
            )])
        );
    }
}

#[test]
fn own_pre_empty_modifiers_refuse_before_warning_and_preserve_actual_parser_error() {
    for source in [
        "<template><div v-pre align='yes' :title.='x' /></template>",
        "<template><div align='yes' :title.='x' v-pre /></template>",
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let head = span(source, ":title.");
        assert_eq!(
            lint.deprecated_attr(&element, &NeverLookup).err(),
            Some(NativeLintRefusal::UnsupportedDirective { span: head })
        );
        for locale in LOCALES {
            assert_eq!(
                complete(&registered(source, locale)),
                expected(vec![
                    warning(locale, span(source, "align='yes'"), "div", "align", ALIGN),
                    parser(
                        "error",
                        "Directive modifier is expected.",
                        Span::new(head.end, head.end + 1)
                    )
                ])
            );
        }
    }
}
