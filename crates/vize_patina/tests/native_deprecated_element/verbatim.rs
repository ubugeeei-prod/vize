use super::support::{
    LOCALES, NeverLookup, complete, expected, parity, registered, selected, span, warning,
};
use vize_l0::{Allocator, cstr};
use vize_l1::markup::NativeLintTagRefusal;
use vize_patina::native::{NativeLintRefusal, NativeSyntaxLint};

#[test]
fn exact_and_inherited_pre_keep_deprecated_tag_findings_without_binding_inference() {
    for (source, opening) in [
        (
            "<template><center v-pre :title='x' .id='y'></center></template>",
            "<center v-pre :title='x' .id='y'>",
        ),
        (
            "<template><div v-pre><center :title.='x' v-bind:title..camel='y' v-unknown='z' :[key]='x'></center></div></template>",
            "<center :title.='x' v-bind:title..camel='y' v-unknown='z' :[key]='x'>",
        ),
        (
            "<template><center v-pre.foo v-pre></center></template>",
            "<center v-pre.foo v-pre>",
        ),
        (
            "<template><center v-pre v-pre.foo></center></template>",
            "<center v-pre v-pre.foo>",
        ),
    ] {
        for locale in LOCALES {
            assert_eq!(
                parity(source, locale),
                expected(vec![warning(locale, span(source, opening), "center")])
            );
        }
    }
    for source in [
        "<template><Center v-pre /></template>",
        "<template><div v-pre><Center /></div></template>",
        "<template><template v-pre v-if='ok'></template></template>",
    ] {
        for locale in LOCALES {
            assert_eq!(parity(source, locale), expected(vec![]));
        }
    }
}

#[test]
fn modified_and_argument_pre_refuse_without_losing_original_deprecated_findings() {
    for pre in ["v-pre.foo", "v-pre:arg", "v-pre:[key]"] {
        let opening = cstr!("<center {pre}>");
        let source = cstr!("<template>{opening}<blink /></center></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let parent = owner.children().next().unwrap().into_element().unwrap();
        let child = parent.children().next().unwrap().into_element().unwrap();
        for element in [&parent, &child] {
            assert_eq!(
                lint.deprecated_element(element, &NeverLookup).err(),
                Some(NativeLintRefusal::LintTag {
                    reason: NativeLintTagRefusal::AmbiguousVerbatim
                })
            );
        }
        for locale in LOCALES {
            assert_eq!(
                complete(&registered(&source, locale)),
                expected(vec![
                    warning(locale, span(&source, &opening), "center"),
                    warning(locale, span(&source, "<blink />"), "blink"),
                ])
            );
        }
    }
}

#[test]
fn inherited_template_category_refusal_remains_before_even_nonmatching_rule_lookup() {
    let source = "<template><div v-pre><template v-if='ok'></template></div></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let parent = owner.children().next().unwrap().into_element().unwrap();
    let element = parent.children().next().unwrap().into_element().unwrap();
    assert_eq!(
        lint.deprecated_element(&element, &NeverLookup).err(),
        Some(NativeLintRefusal::LintTag {
            reason: NativeLintTagRefusal::InheritedTemplate
        })
    );
    for locale in LOCALES {
        assert_eq!(complete(&registered(source, locale)), expected(vec![]));
    }
}
