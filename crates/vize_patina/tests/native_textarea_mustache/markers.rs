use super::support::{LOCALES, error, expected, parity, selected, span};
use vize_l0::{Allocator, Span, cstr};
use vize_patina::{
    Rule, RuleCategory, Severity, native::NativeSyntaxLint, rules::vue::NoTextareaMustache,
};

#[test]
fn exact_rule_metadata_and_default_error_are_preserved() {
    let meta = NoTextareaMustache.meta();
    assert_eq!(meta.name, "vue/no-textarea-mustache");
    assert_eq!(meta.category, RuleCategory::Essential);
    assert_eq!(meta.default_severity, Severity::Error);
    assert_eq!(meta.fixable, false);
}

#[test]
fn direct_marker_retains_full_delimiters_and_complete_three_locale_output() {
    let source = "<template><textarea>{{ message }}</textarea></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![error(locale, Span::new(20, 33))])
        );
    }
}

#[test]
fn empty_whitespace_and_invalid_expression_contents_remain_opaque_markers() {
    for body in [
        "",
        " ",
        "\n\t",
        "/* only comment */",
        "??",
        "a +",
        "a =&gt; b",
        "未定義 +",
    ] {
        let marker = cstr!("{{{{{body}}}}}");
        let source = cstr!("<template><textarea>{marker}</textarea></template>");
        for locale in LOCALES {
            assert_eq!(
                parity(&source, locale),
                expected(vec![error(locale, span(&source, &marker))])
            );
        }
    }
}

#[test]
fn source_geometry_includes_empty_content_pointer_and_actual_child_ordinal() {
    let source = "<template><textarea>lead{{}}tail</textarea></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let header = lint.header_facts(&element).unwrap();
    let child = element.children().nth(1).unwrap();
    let facts = header.child_facts(child).unwrap();
    let table = facts.markers().unwrap();
    assert_eq!(table.len(), 1);
    let marker = table.get(&1).unwrap();
    assert_eq!(marker.header_key(), 0);
    assert_eq!(marker.span(), span(source, "{{}}"));
    assert_eq!(marker.content(), Span::new(26, 26));
    assert_eq!(facts.original().ordinal(), 1);
}

#[test]
fn first_authored_close_is_used_without_expression_or_string_scanning() {
    for (body, marker) in [
        ("{{ \"}}\" }}", "{{ \"}}"),
        ("{{{x}}}", "{{{x}}"),
        ("{{ a }} }}}", "{{ a }}"),
    ] {
        let source = cstr!("<template><textarea>{body}</textarea></template>");
        for locale in LOCALES {
            assert_eq!(
                parity(&source, locale),
                expected(vec![error(locale, span(&source, marker))])
            );
        }
    }
}

#[test]
fn rcdata_apparent_nested_element_is_text_with_a_real_direct_marker() {
    let source = "<template><textarea><span>{{x}}</span></textarea></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![error(locale, span(source, "{{x}}"))])
        );
    }
}

#[test]
fn rcdata_comment_spelling_does_not_hide_its_direct_marker() {
    let source = "<template><textarea><!-- {{x}} --></textarea></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![error(locale, span(source, "{{x}}"))])
        );
    }
}

#[test]
fn v_model_is_not_a_registered_rule_exemption() {
    let source = "<template><textarea v-model='x'>{{ x }}</textarea></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![error(locale, span(source, "{{ x }}"))])
        );
    }
}

#[test]
fn multiple_markers_and_elements_retain_original_unsorted_source_order() {
    let source = "<template><div><textarea>A{{first}}B{{ second }}</textarea><textarea>{{third}}</textarea></div></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![
                error(locale, span(source, "{{first}}")),
                error(locale, span(source, "{{ second }}")),
                error(locale, span(source, "{{third}}")),
            ])
        );
    }
}

#[test]
fn unicode_and_nonfirst_template_offsets_are_absolute_original_bytes() {
    let source = "<!--前置き-->\n<script>const 雪 = 1</script>\n<template>雪<textarea>é{{ 雪 }}終</textarea></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![error(locale, span(source, "{{ 雪 }}"))])
        );
    }
}
