use super::{IdentifierRef, extract_identifier_refs_oxc, extract_identifiers_oxc};
use vize_carton::CompactString;

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
