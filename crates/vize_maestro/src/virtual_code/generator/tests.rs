use super::{
    ArtCursorPosition, BatchVirtualCodeGenerator, BlockType, VirtualCodeGenerator, VirtualLanguage,
    find_art_block_at_offset, find_block_at_offset,
};

#[test]
fn test_virtual_code_generator() {
    let source = r#"<template>
  <div>{{ message }}</div>
</template>

<script setup lang="ts">
const message = ref('hello')
</script>

<style scoped>
.container { color: red; }
</style>"#;

    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();

    let mut generator = VirtualCodeGenerator::new();
    let docs = generator.generate(&descriptor, "test.vue");

    assert!(docs.template.is_some());
    assert!(docs.script_setup.is_some());
    assert_eq!(docs.styles.len(), 1);

    // Check template virtual code
    let template = docs.template.unwrap();
    assert!(!template.source_map.is_empty());
    insta::assert_snapshot!(template.content.as_str());
}

#[test]
fn test_script_setup_exports_template_used_bindings() {
    let source = r#"<script setup lang="ts">
const count = ref(0)
function handleClick() {
  count.value++
}
const double = computed(() => count.value * 2)
const unused = 1
</script>

<template>
  <button @click="handleClick">{{ count }}</button>
  <p>{{ double }}</p>
</template>"#;

    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
    let mut generator = VirtualCodeGenerator::new();
    let docs = generator.generate(&descriptor, "test.vue");
    let script_setup = docs.script_setup.unwrap();

    assert!(
        script_setup
            .content
            .contains("export { count, double, handleClick };")
    );
    assert!(
        !script_setup
            .content
            .contains("export { count, double, handleClick, unused };")
    );
}

#[test]
fn test_batch_generator() {
    let source1 = "<template><div>{{ a }}</div></template>";
    let source2 = "<template><div>{{ b }}</div></template>";

    let desc1 = vize_atelier_sfc::parse_sfc(source1, Default::default()).unwrap();
    let desc2 = vize_atelier_sfc::parse_sfc(source2, Default::default()).unwrap();

    let mut batch = BatchVirtualCodeGenerator::new();
    let results = batch.generate_batch(&[(&desc1, "file1.vue"), (&desc2, "file2.vue")]);

    assert_eq!(results.len(), 2);
    assert!(results[0].template.is_some());
    assert!(results[1].template.is_some());
}

#[test]
fn test_find_block_at_offset() {
    let source = r#"<template>
  <div>test</div>
</template>

<script setup>
const x = 1
</script>"#;

    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();

    // In template
    assert_eq!(
        find_block_at_offset(&descriptor, 15),
        Some(BlockType::Template)
    );

    // In script setup
    assert_eq!(
        find_block_at_offset(&descriptor, 60),
        Some(BlockType::ScriptSetup)
    );
}

#[test]
fn test_find_block_at_offset_inline_art() {
    let source = r#"<template>
  <div>test</div>
</template>

<script setup>
const x = 1
</script>

<art title="Test" component="./Foo.vue">
  <variant name="Default" default>
    <Foo />
  </variant>
</art>"#;

    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();

    // Verify custom_blocks contains the art block
    assert_eq!(descriptor.custom_blocks.len(), 1);
    assert_eq!(descriptor.custom_blocks[0].block_type, "art");

    // Offset inside <art> content area before the first variant
    let art_content_start = descriptor.custom_blocks[0].loc.start;
    assert_eq!(
        find_block_at_offset(&descriptor, art_content_start + 1),
        Some(BlockType::Art(ArtCursorPosition::ArtContent))
    );
    assert_eq!(
        find_block_at_offset(&descriptor, source.find("<variant").unwrap() + 1),
        Some(BlockType::Art(ArtCursorPosition::VariantTag(0)))
    );

    // In template - should still be Template
    assert_eq!(
        find_block_at_offset(&descriptor, 15),
        Some(BlockType::Template)
    );

    // Outside any block
    assert_eq!(find_block_at_offset(&descriptor, 0), None);
}

#[test]
fn test_find_block_at_offset_detects_inline_art_variant_template() {
    let source = r#"<template><button /></template>

<art>
  <variant name="Primary">
    <Self variant="primary" />
  </variant>
</art>"#;

    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
    let offset = source.find("variant=\"primary\"").unwrap();

    let Some(BlockType::Art(ArtCursorPosition::VariantTemplate(info))) =
        find_block_at_offset(&descriptor, offset)
    else {
        panic!("expected inline art variant template");
    };

    assert_eq!(info.variant_index, 0);
    assert_eq!(info.template_start, source.find("<Self").unwrap());
}

#[test]
fn test_block_type_art_language() {
    assert_eq!(
        BlockType::Art(ArtCursorPosition::ArtContent).language(),
        VirtualLanguage::Template
    );
}

#[test]
fn test_find_art_block_at_offset() {
    let source = r#"<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button>Click me</Button>
  </variant>
</art>

<script setup lang="ts">
import Button from './Button.vue'
</script>"#;

    // In script setup
    let script_offset = source.find("import Button").unwrap();
    assert_eq!(
        find_art_block_at_offset(source, script_offset),
        Some(BlockType::ScriptSetup)
    );

    // In variant template content
    let template_offset = source.find("<Button>Click me</Button>").unwrap();
    let result = find_art_block_at_offset(source, template_offset);
    assert!(matches!(
        result,
        Some(BlockType::Art(ArtCursorPosition::VariantTemplate(_)))
    ));

    // In art content (between variants)
    let art_content_offset = source.find("\n  <variant").unwrap() + 1;
    // This offset is just before <variant, which is inside the <art> but before variant tag starts
    // It should be ArtContent
    assert!(matches!(
        find_art_block_at_offset(source, art_content_offset),
        Some(BlockType::Art(_))
    ));
}

#[test]
fn test_find_art_block_at_offset_treats_variant_body_whitespace_as_template() {
    let source = r#"<art title="Button" component="./Button.vue">
  <variant name="Primary" default>

    <Button>Click me</Button>
  </variant>
</art>"#;

    let body_whitespace_offset = source.find("\n\n    <Button>").unwrap() + 1;
    let result = find_art_block_at_offset(source, body_whitespace_offset);

    let Some(BlockType::Art(ArtCursorPosition::VariantTemplate(info))) = result else {
        panic!("expected variant template, got {result:?}");
    };

    assert_eq!(info.relative_offset, 0);
    assert_eq!(info.template_start, source.find("<Button>").unwrap());
}

#[test]
fn test_find_art_block_at_offset_treats_variant_body_as_template() {
    let source = r#"<script setup>
const count = ref(0)
</script>

<art title="Counter" component="./Counter.vue">
  <variant name="Interactive">
    <Counter :count="count" />
  </variant>
</art>"#;

    let offset = source.find("count = ref").unwrap();
    assert_eq!(
        find_art_block_at_offset(source, offset),
        Some(BlockType::ScriptSetup)
    );

    let template_offset = source.find(":count=\"count\"").unwrap();
    assert!(matches!(
        find_art_block_at_offset(source, template_offset),
        Some(BlockType::Art(ArtCursorPosition::VariantTemplate(_)))
    ));
}
