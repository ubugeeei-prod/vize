use super::{format_json_source, format_jsonc_source};
use crate::options::FormatOptions;

fn opts() -> FormatOptions {
    FormatOptions::default()
}

// -- strict JSON (unchanged behaviour) ---------------------------------

#[test]
fn pretty_prints_minified_object() {
    let source = r#"{"name":"vize","version":"0.259.0","keywords":["vue","toolchain"]}"#;
    let result = format_json_source(source, &opts()).unwrap();
    assert_eq!(
        result.as_str(),
        "{\n  \"name\": \"vize\",\n  \"version\": \"0.259.0\",\n  \"keywords\": [\n    \"vue\",\n    \"toolchain\"\n  ]\n}\n",
    );
}

#[test]
fn preserves_key_order_from_source() {
    let source = r#"{"z":1,"a":2,"m":3}"#;
    let result = format_json_source(source, &opts()).unwrap();
    assert_eq!(
        result.as_str(),
        "{\n  \"z\": 1,\n  \"a\": 2,\n  \"m\": 3\n}\n"
    );
}

#[test]
fn already_formatted_is_idempotent() {
    let source = "{\n  \"a\": 1,\n  \"b\": [\n    true,\n    null\n  ]\n}\n";
    let first = format_json_source(source, &opts()).unwrap();
    let second = format_json_source(first.as_str(), &opts()).unwrap();
    assert_eq!(first.as_str(), second.as_str());
}

#[test]
fn empty_collections_stay_compact() {
    let result = format_json_source(r#"{"a":[],"b":{}}"#, &opts()).unwrap();
    assert_eq!(result.as_str(), "{\n  \"a\": [],\n  \"b\": {}\n}\n");
}

#[test]
fn empty_input_yields_empty_output() {
    assert!(format_json_source("", &opts()).unwrap().is_empty());
    assert!(format_json_source("   \n\t  ", &opts()).unwrap().is_empty());
}

#[test]
fn escapes_required_string_characters() {
    let source = r#"{"k":"line\nbreak\t\"quoted\""}"#;
    let result = format_json_source(source, &opts()).unwrap();
    assert!(result.contains(r#""line\nbreak\t\"quoted\"""#));
}

#[test]
fn invalid_json_returns_error() {
    assert!(format_json_source("{\"a\":}", &opts()).is_err());
}

#[test]
fn honors_custom_indent_width() {
    let mut options = opts();
    options.tab_width = 4;
    let result = format_json_source(r#"{"a":1}"#, &options).unwrap();
    assert_eq!(result.as_str(), "{\n    \"a\": 1\n}\n");
}

#[test]
fn strict_json_rejects_comments_and_trailing_commas() {
    assert!(format_json_source("{\n  // hi\n  \"a\": 1\n}", &opts()).is_err());
    assert!(format_json_source(r#"{"a":1,}"#, &opts()).is_err());
    assert!(format_json_source("[1,2,]", &opts()).is_err());
}

// -- JSONC -------------------------------------------------------------

#[test]
fn jsonc_without_comments_matches_strict_json() {
    for source in [
        r#"{"name":"vize","keywords":["vue","toolchain"],"nested":{"a":[1,2]}}"#,
        r#"{"a":[],"b":{}}"#,
        r#"[1,2,3]"#,
        r#""scalar""#,
    ] {
        let json = format_json_source(source, &opts()).unwrap();
        let jsonc = format_jsonc_source(source, &opts()).unwrap();
        assert_eq!(json.as_str(), jsonc.as_str(), "diverged for {source}");
    }
}

#[test]
fn jsonc_keeps_leading_line_comment_on_member() {
    let source = "{\n  // the package name\n  \"name\": \"vize\"\n}\n";
    let result = format_jsonc_source(source, &opts()).unwrap();
    assert_eq!(
        result.as_str(),
        "{\n  // the package name\n  \"name\": \"vize\"\n}\n"
    );
}

#[test]
fn jsonc_keeps_trailing_line_comment_after_comma() {
    let source = "{\n  \"a\": 1, // first\n  \"b\": 2 // last\n}\n";
    let result = format_jsonc_source(source, &opts()).unwrap();
    assert_eq!(
        result.as_str(),
        "{\n  \"a\": 1, // first\n  \"b\": 2 // last\n}\n"
    );
}

#[test]
fn jsonc_normalizes_indentation_but_keeps_comments() {
    let source = "{\n// compilerOptions\n\"compilerOptions\":{\n\"strict\":true, // be strict\n\"target\":\"ES2022\"\n}\n}";
    let result = format_jsonc_source(source, &opts()).unwrap();
    assert_eq!(
        result.as_str(),
        "{\n  // compilerOptions\n  \"compilerOptions\": {\n    \"strict\": true, // be strict\n    \"target\": \"ES2022\"\n  }\n}\n"
    );
}

#[test]
fn jsonc_drops_trailing_comma() {
    let source = "{\n  \"a\": 1,\n  \"b\": [\n    1,\n    2,\n  ],\n}\n";
    let result = format_jsonc_source(source, &opts()).unwrap();
    assert_eq!(
        result.as_str(),
        "{\n  \"a\": 1,\n  \"b\": [\n    1,\n    2\n  ]\n}\n"
    );
}

#[test]
fn jsonc_keeps_dangling_comment_before_close() {
    // A comma whose only follower is an own-line comment then `}` is a
    // genuine trailing comma: the comment is preserved, the comma dropped.
    let source = "{\n  \"a\": 1,\n  // nothing else yet\n}\n";
    let result = format_jsonc_source(source, &opts()).unwrap();
    assert_eq!(result.as_str(), "{\n  \"a\": 1\n  // nothing else yet\n}\n");
}

#[test]
fn jsonc_keeps_block_comment() {
    let source = "{ /* header */ \"a\": 1 }";
    let result = format_jsonc_source(source, &opts()).unwrap();
    assert_eq!(result.as_str(), "{\n  /* header */\n  \"a\": 1\n}\n");
}

#[test]
fn jsonc_keeps_leading_file_comment() {
    let source = "// vize config\n{\n  \"a\": 1\n}\n";
    let result = format_jsonc_source(source, &opts()).unwrap();
    assert_eq!(result.as_str(), "// vize config\n{\n  \"a\": 1\n}\n");
}

#[test]
fn jsonc_is_idempotent_across_comment_positions() {
    let source = "// top\n{\n  // lead a\n  \"a\": 1, // trail a\n  \"b\": [\n    // lead 0\n    10,\n    20, // trail 1\n  ],\n  /* block */ \"c\": true,\n  // dangling\n}\n";
    let first = format_jsonc_source(source, &opts()).unwrap();
    let second = format_jsonc_source(first.as_str(), &opts()).unwrap();
    assert_eq!(first.as_str(), second.as_str(), "first pass:\n{first}");
}

#[test]
fn jsonc_unterminated_block_comment_errors() {
    assert!(format_jsonc_source("{ /* oops \n \"a\": 1 }", &opts()).is_err());
}
