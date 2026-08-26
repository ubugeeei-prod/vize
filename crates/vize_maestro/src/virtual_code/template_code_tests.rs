//! Tests for [`super::TemplateCodeGenerator`].
//!
//! Kept in `src/virtual_code/` (via `#[path]`) rather than a `template_code/`
//! subdirectory so `insta` keeps resolving the recorded snapshots from
//! `src/virtual_code/snapshots/`.

use super::TemplateCodeGenerator;

#[test]
fn test_template_code_generator() {
    let source = r#"<div>{{ message }}</div>"#;
    let allocator = vize_s0::Allocator::new();
    let (ast, errors) = vize_armature::parse(&allocator, source);
    assert!(errors.is_empty());

    let mut generator = TemplateCodeGenerator::new();
    let doc = generator.generate(&ast, source);

    assert!(!doc.source_map.is_empty());
    insta::assert_snapshot!(doc.content.as_str());
}

#[test]
fn test_generator_with_directives() {
    let source = r#"<div v-if="show">test</div>"#;
    let allocator = vize_s0::Allocator::new();
    let (ast, errors) = vize_armature::parse(&allocator, source);
    assert!(errors.is_empty());

    let mut generator = TemplateCodeGenerator::new();
    let doc = generator.generate(&ast, source);

    insta::assert_snapshot!(doc.content.as_str());
}

#[test]
fn test_event_arrow_expression_is_not_prefixed_as_member() {
    let source = r#"<button @click="() => { count++ }">{{ count }}</button>"#;
    let allocator = vize_s0::Allocator::new();
    let (ast, errors) = vize_armature::parse(&allocator, source);
    assert!(errors.is_empty());

    let mut generator = TemplateCodeGenerator::new();
    let doc = generator.generate(&ast, source);

    assert!(
        doc.content
            .contains("const __VIZE_0 = () => { count++ };\n")
    );
    assert!(!doc.content.contains("__VIZE_ctx.() =>"));
}

#[test]
fn test_multiline_event_arrow_expression_is_not_prefixed_as_member() {
    let source = r#"<button
  @click="
    () => {
      count++;
    }
  "
>
  {{ count }}
</button>"#;
    let allocator = vize_s0::Allocator::new();
    let (ast, errors) = vize_armature::parse(&allocator, source);
    assert!(errors.is_empty());

    let mut generator = TemplateCodeGenerator::new();
    let doc = generator.generate(&ast, source);

    assert!(doc.content.contains("() =>"));
    assert!(doc.content.contains("count++;"));
    assert!(!doc.content.contains("__VIZE_ctx.\n"));
    assert!(!doc.content.contains("__VIZE_ctx.() =>"));
}
