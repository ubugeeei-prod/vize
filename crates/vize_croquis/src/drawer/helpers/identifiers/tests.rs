use super::{
    IdentifierRef, extract_identifier_refs_fast, extract_identifier_refs_oxc,
    extract_identifiers_oxc, strip_js_comments,
};
use vize_carton::CompactString;

#[test]
fn test_extract_identifier_refs_fast_treats_spread_as_read() {
    let expr = "[...menu, item.id]";
    assert_eq!(
        extract_identifier_refs_fast(expr),
        vec![
            IdentifierRef {
                name: "menu".into(),
                offset: expr.find("menu").unwrap() as u32,
            },
            IdentifierRef {
                name: "item".into(),
                offset: expr.find("item").unwrap() as u32,
            },
        ]
    );
}

#[test]
fn test_extract_identifiers_oxc() {
    fn to_strings(ids: Vec<CompactString>) -> Vec<CompactString> {
        ids
    }

    let ids = to_strings(extract_identifiers_oxc("count + 1"));
    assert_eq!(ids, vec!["count"]);

    let ids = to_strings(extract_identifiers_oxc("user.name + item.value"));
    assert_eq!(ids, vec!["user", "item"]);

    let ids = to_strings(extract_identifiers_oxc("{ active: isActive }"));
    assert_eq!(ids, vec!["isActive"]);

    let ids = to_strings(extract_identifiers_oxc("{ foo }"));
    assert_eq!(ids, vec!["foo"]);

    let ids = to_strings(extract_identifiers_oxc("[...menu]"));
    assert_eq!(ids, vec!["menu"]);

    let ids = to_strings(extract_identifiers_oxc("cond ? a : b"));
    assert_eq!(ids, vec!["cond", "a", "b"]);
}

#[test]
fn test_extract_identifiers_ignores_comment_words() {
    fn to_strings(ids: Vec<CompactString>) -> Vec<CompactString> {
        ids
    }

    let ids = to_strings(extract_identifiers_oxc(
        "/** comment words should disappear */ disabled ? true : undefined",
    ));
    assert_eq!(ids, vec!["disabled", "true", "undefined"]);
}

#[test]
fn test_extract_identifiers_supports_unicode_identifier_references() {
    let expression = "挨拶 + profile.表示名 + { label: 地域名 }";

    assert_eq!(
        extract_identifiers_oxc(expression),
        vec!["挨拶", "profile", "地域名"]
    );
    assert_eq!(
        extract_identifier_refs_oxc(expression),
        vec![
            IdentifierRef {
                name: "挨拶".into(),
                offset: expression.find("挨拶").unwrap() as u32,
            },
            IdentifierRef {
                name: "profile".into(),
                offset: expression.find("profile").unwrap() as u32,
            },
            IdentifierRef {
                name: "地域名".into(),
                offset: expression.find("地域名").unwrap() as u32,
            },
        ]
    );
}

#[test]
fn test_strip_js_comments_preserves_non_ascii_literals_after_comments() {
    let expr = "/* hidden */ menu.header === 'アカウント' ? '選択中' : '通常'";
    let stripped = strip_js_comments(expr);

    assert_eq!(
        stripped.as_ref(),
        "  menu.header === 'アカウント' ? '選択中' : '通常'"
    );
}

#[test]
fn test_strip_js_comments_preserves_regex_with_escaped_slashes() {
    let expression = r#"url.replace(/^http:\/\//, '') // remove this comment
+ suffix"#;
    let stripped = strip_js_comments(expression);

    assert_eq!(
        stripped.as_ref(),
        concat!(r#"url.replace(/^http:\/\//, '') "#, "\n+ suffix")
    );

    let block_like = r#"value.match(/\/\*literal/) /* remove this comment */"#;
    assert_eq!(
        strip_js_comments(block_like).as_ref(),
        r#"value.match(/\/\*literal/)  "#
    );
}

#[test]
fn test_extract_identifiers_ignores_regex_literals() {
    fn to_strings(ids: Vec<CompactString>) -> Vec<CompactString> {
        ids
    }

    let ids = to_strings(extract_identifiers_oxc("message.match(/foo/)"));
    assert_eq!(ids, vec!["message"]);

    let ids = to_strings(extract_identifiers_oxc("/foo/.test(message)"));
    assert_eq!(ids, vec!["message"]);

    let ids = to_strings(extract_identifiers_oxc("message.replace(/foo/g, bar)"));
    assert_eq!(ids, vec!["message", "bar"]);

    let ids = to_strings(extract_identifiers_oxc("count / divisor"));
    assert_eq!(ids, vec!["count", "divisor"]);
}

#[test]
fn test_extract_identifiers_ignores_numeric_literal_tails() {
    fn to_pairs(ids: Vec<IdentifierRef>) -> Vec<(CompactString, u32)> {
        ids.into_iter()
            .map(|identifier| (identifier.name, identifier.offset))
            .collect()
    }

    let expr = "1e22 + 0xFF + 0b1010 + 123n + .5e-2 + count";
    assert_eq!(extract_identifiers_oxc(expr), vec!["count"]);
    assert_eq!(
        to_pairs(extract_identifier_refs_oxc(expr)),
        vec![("count".into(), expr.find("count").unwrap() as u32)]
    );
}

#[test]
fn test_extract_identifier_refs_preserve_root_offsets() {
    fn to_pairs(ids: Vec<IdentifierRef>) -> Vec<(CompactString, u32)> {
        ids.into_iter()
            .map(|identifier| (identifier.name, identifier.offset))
            .collect()
    }

    let ids = to_pairs(extract_identifier_refs_oxc("user.name + name"));
    assert_eq!(ids, vec![("user".into(), 0), ("name".into(), 12)]);

    let ids = to_pairs(extract_identifier_refs_oxc("{ active: isActive }"));
    assert_eq!(ids, vec![("isActive".into(), 10)]);

    let division = "count / divisor";
    let ids = to_pairs(extract_identifier_refs_oxc(division));
    assert_eq!(
        ids,
        vec![
            ("count".into(), division.find("count").unwrap() as u32),
            ("divisor".into(), division.find("divisor").unwrap() as u32),
        ]
    );

    let with_comment = "/** hidden */ disabled";
    let ids = to_pairs(extract_identifier_refs_oxc(with_comment));
    assert_eq!(
        ids,
        vec![(
            "disabled".into(),
            with_comment.find("disabled").unwrap() as u32
        )]
    );
}
