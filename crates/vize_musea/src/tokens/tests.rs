use super::{
    build_token_map, generate_tokens_markdown, parse_tokens_from_json, parse_tokens_from_path,
    resolve_token_categories, validate_reference, TokenCategory,
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

fn contains_category(categories: &[TokenCategory], expected: &str) -> bool {
    categories.iter().any(|category| {
        category.name.as_str() == expected || contains_category(&category.subcategories, expected)
    })
}

#[cfg(unix)]
#[test]
fn skips_symlinked_token_entries_when_merging_a_directory() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("vize-musea-tokens-{unique}"));
    let tokens = root.join("tokens");
    let outside = root.join("outside");
    std::fs::create_dir_all(&tokens).unwrap();
    std::fs::create_dir_all(&outside).unwrap();

    std::fs::write(
        tokens.join("safe.json"),
        r##"{"color":{"brand":{"value":"#111111"}}}"##,
    )
    .unwrap();
    std::fs::write(
        outside.join("secret.json"),
        r#"{"secret":{"leak":{"value":"should-not-appear"}}}"#,
    )
    .unwrap();
    std::os::unix::fs::symlink(outside.join("secret.json"), tokens.join("leak.json")).unwrap();
    std::os::unix::fs::symlink(&outside, tokens.join("nested")).unwrap();

    let categories = parse_tokens_from_path(&tokens).unwrap();
    let parsed_safe = contains_category(&categories, "Color");
    let parsed_secret = contains_category(&categories, "Secret");
    let _ = std::fs::remove_dir_all(&root);

    assert!(parsed_safe, "in-tree token files must still be parsed");
    assert!(!parsed_secret, "symlinked token files must not be merged");
}
