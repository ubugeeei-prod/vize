use super::{
    build_token_map, generate_tokens_markdown, parse_tokens_from_json, resolve_token_categories,
    validate_reference,
};

#[test]
fn parses_and_resolves_token_references() {
    let categories = parse_tokens_from_json(
        r##"{
  "color": {
    "primitive": {
      "gray": { "50": { "value": "#f7f7f7", "type": "color" } }
    },
    "semantic": {
      "surface": { "value": "{color.primitive.gray.50}" }
    }
  }
}"##,
    )
    .unwrap();

    let resolved = resolve_token_categories(categories);
    insta::assert_debug_snapshot!(resolved);
}

#[test]
fn validates_reference_cycles() {
    let categories = parse_tokens_from_json(
        r##"{
  "color": {
    "a": { "value": "{color.b}" },
    "b": { "value": "{color.a}" }
  }
}"##,
    )
    .unwrap();
    let map = build_token_map(&categories);

    insta::assert_debug_snapshot!(validate_reference(&map, "color.a", Some("color.b")));
}

#[test]
fn renders_markdown_snapshot() {
    let categories = parse_tokens_from_json(
        r##"{
  "spacing": {
    "sm": { "value": "4px", "description": "Small gap" },
    "md": { "value": 8 }
  }
}"##,
    )
    .unwrap();

    insta::assert_snapshot!(generate_tokens_markdown(
        &categories,
        Some("2026-05-17T00:00:00.000Z")
    ));
}
