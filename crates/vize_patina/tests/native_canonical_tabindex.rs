mod refusals;
mod support;
mod timing;

use support::{Dialect, parity};
use vize_patina::Locale;

#[test]
fn actual_vue_and_jsx_owners_match_complete_registered_rule_packets() {
    for dialect in [Dialect::Vue, Dialect::Jsx, Dialect::Tsx] {
        for newline in ["\n", "\r\n"] {
            for quote in ['\'', '"'] {
                for locale in [Locale::En, Locale::Ja, Locale::Zh] {
                    let body = vize_l0::cstr!(
                        "<section title={quote}作者😀{quote}><p tabindex={quote}1{quote}/><p tabIndex={quote}+002{quote}/><p tabindex={quote}-1{quote}/></section>"
                    );
                    let output = parity(dialect, &body, newline, locale);
                    assert_eq!(output.len(), 2);
                }
            }
        }
    }
}

#[test]
fn every_native_input_uses_the_same_value_sensitive_i32_policy() {
    for dialect in [Dialect::Vue, Dialect::Jsx, Dialect::Tsx] {
        for value in ["1", "+1", "01", "00099", "2147483647", "+2147483647"] {
            let body = vize_l0::cstr!("<div tabindex='{value}'/>");
            assert_eq!(parity(dialect, &body, "\n", Locale::En).len(), 1);
        }
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
            "1.0",
            "1e1",
            "++1",
            "--1",
            "0x1",
            "１",
            "١",
        ] {
            let body = vize_l0::cstr!("<div tabindex='{value}'/>");
            assert!(
                parity(dialect, &body, "\n", Locale::En).is_empty(),
                "{value:?}"
            );
        }
        assert!(parity(dialect, "<div tabindex/>", "\n", Locale::En).is_empty());
        assert!(parity(dialect, "<div data-tabindex='1'/>", "\n", Locale::En).is_empty());
    }
}

#[test]
fn only_the_actual_vue_producer_admits_its_decoded_entity_values() {
    for (value, count) in [
        ("&#49;", 1),
        ("&#x31;", 1),
        ("&#43;1", 1),
        ("&#x2b;&#49;", 1),
        ("&#32;1", 0),
        ("1&#9;", 0),
        ("&nbsp;1", 0),
        ("&#45;1", 0),
        ("&unknown;1", 0),
        ("&fjlig;", 0),
        ("2&#49;47483648", 0),
    ] {
        let body = vize_l0::cstr!("<div tabindex='{value}'/>");
        assert_eq!(parity(Dialect::Vue, &body, "\r\n", Locale::Ja).len(), count);
    }
}
