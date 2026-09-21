use super::parse_art;
use crate::types::{ArtParseError, ArtParseOptions, ArtStatus};
use vize_s0::Allocator;

#[test]
fn test_parse_simple_art() {
    let allocator = Allocator::new();
    let source = r#"
<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button>Click me</Button>
  </variant>
</art>

<script setup lang="ts">
import Button from "./Button.vue";
</script>
"#;

    let desc = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
    assert_eq!(desc.metadata.title, "Button");
    assert_eq!(desc.metadata.component, Some("./Button.vue"));
    assert_eq!(desc.variants.len(), 1);
    assert_eq!(desc.variants[0].name, "Primary");
    assert!(desc.variants[0].is_default);
    let script = desc.script_setup.expect("script setup");
    assert_eq!(
        (script.content, script.lang, script.setup),
        (r#"import Button from "./Button.vue";"#, Some("ts"), true)
    );
}

#[test]
fn test_parse_multiple_variants() {
    let allocator = Allocator::new();
    let source = r#"
<art title="Button">
  <variant name="Primary" default>
    <Button variant="primary">Click</Button>
  </variant>
  <variant name="Secondary">
    <Button variant="secondary">Click</Button>
  </variant>
  <variant name="Disabled">
    <Button disabled>Click</Button>
  </variant>
</art>
"#;

    let desc = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
    let names: std::vec::Vec<_> = desc.variants.iter().map(|v| v.name).collect();
    assert_eq!(names, ["Primary", "Secondary", "Disabled"]);
}

#[test]
fn test_parse_define_art_metadata() {
    let allocator = Allocator::new();
    let source = r#"
<script setup lang="ts">
import Button from "./Button.vue";

defineArt(Button, {
  title: "Button",
  description: "A button component",
  category: "Components",
  tags: ["button", "ui"],
  status: "draft",
  order: 2,
});
</script>

<art>
  <variant name="Primary" default>
    <Button>Click</Button>
  </variant>
</art>
"#;

    let desc = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();

    assert_eq!(desc.metadata.title, "Button");
    assert_eq!(desc.metadata.component, Some("./Button.vue"));
    assert_eq!(desc.metadata.description, Some("A button component"));
    assert_eq!(desc.metadata.category, Some("Components"));
    assert_eq!(desc.metadata.tags.as_slice(), ["button", "ui"]);
    assert_eq!(desc.metadata.status, ArtStatus::Draft);
    assert_eq!(desc.metadata.order, Some(2));
}

#[test]
fn test_parse_define_art_source_literal_metadata() {
    let allocator = Allocator::new();
    let source = r#"
<script setup lang="ts">
defineArt("./base-button.vue", {
  title: "Base Button",
  category: "Components",
});
</script>

<art>
  <variant name="Primary" default>
    <BaseButton>Click</BaseButton>
  </variant>
</art>
"#;

    let desc = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();

    assert_eq!(desc.metadata.title, "Base Button");
    assert_eq!(desc.metadata.component, Some("./base-button.vue"));
    assert_eq!(desc.metadata.category, Some("Components"));
}

#[test]
fn test_missing_title_error() {
    let allocator = Allocator::new();
    let source = r#"<art><variant name="Test"></variant></art>"#;
    let result = parse_art(&allocator, source, ArtParseOptions::default());
    assert!(matches!(result, Err(ArtParseError::MissingTitle)));
}

#[test]
fn test_no_art_block_error() {
    let allocator = Allocator::new();
    let source = r#"<template><div>Hello</div></template>"#;
    let result = parse_art(&allocator, source, ArtParseOptions::default());
    assert!(matches!(result, Err(ArtParseError::NoArtBlock)));
}

#[test]
fn container_errors_surface_as_parse_errors() {
    let allocator = Allocator::new();
    let source = "<art title=\"A\"></art>\n<script setup>\n</script>\n<script setup>\n</script>\n";
    let Err(ArtParseError::ParseError { line, message }) =
        parse_art(&allocator, source, ArtParseOptions::default())
    else {
        panic!("expected the SFC splitter's duplicate-block error");
    };
    assert_eq!(
        (line, message.as_str()),
        (4, "SFC can only contain one <script setup> block")
    );
}

#[test]
fn every_slice_borrows_the_source_at_file_absolute_offsets() {
    let allocator = Allocator::new();
    let source = "<script setup lang=\"ts\">\nconst a = 1\n</script>\n<art title=\"T\">\n  <variant name=\"V\">\n    <b>x</b>\n  </variant>\n</art>\n<style scoped lang=\"css\">\n.a {}\n</style>\n";
    let desc = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
    let at = |slice: &str| slice.as_ptr() as usize - source.as_ptr() as usize;
    let variant = &desc.variants[0];
    let loc = variant.loc.expect("variant loc");
    assert_eq!(
        (at(variant.template), variant.template, at(variant.name)),
        (
            source.find("<b>").unwrap(),
            "<b>x</b>",
            source.find("V\"").unwrap()
        )
    );
    assert_eq!(
        (loc.start, loc.end, loc.start_line, loc.start_column),
        (
            source.find("<variant").unwrap() as u32,
            (source.find("</variant>").unwrap() + 10) as u32,
            5,
            2
        )
    );
    assert_eq!(at(desc.metadata.title), source.find("T\"").unwrap());
    let style = &desc.styles[0];
    let style_loc = style.loc.expect("style loc");
    assert_eq!(
        (style.content, style.lang, style.scoped),
        (".a {}", Some("css"), true)
    );
    assert_eq!(
        (style_loc.start, style_loc.end, style_loc.start_line),
        (
            source.find("<style").unwrap() as u32,
            source.len() as u32 - 1,
            9
        )
    );
}
