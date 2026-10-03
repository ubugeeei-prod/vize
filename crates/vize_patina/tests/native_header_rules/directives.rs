use super::support::{
    HeaderRule, LOCALES, NeverLookup, complete, expected, parser, registered, selected, span,
    warning,
};
use vize_l0::{Allocator, Span, cstr};
use vize_patina::native::{NativeLintRefusal, NativeSyntaxLint};

#[test]
fn later_dynamic_object_and_custom_headers_refuse_whole_results_without_catalog_lookup() {
    for (head, unresolved) in [
        (":[key]", true),
        (".[key]", true),
        ("@[key]", true),
        ("v-bind:[key]", true),
        ("v-bind", true),
        ("v-bind.prop", true),
        ("v-unknown", false),
    ] {
        let opening = cstr!("<marquee autofocus accesskey {head}='value'>");
        let source = cstr!("<template>{opening}</marquee></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let range = span(&source, head);
        let refusal = if unresolved {
            NativeLintRefusal::UnresolvedBinding { span: range }
        } else {
            NativeLintRefusal::UnsupportedDirective { span: range }
        };
        for rule in HeaderRule::ALL {
            assert_eq!(
                rule.check(&lint, &element, &NeverLookup).err(),
                Some(refusal)
            );
            for locale in LOCALES {
                let warning_span = if matches!(rule, HeaderRule::Distracting) {
                    span(&source, &opening)
                } else {
                    span(&source, rule.attribute())
                };
                assert_eq!(
                    complete(&registered(&source, locale, rule)),
                    expected(vec![warning(rule, locale, warning_span, "marquee")])
                );
            }
        }
        assert!(owner.component().carrier().errors.is_empty());
    }
}

#[test]
fn empty_modifier_segments_refuse_and_keep_the_unfiltered_reportable_parser_error() {
    for head in [
        ":title.",
        ":title..prop",
        ".title.",
        "v-bind:title..camel",
        "@click.",
        "v-on:click.",
        "v-model:title.",
    ] {
        for pre in ["", "v-pre ", "__after__"] {
            let header = if pre == "__after__" {
                cstr!("autofocus accesskey {head}='x' v-pre")
            } else {
                cstr!("{pre}autofocus accesskey {head}='x'")
            };
            let opening = cstr!("<marquee {header}>");
            let source = cstr!("<template>{opening}</marquee></template>");
            let arena = Allocator::default();
            let owner = selected(&arena, &source);
            let lint = NativeSyntaxLint::new(&owner).unwrap();
            let element = owner.children().next().unwrap().into_element().unwrap();
            let head_span = span(&source, head);
            let local_error = head.find("..").map_or(head.len(), |index| index + 1);
            let error_start = head_span.start + u32::try_from(local_error).unwrap();
            let parse_error = parser(
                "error",
                "Directive modifier is expected.",
                Span::new(error_start, error_start + 1),
            );
            for rule in HeaderRule::ALL {
                assert_eq!(
                    rule.check(&lint, &element, &NeverLookup).err(),
                    Some(NativeLintRefusal::UnsupportedDirective { span: head_span }),
                    "{source}: {rule:?}"
                );
                let finding_span = if matches!(rule, HeaderRule::Distracting) {
                    span(&source, &opening)
                } else {
                    span(&source, rule.attribute())
                };
                for locale in LOCALES {
                    assert_eq!(
                        complete(&registered(&source, locale, rule)),
                        expected(vec![
                            warning(rule, locale, finding_span, "marquee"),
                            parse_error.clone()
                        ]),
                        "{source}: {rule:?}"
                    );
                }
            }
            assert!(owner.component().carrier().errors.is_empty());
        }
    }
}
