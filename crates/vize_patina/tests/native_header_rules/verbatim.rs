use super::support::{
    HeaderRule, LOCALES, NeverLookup, complete, expected, parity, registered, selected, span,
    warning,
};
use vize_l0::{Allocator, Span, cstr};
use vize_l1::markup::NativeLintTagRefusal;
use vize_patina::Locale;
use vize_patina::native::{NativeLintRefusal, NativeSyntaxLint};

#[test]
fn plain_exact_and_inherited_pre_freeze_bindings_without_exempting_native_tags() {
    for source in [
        "<template><marquee v-pre autofocus accesskey :autofocus='x' .accesskey='y'></marquee></template>",
        "<template><div v-pre><blink autofocus accesskey :autofocus='x' v-bind:accesskey='y'></blink></div></template>",
        "<template><marquee v-pre.foo v-pre autofocus accesskey></marquee></template>",
        "<template><marquee v-pre v-pre.foo autofocus accesskey></marquee></template>",
    ] {
        for rule in HeaderRule::ALL {
            for locale in LOCALES {
                assert_eq!(parity(source, locale, rule)["warning_count"], 1, "{source}");
            }
        }
    }
    for source in [
        "<template><Foo v-pre autofocus accesskey /></template>",
        "<template><div v-pre><Foo autofocus accesskey /></div></template>",
    ] {
        for rule in HeaderRule::ALL {
            assert_eq!(parity(source, Locale::En, rule), expected(vec![]));
        }
    }
}

#[test]
fn inherited_pre_keeps_malformed_directive_spellings_as_literal_static_attributes() {
    for source in [
        "<template><div v-pre><marquee autofocus accesskey :title.='x'></marquee></div></template>",
        "<template><div v-pre><blink autofocus accesskey v-bind:title..camel='x' v-unknown='x' :[key]='x'></blink></div></template>",
    ] {
        for rule in HeaderRule::ALL {
            for locale in LOCALES {
                assert_eq!(parity(source, locale, rule)["warning_count"], 1);
            }
        }
    }
}

#[test]
fn own_exact_pre_structural_template_keeps_complete_registered_rule_output() {
    let source = "<template><template v-pre v-if='ok' autofocus accesskey :autofocus='ignored'></template></template>";
    for rule in HeaderRule::ALL {
        for locale in LOCALES {
            let diagnostics = if matches!(rule, HeaderRule::Distracting) {
                vec![]
            } else {
                vec![warning(
                    rule,
                    locale,
                    span(source, rule.attribute()),
                    "template",
                )]
            };
            assert_eq!(parity(source, locale, rule), expected(diagnostics));
        }
    }
}

#[test]
fn inherited_structural_template_refuses_without_losing_the_registered_oracle() {
    let source = "<!--🦀--><template><div v-pre><template v-if='ok' autofocus accesskey></template></div></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let parent = owner.children().next().unwrap().into_element().unwrap();
    let element = parent.children().next().unwrap().into_element().unwrap();
    for rule in HeaderRule::ALL {
        assert_eq!(
            rule.check(&lint, &element, &NeverLookup).err(),
            Some(NativeLintRefusal::LintTag {
                reason: NativeLintTagRefusal::InheritedTemplate
            })
        );
        for locale in LOCALES {
            let diagnostics = if matches!(rule, HeaderRule::Distracting) {
                vec![]
            } else {
                vec![warning(
                    rule,
                    locale,
                    span(source, rule.attribute()),
                    "template",
                )]
            };
            assert_eq!(
                complete(&registered(source, locale, rule)),
                expected(diagnostics)
            );
        }
    }
}

#[test]
fn modified_and_argument_pre_scopes_refuse_authentic_lint_ambiguity() {
    for pre in ["v-pre.foo", "v-pre:arg", "v-pre:[key]"] {
        let opening = cstr!("<marquee {pre} autofocus accesskey>");
        let child_opening = "<blink autofocus accesskey>";
        let source = cstr!("<template>{opening}{child_opening}</blink></marquee></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let parent = owner.children().next().unwrap().into_element().unwrap();
        let child = parent.children().next().unwrap().into_element().unwrap();
        for rule in HeaderRule::ALL {
            for element in [&parent, &child] {
                assert_eq!(
                    rule.check(&lint, element, &NeverLookup).err(),
                    Some(NativeLintRefusal::LintTag {
                        reason: NativeLintTagRefusal::AmbiguousVerbatim
                    })
                );
            }
            for locale in LOCALES {
                let ranges = if matches!(rule, HeaderRule::Distracting) {
                    [span(&source, &opening), span(&source, child_opening)]
                } else {
                    let first = span(&source, rule.attribute());
                    let child_start = span(&source, child_opening).start as usize;
                    let relative = span(&source[child_start..], rule.attribute());
                    [
                        first,
                        Span::new(
                            child_start as u32 + relative.start,
                            child_start as u32 + relative.end,
                        ),
                    ]
                };
                assert_eq!(
                    complete(&registered(&source, locale, rule)),
                    expected(vec![
                        warning(rule, locale, ranges[0], "marquee"),
                        warning(rule, locale, ranges[1], "blink")
                    ])
                );
            }
        }
    }
}
