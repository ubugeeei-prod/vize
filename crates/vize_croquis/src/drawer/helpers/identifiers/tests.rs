use super::extract_identifiers_oxc;
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
