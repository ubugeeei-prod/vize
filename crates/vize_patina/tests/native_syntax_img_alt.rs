//! A dev-only legacy oracle checks complete diagnostics; production uses L1 only.
#[cfg(test)]
mod native_syntax_img_alt {
    pub mod support;
}

use native_syntax_img_alt::support::{native, owner, parity};
use vize_carton::i18n::{Locale, translator};
use vize_l0::{Allocator, Span};
use vize_patina::native::{NativeLintRefusal, NativeSyntaxLint};

#[test]
fn full_catalog_diagnostic_and_absolute_authored_span_match() {
    let source = "<template><img src=\"/photo.jpg\" /></template>";
    for locale in [Locale::En, Locale::Ja, Locale::Zh] {
        let output = parity(source, locale);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0]["rule_name"], "a11y/img-alt");
        assert_eq!(output[0]["start"], 10);
        assert_eq!(output[0]["end"], 34);
        assert_eq!(output[0]["severity"], "warning");
        assert_eq!(output[0]["labels"], serde_json::json!([]));
        assert_eq!(output[0]["fix"], serde_json::Value::Null);
    }
    assert_eq!(
        parity(source, Locale::En)[0]["message"],
        "<img> elements must have an alt attribute for accessibility"
    );
}

#[test]
fn static_and_typed_static_alt_bindings_match_without_value_parsing() {
    for header in [
        "alt",
        "alt=''",
        "ALT=\"\"",
        ":alt='photo?.description'",
        ".alt='text'",
        "v-bind:alt='text'",
        ":ALT.camel='text'",
        ".ALT.prop='text'",
        "v-bind:alt.prop='text'",
        "alt='&quot;日本語&amp;🦀'",
        ":alt",
    ] {
        let source = vize_l0::cstr!("<template><img {header} /></template>");
        assert!(parity(&source, Locale::En).is_empty(), "{header}");
    }
}

#[test]
fn absent_alt_events_and_case_insensitive_whole_element_spans_match() {
    for source in [
        "<template><IMG src='photo' /></template>",
        "<template><IMG src='photo'></IMG></template>",
        "<template><img @alt='onAlt' /></template>",
        "<template><img v-if='visible' /></template>",
        "<template><img data-alt='x' /></template>",
    ] {
        assert_eq!(parity(source, Locale::En).len(), 1);
    }
}

#[test]
fn selected_template_and_nested_original_children_keep_unicode_offsets() {
    let source = "<!--🦀--><script lang=ts>const fake='<img />'</script><template>日本語<div><img /><img alt='x' /><section><img /></section></div></template><script setup lang=ts>const other='<img />'</script>";
    let output = parity(source, Locale::En);
    assert_eq!(output.len(), 2);
    let first = u32::try_from(source.find("<img /><img").unwrap()).unwrap();
    let second = u32::try_from(source.find("<section><img").unwrap() + "<section>".len()).unwrap();
    assert_eq!(output[0]["start"], first);
    assert_eq!(output[0]["end"], first + 7);
    assert_eq!(output[1]["start"], second);
    assert_eq!(output[1]["end"], second + 7);
}

#[test]
fn original_verbatim_flag_keeps_directive_like_names_literal() {
    for source in [
        "<template><img v-pre :alt='x' /></template>",
        "<template><div v-pre><img :[alt]='x' /></div></template>",
    ] {
        assert_eq!(parity(source, Locale::En).len(), 1);
    }
    assert!(parity("<template><img v-pre alt='' /></template>", Locale::En).is_empty());
}

#[test]
fn later_unresolved_headers_refuse_even_after_known_alt() {
    for (header, expected) in [
        (
            "alt='' :[key]='x'",
            NativeLintRefusal::UnresolvedBinding {
                span: Span::new(22, 28),
            },
        ),
        (
            "alt='' v-bind='attrs'",
            NativeLintRefusal::UnresolvedBinding {
                span: Span::new(22, 28),
            },
        ),
        (
            "alt='' @[key]='x'",
            NativeLintRefusal::UnresolvedBinding {
                span: Span::new(22, 28),
            },
        ),
        (
            "alt='' v-unknown='x'",
            NativeLintRefusal::UnsupportedDirective {
                span: Span::new(22, 31),
            },
        ),
        (
            ":",
            NativeLintRefusal::UnresolvedBinding {
                span: Span::new(15, 16),
            },
        ),
    ] {
        let source = vize_l0::cstr!("<template><img {header} /></template>");
        let arena = Allocator::default();
        let original = owner(&arena, &source);
        let lint = NativeSyntaxLint::new(&original).unwrap();
        let element = original.children().next().unwrap().into_element().unwrap();
        let refusal = lint.img_alt(&element, &translator().for_locale(Locale::En));
        assert_eq!(refusal.err(), Some(expected), "{header}");
        assert!(original.component().carrier().errors.is_empty());
        assert_eq!(
            element.surface().open.attrs.len(),
            element.attributes().len()
        );
    }
}

#[test]
fn equal_bytes_from_a_different_original_owner_cannot_admit_an_element() {
    let arena = Allocator::default();
    let source = "<template><img /></template>";
    let first = owner(&arena, source);
    let second = owner(&arena, source);
    let lint = NativeSyntaxLint::new(&first).unwrap();
    let element = second.children().next().unwrap().into_element().unwrap();
    assert_eq!(
        lint.img_alt(&element, &translator().for_locale(Locale::En))
            .err(),
        Some(NativeLintRefusal::ForeignElement)
    );
    assert!(core::ptr::eq(lint.owner(), &first));
    assert!(core::ptr::eq(element.component(), second.component()));
    assert_eq!(
        first.component().carrier().tree.source,
        second.component().carrier().tree.source
    );
}

#[test]
fn recovery_is_read_from_original_owner_and_never_cleared() {
    let arena = Allocator::default();
    let original = owner(&arena, "<template><img alt='unterminated /></template>");
    let errors = &original.component().carrier().errors;
    assert!(!errors.is_empty());
    let offset = original.component().block().start() + errors[0].offset;
    let error_count = errors.len();
    assert_eq!(
        NativeSyntaxLint::new(&original).err(),
        Some(NativeLintRefusal::Recovered { offset })
    );
    assert_eq!(original.component().carrier().errors.len(), error_count);
}

#[test]
fn finished_diagnostic_outlives_selected_owner_and_arena() {
    let (finding, before) = {
        let arena = Allocator::default();
        let original = owner(&arena, "<template><img /></template>");
        let lint = NativeSyntaxLint::new(&original).unwrap();
        let element = original.children().next().unwrap().into_element().unwrap();
        let finding = lint
            .img_alt(&element, &translator().for_locale(Locale::En))
            .unwrap()
            .unwrap();
        let before = native(&finding);
        (finding, before)
    };
    assert_eq!(native(&finding), before);
    assert_eq!(native(&finding)["start"], 10);
    assert_eq!(native(&finding)["end"], 17);
    assert_eq!(
        finding.into_diagnostic().message,
        "<img> elements must have an alt attribute for accessibility"
    );
}

#[test]
fn selected_empty_template_emits_nothing() {
    assert!(parity("<template></template>", Locale::En).is_empty());
}
