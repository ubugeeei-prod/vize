use super::{escape_string, escape_template, to_pascal_case, transform_to_csf};
use crate::parse::parse_art;
use crate::types::ArtParseOptions;
use vize_carton::Bump;

fn transform(source: &str) -> crate::types::CsfOutput {
    let allocator = Bump::new();
    let art = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
    transform_to_csf(&art)
}

#[test]
fn transform_simple() {
    let source = r#"
<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button variant="primary">Click me</Button>
  </variant>
</art>
"#;

    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_debug_snapshot!("transform_simple", transform(source));
    });
}

#[test]
fn transform_with_category() {
    let source = r#"
<art title="Button" category="atoms" component="./Button.vue">
  <variant name="Default">
    <Button>Click</Button>
  </variant>
</art>
"#;

    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_debug_snapshot!("transform_with_category", transform(source));
    });
}

#[test]
fn transform_multiple_variants() {
    let source = r#"
<art title="Button" component="./Button.vue">
  <variant name="Primary">
    <Button variant="primary">Primary</Button>
  </variant>
  <variant name="Secondary">
    <Button variant="secondary">Secondary</Button>
  </variant>
</art>
"#;

    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_debug_snapshot!("transform_multiple_variants", transform(source));
    });
}

#[test]
fn transform_preserves_script_context_for_storybook() {
    let source = r#"
<script setup lang="ts">
import {
  base,
  preparing,
  finished,
} from "./fixtures";
import type { FixtureShape } from "./types";

const notSucceeded: FixtureShape = { id: "not-succeeded" };

defineArt("./MoshiDetailCard.vue", {
  title: "MoshiDetailCard",
  category: "Features/MoshiDetail",
  tags: ["moshi", "detail"],
});
</script>

<art>
  <variant name="Available" default>
    <MoshiDetailCard :moshi-with-student="base" />
  </variant>
  <variant name="Preparing">
    <MoshiDetailCard :moshi-with-student="preparing" />
  </variant>
  <variant name="Finished">
    <MoshiDetailCard :moshi-with-student="finished" />
  </variant>
  <variant name="Not Succeeded">
    <MoshiDetailCard :moshi-with-student="notSucceeded" />
  </variant>
</art>
"#;

    let csf = transform(source);

    assert!(
        csf.code
            .contains("import {\n  base,\n  preparing,\n  finished,\n} from \"./fixtures\";")
    );
    assert!(
        csf.code
            .contains("import type { FixtureShape } from \"./types\";")
    );
    assert!(csf.code.contains("const notSucceeded: FixtureShape"));
    assert!(!csf.code.contains("defineArt("));
    assert!(
        csf.code
            .contains("return { args, base, finished, notSucceeded, preparing };")
    );
}

#[test]
fn test_to_pascal_case() {
    assert_eq!(to_pascal_case("primary"), "Primary");
    assert_eq!(to_pascal_case("with icon"), "WithIcon");
    assert_eq!(to_pascal_case("my-button"), "MyButton");
    assert_eq!(to_pascal_case("my_button"), "MyButton");
}

#[test]
fn test_escape_string() {
    assert_eq!(escape_string("hello"), "hello");
    assert_eq!(escape_string("it's"), "it\\'s");
    assert_eq!(escape_string("line\nbreak"), "line\\nbreak");
}

#[test]
fn test_escape_template() {
    assert_eq!(escape_template("hello"), "hello");
    assert_eq!(escape_template("`code`"), "\\`code\\`");
    assert_eq!(escape_template("${var}"), "\\${var}");
}
