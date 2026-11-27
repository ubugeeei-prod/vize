use super::parser::{MAX_NESTING_DEPTH, NESTING_DEPTH_ERROR};
use super::{format_json_source, format_jsonc_source};
use crate::error::FormatError;
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
        "{ \"name\": \"vize\", \"version\": \"0.259.0\", \"keywords\": [\"vue\", \"toolchain\"] }\n",
    );
}

#[test]
fn preserves_key_order_from_source() {
    let source = r#"{"z":1,"a":2,"m":3}"#;
    let result = format_json_source(source, &opts()).unwrap();
    assert_eq!(result.as_str(), "{ \"z\": 1, \"a\": 2, \"m\": 3 }\n");
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
    assert_eq!(result.as_str(), "{ \"a\": [], \"b\": {} }\n");
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
fn preserves_valid_number_tokens() {
    let source = r#"{"ints":[0,-0,10],"fraction":1.25,"exp":6.02e+23,"small":1E-9}"#;
    let result = format_json_source(source, &opts()).unwrap();
    assert_eq!(
        result.as_str(),
        "{ \"ints\": [0, -0, 10], \"fraction\": 1.25, \"exp\": 6.02e+23, \"small\": 1E-9 }\n"
    );
}

#[test]
fn invalid_json_returns_error() {
    assert!(format_json_source("{\"a\":}", &opts()).is_err());
}

#[test]
fn invalid_number_tokens_return_errors() {
    for source in ["-", "01", "1.", "1e", "1e+", "[1e-]", r#"{"a":-}"#] {
        assert!(
            format_json_source(source, &opts()).is_err(),
            "strict JSON should reject {source:?}"
        );
        assert!(
            format_jsonc_source(source, &opts()).is_err(),
            "JSONC should reject {source:?}"
        );
    }
}

#[test]
fn invalid_string_escapes_return_errors() {
    for source in [r#"{"a":"\x"}"#, r#"{"a":"\q"}"#, r#"{"a":"\u12g4"}"#] {
        assert!(
            format_json_source(source, &opts()).is_err(),
            "strict JSON should reject {source:?}"
        );
        assert!(
            format_jsonc_source(source, &opts()).is_err(),
            "JSONC should reject {source:?}"
        );
    }
}

#[test]
fn honors_custom_indent_width() {
    let mut options = opts();
    options.tab_width = 4;
    let result = format_json_source(r#"{"a":1}"#, &options).unwrap();
    assert_eq!(result.as_str(), "{ \"a\": 1 }\n");
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
    assert_eq!(result.as_str(), "{\n  \"a\": 1,\n  \"b\": [1, 2]\n}\n");
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

// -- nesting depth -----------------------------------------------------

/// `depth` arrays, each the only element of the one around it.
fn nested_arrays(depth: usize) -> std::string::String {
    "[".repeat(depth) + &"]".repeat(depth)
}

/// `depth` objects, each the only member value of the one around it.
fn nested_objects(depth: usize) -> std::string::String {
    "{\"a\":".repeat(depth) + "1" + &"}".repeat(depth)
}

#[track_caller]
fn assert_nesting_depth_error(result: Result<vize_l0::String, FormatError>) {
    let Err(FormatError::JsonFormatError(message)) = result else {
        panic!("nesting past the limit must be a JSON format error");
    };
    assert_eq!(message.as_str(), NESTING_DEPTH_ERROR);
}

#[test]
fn nesting_at_the_limit_formats() {
    let source = nested_arrays(MAX_NESTING_DEPTH);
    let innermost = MAX_NESTING_DEPTH - 1;
    let mut expected = std::string::String::new();
    for depth in 0..innermost {
        expected += &"  ".repeat(depth);
        expected += "[\n";
    }
    expected += &"  ".repeat(innermost);
    expected += "[]\n";
    for depth in (0..innermost).rev() {
        expected += &"  ".repeat(depth);
        expected += "]\n";
    }
    for format in [format_json_source, format_jsonc_source] {
        let first = format(&source, &opts()).unwrap();
        assert_eq!(first.as_str(), expected);
        assert_eq!(format(first.as_str(), &opts()).unwrap().as_str(), expected);
    }
}

#[test]
fn nesting_past_the_limit_is_a_format_error() {
    for source in [
        nested_arrays(MAX_NESTING_DEPTH + 1),
        nested_objects(MAX_NESTING_DEPTH + 1),
    ] {
        assert_nesting_depth_error(format_json_source(&source, &opts()));
        assert_nesting_depth_error(format_jsonc_source(&source, &opts()));
    }
    assert!(format_json_source(&nested_objects(MAX_NESTING_DEPTH), &opts()).is_ok());
}

#[test]
fn unclosed_nesting_far_past_the_limit_is_the_same_format_error() {
    let source = "[".repeat(50_000);
    assert_nesting_depth_error(format_json_source(&source, &opts()));
    assert_nesting_depth_error(format_jsonc_source(&source, &opts()));
    let source = "{\"a\":".repeat(50_000);
    assert_nesting_depth_error(format_json_source(&source, &opts()));
    assert_nesting_depth_error(format_jsonc_source(&source, &opts()));
}

#[test]
fn sibling_collections_do_not_accumulate_depth() {
    let siblings = MAX_NESTING_DEPTH * 2;
    let source = std::format!("[{}]", vec!["[{}]"; siblings].join(","));
    let expected = std::format!("[\n{}\n]\n", vec!["  [{}]"; siblings].join(",\n"));
    let result = format_json_source(&source, &opts()).unwrap();
    assert_eq!(result.as_str(), expected);
}
