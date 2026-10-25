//! Full diagnostics from actual native owners; no configured-route migration.
mod refusals;
mod support;

use serde_json::json;
use support::{Dialect, parity};
use vize_patina::Locale;

#[test]
fn native_vue_jsx_and_tsx_match_full_localized_registered_packets() {
    for dialect in [Dialect::Vue, Dialect::Jsx, Dialect::Tsx] {
        for newline in ["\n", "\r\n"] {
            for quote in ['\'', '"'] {
                for locale in [Locale::En, Locale::Ja, Locale::Zh] {
                    let body = vize_l0::cstr!(
                        "<section title={quote}作者😀{quote}><img src={quote}first{quote}/><img alt={quote}{quote}/><div><img src={quote}last{quote}/></div><img alt/></section>"
                    );
                    let output = parity(dialect, &body, newline, locale);
                    assert_eq!(output.len(), 2);
                }
            }
        }
    }
}

#[test]
fn complete_literal_catalog_packet_is_independent_of_the_registered_oracle() {
    let output = parity(Dialect::Vue, "<img/>", "\r\n", Locale::Ja);
    assert_eq!(
        output,
        vec![json!({
            "rule_name":"a11y/img-alt","severity":"warning",
            "message":"<img>要素にはアクセシビリティのためにalt属性が必要です",
            "start":29,"end":35,
            "help":"情報を伝える画像にはalt=\"説明\"を、装飾的な画像にはalt=\"\"を追加してください",
            "labels":[],"fix":null,
        })]
    );
}

#[test]
fn presence_does_not_interpret_static_values_or_unrelated_names() {
    for dialect in [Dialect::Vue, Dialect::Jsx, Dialect::Tsx] {
        for header in ["alt", "alt=''", "ALT=\"\"", "alt='作者😀'", "Alt='0'"] {
            let body = vize_l0::cstr!("<img {header}/>");
            assert!(parity(dialect, &body, "\n", Locale::En).is_empty());
        }
        for body in ["<img/>", "<img data-alt='x'/>", "<img title='alt'/>"] {
            assert_eq!(parity(dialect, body, "\r\n", Locale::Zh).len(), 1);
        }
        assert!(parity(dialect, "<div/>", "\n", Locale::En).is_empty());
    }
}

#[test]
fn original_static_bound_names_count_without_interpreting_values() {
    for header in [":alt='1'", "v-bind:alt='null'", ":ALT='\"text\"'"] {
        let body = vize_l0::cstr!("<section><img {header}/><img alt=''/></section>");
        assert!(parity(Dialect::Vue, &body, "\r\n", Locale::En).is_empty());
    }
}

#[test]
fn entity_values_remain_the_original_producers_responsibility() {
    for header in [
        "alt='&quot;日本語&amp;🦀'",
        "alt='&unknown;'",
        "title='&amp;'",
    ] {
        let body = vize_l0::cstr!("<img {header}/>");
        let expected = usize::from(header.starts_with("title"));
        assert_eq!(
            parity(Dialect::Vue, &body, "\n", Locale::En).len(),
            expected
        );
    }
}

#[test]
fn pending_header_is_flushed_at_siblings_and_at_end_of_file() {
    for dialect in [Dialect::Vue, Dialect::Jsx, Dialect::Tsx] {
        assert_eq!(parity(dialect, "<img/>", "\n", Locale::En).len(), 1);
        assert!(parity(dialect, "<img alt/>", "\n", Locale::En).is_empty());
        assert_eq!(
            parity(
                dialect,
                "<div><img/><img alt/><img/></div>",
                "\n",
                Locale::En
            )
            .len(),
            2
        );
    }
}

#[test]
fn pinned_production_inputs_compare_full_packets_on_their_genuine_owners() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/linter/native-canonical-image-alt-10563");
    for (file, dialect) in [
        ("images.lf.vue.txt", Dialect::Vue),
        ("images.crlf.vue.txt", Dialect::Vue),
        ("images.jsx.txt", Dialect::Jsx),
        ("images.tsx.txt", Dialect::Tsx),
    ] {
        let source = std::fs::read_to_string(root.join(file)).unwrap();
        for locale in [Locale::En, Locale::Ja, Locale::Zh] {
            assert_eq!(
                support::source_parity(dialect, &source, locale).len(),
                2,
                "{file}"
            );
        }
    }
}
