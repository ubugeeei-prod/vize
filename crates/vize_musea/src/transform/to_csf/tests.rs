use super::{escape_string, escape_template, to_pascal_case, transform_to_csf};
use crate::parse::parse_art;
use crate::types::ArtParseOptions;
use vize_carton::Bump;

#[test]
fn test_transform_simple() {
    let allocator = Bump::new();
    let source = r#"
<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button variant="primary">Click me</Button>
  </variant>
</art>
"#;

    let art = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
    let csf = transform_to_csf(&art);

    insta::assert_debug_snapshot!(csf);
}

#[test]
fn test_transform_with_category() {
    let allocator = Bump::new();
    let source = r#"
<art title="Button" category="atoms" component="./Button.vue">
  <variant name="Default">
    <Button>Click</Button>
  </variant>
</art>
"#;

    let art = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
    let csf = transform_to_csf(&art);

    insta::assert_debug_snapshot!(csf);
}

#[test]
fn test_transform_multiple_variants() {
    let allocator = Bump::new();
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

    let art = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
    let csf = transform_to_csf(&art);

    insta::assert_debug_snapshot!(csf);
}

#[test]
fn test_transform_preserves_script_setup_fixtures() {
    let allocator = Bump::new();
    let source = r#"
<script setup lang="ts">
import MoshiDetailCard from './MoshiDetailCard.vue';
import {
  base,
  preparing,
  finished,
  notSucceeded,
} from './fixtures';

defineArt("./MoshiDetailCard.vue", {
  title: "Moshi Detail Card",
  category: "Features/MoshiDetail",
  tags: ["moshi", "detail"],
});

const localFixture = { id: "local" };
</script>

<art>
  <variant name="Available" default>
    <MoshiDetailCard :moshi-with-student="base" />
  </variant>
  <variant name="Preparing">
    <MoshiDetailCard :moshi-with-student="preparing" :fallback="localFixture" />
  </variant>
</art>
"#;

    let art = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
    let csf = transform_to_csf(&art);

    assert!(csf.code.contains(
        "import {\n  base,\n  preparing,\n  finished,\n  notSucceeded,\n} from './fixtures';"
    ));
    assert!(!csf.code.contains("import {\n\nconst"));
    assert!(!csf.code.contains("defineArt"));
    assert!(csf.code.contains("const localFixture = { id: \"local\" };"));
    assert!(csf.code.contains(
        "return { args, MoshiDetailCard, base, finished, localFixture, notSucceeded, preparing };"
    ));
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
