use super::support::{LOCALES, NeverLookup, expected, parity, selected};
use vize_l0::{Allocator, cstr};
use vize_patina::native::NativeSyntaxLint;

#[test]
fn exact_authored_lowercase_unqualified_tag_contract_has_no_name_guess() {
    for tag in [
        "Textarea",
        "TEXTAREA",
        "svg:textarea",
        "foo.textarea",
        "div",
        "title",
        "my-textarea",
    ] {
        let source = cstr!("<template><{tag}>{{{{ x }}}}</{tag}></template>");
        for locale in LOCALES {
            assert_eq!(parity(&source, locale), expected(vec![]));
        }
    }
}

#[test]
fn genuine_nonmatching_markers_do_not_touch_a_catalog() {
    let source = "<template><div>{{ x }}</div></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let header = lint.header_facts(&element).unwrap();
    let facts = header
        .child_facts(element.children().next().unwrap())
        .unwrap();
    assert_eq!(facts.markers().unwrap().len(), 1);
    assert_eq!(facts.textarea_mustache().unwrap().len(), 0);
    assert_eq!(facts.no_textarea_mustache(&NeverLookup).unwrap().len(), 0);
}

#[test]
fn opaque_literal_and_entity_spelled_braces_are_not_regex_interpolations() {
    for body in [
        "plain",
        "{x}",
        "}",
        "&#123;&#123;x&#125;&#125;",
        "&lbrace;&lbrace;x&rbrace;&rbrace;",
    ] {
        let source = cstr!("<template><textarea>{body}</textarea></template>");
        for locale in LOCALES {
            assert_eq!(parity(&source, locale), expected(vec![]));
        }
    }
}

#[test]
fn attribute_mustaches_are_never_supplied_child_markers() {
    let source = "<template><textarea title='{{ x }}'></textarea></template>";
    for locale in LOCALES {
        assert_eq!(parity(source, locale), expected(vec![]));
    }
}

#[test]
fn empty_and_self_closing_textareas_retain_original_complete_clean_output() {
    for source in [
        "<template><textarea></textarea></template>",
        "<template><textarea /></template>",
    ] {
        for locale in LOCALES {
            assert_eq!(parity(source, locale), expected(vec![]));
        }
    }
}

#[test]
fn raw_script_and_style_text_remain_opaque_supplied_children() {
    let source = "<template><script>const html = '<textarea>{{x}}</textarea>'</script><style>.x::before{content:'<textarea>{{x}}</textarea>'}</style></template>";
    for locale in LOCALES {
        assert_eq!(parity(source, locale), expected(vec![]));
    }
}

#[test]
fn actual_nested_marker_belongs_to_its_own_parent_not_the_supplied_div() {
    let source = "<template><div><span>{{x}}</span></div></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let header = lint.header_facts(&element).unwrap();
    let facts = header
        .child_facts(element.children().next().unwrap())
        .unwrap();
    assert_eq!(facts.markers().unwrap().len(), 0);
    assert_eq!(facts.no_textarea_mustache(&NeverLookup).unwrap().len(), 0);
    for locale in LOCALES {
        assert_eq!(parity(source, locale), expected(vec![]));
    }
}

#[test]
fn literal_pre_on_the_original_header_freezes_markers_without_a_body_certificate() {
    for head in ["v-pre", "v-pre v-pre.foo", "v-pre.foo v-pre"] {
        let source = cstr!("<template><textarea {head}>{{{{x}}}}</textarea></template>");
        for locale in LOCALES {
            assert_eq!(parity(&source, locale), expected(vec![]));
        }
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let header = lint.header_facts(&element).unwrap();
        let facts = header
            .child_facts(element.children().next().unwrap())
            .unwrap();
        assert_eq!(facts.markers().unwrap().len(), 0);
        assert_eq!(facts.no_textarea_mustache(&NeverLookup).unwrap().len(), 0);
    }
}

#[test]
fn inherited_exact_pre_context_preserves_clean_original_output() {
    let source = "<template><div v-pre><textarea>{{x}}</textarea></div></template>";
    for locale in LOCALES {
        assert_eq!(parity(source, locale), expected(vec![]));
    }
}
