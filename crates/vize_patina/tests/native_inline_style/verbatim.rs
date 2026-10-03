use super::support::{
    LOCALES, NeverLookup, complete, expected, parity, registered, selected, span, warning,
};
use vize_l0::{Allocator, Span, cstr};
use vize_l1::markup::NativeLintTagRefusal;
use vize_patina::native::{NativeLintRefusal, NativeSyntaxLint};

#[test]
fn exact_own_and_inherited_pre_keep_only_literal_style_matches() {
    for source in [
        "<template><Foo v-pre style='yes' :style='no' .style='no' v-bind:style='no' /></template>",
        "<template><div v-pre><Foo style='yes' :style='no' v-bind:style='no' /></div></template>",
        "<template><template v-pre v-if='ok' style='yes' :style='no'></template></template>",
        "<template><div v-pre.foo v-pre style='yes' /></template>",
        "<template><div v-pre v-pre.foo style='yes' /></template>",
    ] {
        for locale in LOCALES {
            assert_eq!(
                parity(source, locale),
                expected(vec![warning(locale, span(source, "style='yes'"))])
            );
        }
    }
}

#[test]
fn inherited_literal_directive_errors_and_dynamic_spellings_remain_opaque() {
    let source = "<template><div v-pre><Foo style='yes' :style.='no' v-bind:style..camel='no' v-unknown='x' :[key]='x' /></div></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![warning(locale, span(source, "style='yes'"))])
        );
    }
}

#[test]
fn ambiguous_pre_receipts_refuse_without_erasing_complete_registered_warnings() {
    for pre in ["v-pre.foo", "v-pre:arg", "v-pre:[key]"] {
        let source = cstr!(
            "<template><div {pre} style='one' :style='x'><span style='two' :style='y'></span></div></template>"
        );
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let parent = owner.children().next().unwrap().into_element().unwrap();
        let child = parent.children().next().unwrap().into_element().unwrap();
        for element in [&parent, &child] {
            assert_eq!(
                lint.no_inline_style(element, &NeverLookup).err(),
                Some(NativeLintRefusal::LintTag {
                    reason: NativeLintTagRefusal::AmbiguousVerbatim
                })
            );
        }
        for locale in LOCALES {
            assert_eq!(
                complete(&registered(&source, locale)),
                expected(vec![
                    warning(locale, span(&source, "style='one'")),
                    warning(locale, span(&source, "style='two'"))
                ])
            );
        }
    }
}

#[test]
fn inherited_template_retains_the_original_provider_refusal() {
    let source = "<!--🦀--><template><div v-pre><template v-if='ok' style='yes'></template></div></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let parent = owner.children().next().unwrap().into_element().unwrap();
    let element = parent.children().next().unwrap().into_element().unwrap();
    assert_eq!(
        lint.no_inline_style(&element, &NeverLookup).err(),
        Some(NativeLintRefusal::LintTag {
            reason: NativeLintTagRefusal::InheritedTemplate
        })
    );
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![warning(locale, span(source, "style='yes'"))])
        );
    }
}

#[test]
fn own_pre_empty_modifiers_refuse_before_any_warning_or_lookup() {
    for source in [
        "<template><Foo v-pre style='yes' :title.='x' /></template>",
        "<template><Foo style='yes' :title.='x' v-pre /></template>",
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let head = span(source, ":title.");
        assert_eq!(
            lint.no_inline_style(&element, &NeverLookup).err(),
            Some(NativeLintRefusal::UnsupportedDirective { span: head })
        );
        for locale in LOCALES {
            assert_eq!(
                complete(&registered(source, locale)),
                expected(vec![
                    warning(locale, span(source, "style='yes'")),
                    super::support::parser(
                        "error",
                        "Directive modifier is expected.",
                        Span::new(head.end, head.end + 1)
                    )
                ])
            );
        }
    }
}
