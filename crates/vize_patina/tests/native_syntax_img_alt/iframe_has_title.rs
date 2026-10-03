use super::support::{iframe_parity as parity, native, owner};
use vize_carton::i18n::{Locale, translator};
use vize_l0::{Allocator, Span};
use vize_patina::native::{NativeLintRefusal, NativeSyntaxLint};

#[test]
fn complete_warning_help_and_closed_element_range_match_all_locales() {
    let element = "<iframe src='https://example.com'>fallback</iframe>";
    let source = vize_l0::cstr!("<template>{element}</template>");
    for locale in [Locale::En, Locale::Ja, Locale::Zh] {
        let output = parity(&source, locale);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0]["rule_name"], "a11y/iframe-has-title");
        assert_eq!(output[0]["start"], 10);
        assert_eq!(output[0]["end"], 10 + element.len());
        assert_eq!(output[0]["severity"], "warning");
        assert_eq!(output[0]["labels"], serde_json::json!([]));
        assert_eq!(output[0]["fix"], serde_json::Value::Null);
    }
}

#[test]
fn static_title_requires_a_non_whitespace_decoded_scalar() {
    for title in [
        "",
        " ",
        "\t\r\n",
        "&#32;",
        "&#x20;",
        "&#9;",
        "&Tab;&NewLine;",
        "&nbsp;",
        "&#160;",
        "\u{3000}",
    ] {
        let source = vize_l0::cstr!("<template><iframe title='{title}' /></template>");
        assert_eq!(parity(&source, Locale::En).len(), 1, "{title:?}");
    }
    for title in [
        "Example",
        " 日本語🦀 ",
        "&amp;",
        "&#x1f980;",
        "&fjlig;",
        "&unknown;",
        "&notit;",
        "&amp=",
        "&#0;",
    ] {
        let source = vize_l0::cstr!("<template><iframe title='{title}' /></template>");
        assert!(parity(&source, Locale::En).is_empty(), "{title:?}");
    }
    assert_eq!(
        parity("<template><iframe title /></template>", Locale::En).len(),
        1
    );
}

#[test]
fn typed_static_title_bindings_are_opaque_and_keep_authored_modifiers() {
    for header in [
        ":title='frame?.description'",
        ".title='description'",
        "v-bind:title='description'",
        ":title.camel='description'",
        ".title.prop='description'",
        "v-bind:title.prop='description'",
        ":title",
        ":title='null'",
        ":title=\"''\"",
    ] {
        let source = vize_l0::cstr!("<template><iframe {header} /></template>");
        assert!(parity(&source, Locale::En).is_empty(), "{header}");
    }
}

#[test]
fn exact_tag_argument_and_non_binding_controls_match() {
    for header in [
        "TITLE='description'",
        ":TITLE='description'",
        "@title='handler'",
        "v-model:title='description'",
        "data-title='description'",
    ] {
        let source = vize_l0::cstr!("<template><iframe {header} /></template>");
        assert_eq!(parity(&source, Locale::En).len(), 1, "{header}");
    }
    for tag in ["IFRAME", "Iframe", "my-iframe", "svg:iframe", "div"] {
        let source = vize_l0::cstr!("<template><{tag} /></template>");
        assert!(parity(&source, Locale::En).is_empty(), "{tag}");
    }
}

#[test]
fn original_verbatim_flags_keep_directive_looking_titles_literal() {
    for source in [
        "<template><iframe v-pre :title='description' /></template>",
        "<template><div v-pre><iframe :[title]='description' /></div></template>",
    ] {
        assert_eq!(parity(source, Locale::En).len(), 1);
    }
    assert!(
        parity(
            "<template><iframe v-pre title='description' /></template>",
            Locale::En
        )
        .is_empty()
    );
}

#[test]
fn nested_selected_elements_keep_full_absolute_unicode_spans() {
    let source = "<!--🦀--><script lang=ts>const fake='<iframe />'</script><template>日本語<div><iframe /><iframe title='x' /><section><iframe>🦀</iframe></section></div></template><script setup lang=ts>const other='<iframe />'</script>";
    let output = parity(source, Locale::En);
    assert_eq!(output.len(), 2);
    let first = source.find("<iframe /><iframe").unwrap();
    let second = source.find("<iframe>🦀</iframe>").unwrap();
    for ((finding, element), start) in output
        .iter()
        .zip(["<iframe />", "<iframe>🦀</iframe>"])
        .zip([first, second])
    {
        assert_eq!(finding["start"], start);
        assert_eq!(finding["end"], start + element.len());
    }
}

#[test]
fn later_unresolved_headers_refuse_before_a_known_title_result() {
    for (head, unresolved) in [
        (":[key]", true),
        ("v-bind", true),
        ("@[key]", true),
        ("v-unknown", false),
    ] {
        let source = vize_l0::cstr!("<template><iframe title='known' {head}='value' /></template>");
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
            lint.iframe_has_title(&element, &translator().for_locale(Locale::En))
                .err(),
            Some(expected)
        );
        assert!(original.component().carrier().errors.is_empty());
    }
}

#[test]
fn equal_foreign_owner_is_refused_and_original_recovery_is_retained() {
    let arena = Allocator::default();
    let source = "<template><iframe /></template>";
    let first = owner(&arena, source);
    let second = owner(&arena, source);
    let lint = NativeSyntaxLint::new(&first).unwrap();
    let element = second.children().next().unwrap().into_element().unwrap();
    assert_eq!(
        lint.iframe_has_title(&element, &translator().for_locale(Locale::En))
            .err(),
        Some(NativeLintRefusal::ForeignElement)
    );
    assert!(core::ptr::eq(lint.owner(), &first));
    assert!(core::ptr::eq(element.component(), second.component()));
    let recovered = owner(
        &arena,
        "<template><iframe title='unterminated /></template>",
    );
    let errors = &recovered.component().carrier().errors;
    assert!(!errors.is_empty());
    let offset = recovered.component().block().start() + errors[0].offset;
    let error_count = errors.len();
    assert_eq!(
        NativeSyntaxLint::new(&recovered).err(),
        Some(NativeLintRefusal::Recovered { offset })
    );
    assert_eq!(recovered.component().carrier().errors.len(), error_count);
}

#[test]
fn completed_iframe_diagnostic_outlives_its_selected_owner_and_arena() {
    let (finding, before) = {
        let arena = Allocator::default();
        let original = owner(&arena, "<template><iframe /></template>");
        let lint = NativeSyntaxLint::new(&original).unwrap();
        let element = original.children().next().unwrap().into_element().unwrap();
        let finding = lint
            .iframe_has_title(&element, &translator().for_locale(Locale::Ja))
            .unwrap()
            .unwrap();
        let before = native(&finding);
        (finding, before)
    };
    assert_eq!(native(&finding), before);
    assert_eq!(native(&finding)["start"], 10);
    assert_eq!(native(&finding)["end"], 20);
    assert_eq!(finding.into_diagnostic().parts.len(), 1);
}

#[test]
fn empty_selected_template_emits_nothing() {
    assert!(parity("<template></template>", Locale::En).is_empty());
}
