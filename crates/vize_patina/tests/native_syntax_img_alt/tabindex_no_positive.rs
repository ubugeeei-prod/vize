use super::support::{native, owner, tabindex_parity as parity, tabindex_reference};
use vize_carton::i18n::{Locale, translator};
use vize_l0::{Allocator, Span};
use vize_patina::native::{NativeLintRefusal, NativeSyntaxLint};

#[test]
fn complete_catalog_diagnostics_keep_full_authored_attribute_ranges() {
    for attribute in ["tabindex=1", "tabindex='1'", "tabindex=\"+1\""] {
        let source = vize_l0::cstr!("<template><div {attribute}>body</div></template>");
        for locale in [Locale::En, Locale::Ja, Locale::Zh] {
            let output = parity(&source, locale);
            assert_eq!(output.len(), 1);
            let start = source.find(attribute).unwrap();
            assert_eq!(output[0]["start"], start);
            assert_eq!(output[0]["end"], start + attribute.len());
            assert_eq!(output[0]["rule_name"], "a11y/tabindex-no-positive");
            assert_eq!(output[0]["severity"], "warning");
            assert_eq!(output[0]["labels"], serde_json::json!([]));
            assert_eq!(output[0]["fix"], serde_json::Value::Null);
        }
    }
}

#[test]
fn positive_i32_values_keep_upstream_parse_semantics() {
    for value in ["1", "+1", "01", "00099", "2147483647", "+2147483647"] {
        let source = vize_l0::cstr!("<template><div tabindex='{value}' /></template>");
        assert_eq!(parity(&source, Locale::En).len(), 1, "{value}");
    }
}

#[test]
fn zero_negative_whitespace_invalid_and_overflow_values_are_not_positive() {
    for value in [
        "",
        "0",
        "+0",
        "-0",
        "-1",
        "-2147483648",
        "2147483648",
        "+2147483648",
        "-2147483649",
        " 1",
        "1 ",
        "\t1",
        "1\r\n",
        "1.0",
        "1e1",
        "++1",
        "--1",
        "0x1",
        "１",
        "١",
    ] {
        let source = vize_l0::cstr!("<template><div tabindex='{value}' /></template>");
        assert!(parity(&source, Locale::En).is_empty(), "{value:?}");
    }
    assert!(parity("<template><div tabindex /></template>", Locale::En).is_empty());
}

#[test]
fn native_attribute_entities_feed_the_same_i32_parse() {
    for value in [
        "&#49;",
        "&#x31;",
        "&#43;1",
        "&#x2b;&#49;",
        "&#48;1",
        "2&#49;47483647",
    ] {
        let source = vize_l0::cstr!("<template><div tabindex='{value}' /></template>");
        assert_eq!(parity(&source, Locale::En).len(), 1, "{value}");
    }
    for value in [
        "&#32;1",
        "1&#9;",
        "&nbsp;1",
        "&#45;1",
        "&#0;1",
        "&amp;1",
        "&unknown;1",
        "&fjlig;",
        "2&#49;47483648",
    ] {
        let source = vize_l0::cstr!("<template><div tabindex='{value}' /></template>");
        assert!(parity(&source, Locale::En).is_empty(), "{value}");
    }
}

#[test]
fn binding_expressions_are_opaque_and_never_static_integer_values() {
    for header in [
        ":tabindex='1'",
        ".tabindex='1'",
        "v-bind:tabindex='1'",
        ":tabindex.prop='1'",
        "@tabindex='1'",
        "v-model:tabindex='value'",
        "data-tabindex='1'",
    ] {
        let source = vize_l0::cstr!("<template><div {header} /></template>");
        assert!(parity(&source, Locale::En).is_empty(), "{header}");
    }
}

#[test]
fn ascii_case_components_and_verbatim_controls_match() {
    for source in [
        "<template><div TABINDEX='1' /></template>",
        "<template><MyControl tabIndex='1' /></template>",
        "<template><div v-pre tabindex='1' /></template>",
    ] {
        assert_eq!(parity(source, Locale::En).len(), 1);
    }
    for source in [
        "<template><div v-pre :tabindex='1' /></template>",
        "<template><div v-pre><span :[tabindex]='1' /></div></template>",
        "<template><div foo:tabindex='1' /></template>",
    ] {
        assert!(parity(source, Locale::En).is_empty());
    }
}

#[test]
fn original_attribute_order_and_unicode_absolute_ranges_match() {
    let source = "<!--🦀--><script>const fake='<div tabindex=9 />'</script><template>日本語<div tabindex='1'><i tabIndex='2'></i><span tabindex='3'></span></div></template>";
    let output = parity(source, Locale::En);
    assert_eq!(output.len(), 3);
    for (finding, attribute) in output
        .iter()
        .zip(["tabindex='1'", "tabIndex='2'", "tabindex='3'"])
    {
        let start = source.find(attribute).unwrap();
        assert_eq!(finding["start"], start);
        assert_eq!(finding["end"], start + attribute.len());
    }
}

#[test]
fn original_duplicate_case_fixture_refuses_instead_of_losing_parser_advisory() {
    let source = "<!--🦀--><script>const fake='<div tabindex=9 />'</script><template>日本語<div tabindex='1' tabIndex='2'><span tabindex='3'></span></div></template>";
    let arena = Allocator::default();
    let original = owner(&arena, source);
    let lint = NativeSyntaxLint::new(&original).unwrap();
    let element = original
        .children()
        .find_map(|child| child.into_element())
        .unwrap();
    let start = u32::try_from(source.find("tabIndex").unwrap()).unwrap();
    assert_eq!(
        lint.tabindex_no_positive(&element, &translator().for_locale(Locale::En))
            .err(),
        Some(NativeLintRefusal::DuplicateAttribute {
            span: Span::new(start, start + 8)
        })
    );
    assert!(original.component().carrier().errors.is_empty());
    let reference = tabindex_reference(source, Locale::En);
    assert_eq!(reference, tabindex_reference(source, Locale::En));
    assert_eq!(reference.len(), 4);
    assert_eq!(
        reference[0],
        serde_json::json!({
            "rule_name": "parser/template", "severity": "warning",
            "message": "Duplicate attribute `tabIndex`. Keeping the repeated attribute so parsing can continue.",
            "start": start, "end": start + 8, "help": null, "labels": [], "fix": null,
        })
    );
    for (finding, attribute) in
        reference
            .iter()
            .skip(1)
            .zip(["tabindex='1'", "tabIndex='2'", "tabindex='3'"])
    {
        let start = source.find(attribute).unwrap();
        assert_eq!(finding["rule_name"], "a11y/tabindex-no-positive");
        assert_eq!(finding["start"], start);
        assert_eq!(finding["end"], start + attribute.len());
    }
}

#[test]
fn later_unsupported_headers_discard_an_earlier_positive_result() {
    for (head, unresolved) in [(":[key]", true), ("v-bind", true), ("v-unknown", false)] {
        let source = vize_l0::cstr!("<template><div tabindex='1' {head}='value' /></template>");
        let arena = Allocator::default();
        let original = owner(&arena, &source);
        let lint = NativeSyntaxLint::new(&original).unwrap();
        let element = original.children().next().unwrap().into_element().unwrap();
        let start = u32::try_from(source.find(head).unwrap()).unwrap();
        let span = Span::new(start, start + u32::try_from(head.len()).unwrap());
        let expected = if unresolved {
            NativeLintRefusal::UnresolvedBinding { span }
        } else {
            NativeLintRefusal::UnsupportedDirective { span }
        };
        assert_eq!(
            lint.tabindex_no_positive(&element, &translator().for_locale(Locale::En))
                .err(),
            Some(expected)
        );
        assert!(original.component().carrier().errors.is_empty());
    }
}

#[test]
fn foreign_owners_and_recovery_never_publish_findings() {
    let arena = Allocator::default();
    let source = "<template><div tabindex='1' /></template>";
    let first = owner(&arena, source);
    let second = owner(&arena, source);
    let lint = NativeSyntaxLint::new(&first).unwrap();
    let element = second.children().next().unwrap().into_element().unwrap();
    assert_eq!(
        lint.tabindex_no_positive(&element, &translator().for_locale(Locale::En))
            .err(),
        Some(NativeLintRefusal::ForeignElement)
    );
    assert!(core::ptr::eq(lint.owner(), &first));
    let recovered = owner(
        &arena,
        "<template><div tabindex='unterminated /></template>",
    );
    let errors = &recovered.component().carrier().errors;
    assert!(!errors.is_empty());
    let count = errors.len();
    let offset = recovered.component().block().start() + errors[0].offset;
    assert_eq!(
        NativeSyntaxLint::new(&recovered).err(),
        Some(NativeLintRefusal::Recovered { offset })
    );
    assert_eq!(recovered.component().carrier().errors.len(), count);
}

#[test]
fn owned_ordered_findings_outlive_the_owner_and_empty_template_has_none() {
    let (findings, before) = {
        let arena = Allocator::default();
        let original = owner(&arena, "<template><div tabindex='1' /></template>");
        let lint = NativeSyntaxLint::new(&original).unwrap();
        let element = original.children().next().unwrap().into_element().unwrap();
        let findings = lint
            .tabindex_no_positive(&element, &translator().for_locale(Locale::Zh))
            .unwrap();
        let before = findings.iter().map(native).collect::<Vec<_>>();
        (findings, before)
    };
    assert_eq!(findings.iter().map(native).collect::<Vec<_>>(), before);
    assert_eq!(before.len(), 1);
    assert!(parity("<template></template>", Locale::En).is_empty());
}
