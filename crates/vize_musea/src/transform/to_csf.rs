//! Transform Art to Storybook CSF 3.0 format.
//!
//! This module generates Storybook-compatible Component Story Format (CSF) files
//! from Art descriptors.

#![allow(clippy::disallowed_macros)]

use crate::types::{ArtDescriptor, ArtVariant, CsfOutput};
use vize_carton::{append, cstr, String, ToCompactString};

/// Transform an Art descriptor to Storybook CSF 3.0 format.
///
/// # Example
///
/// ```ignore
/// use vize_musea::transform::transform_to_csf;
/// use vize_musea::parse::parse_art;
///
/// let source = r#"
/// <art title="Button" component="./Button.vue">
///   <variant name="Primary" default>
///     <Button>Click</Button>
///   </variant>
/// </art>
/// "#;
///
/// let art = parse_art(source, Default::default()).unwrap();
/// let csf = transform_to_csf(&art);
/// ```
pub fn transform_to_csf(art: &ArtDescriptor<'_>) -> CsfOutput {
    let mut output = String::default();

    // Generate imports
    output.push_str(&generate_imports(art));
    output.push('\n');

    // Generate meta (default export)
    output.push_str(&generate_meta(art));
    output.push('\n');

    // Generate stories (named exports)
    for variant in &art.variants {
        output.push_str(&generate_story(variant, art));
        output.push('\n');
    }

    // Determine filename
    let base_name = art
        .filename
        .trim_end_matches(".art.vue")
        .rsplit('/')
        .next()
        .unwrap_or("Component");

    CsfOutput {
        code: output,
        filename: cstr!("{}.stories.ts", base_name),
    }
}

/// Generate import statements.
fn generate_imports(art: &ArtDescriptor<'_>) -> String {
    let mut imports = String::default();

    // Import from Storybook
    imports.push_str("import type { Meta, StoryObj } from '@storybook/vue3';\n");

    // Import the component
    let component_path = art.metadata.component.unwrap_or("./Component.vue");

    append!(imports, "import Component from '{component_path}';\n");

    // Add script imports if present
    if let Some(script) = &art.script_setup {
        // Extract imports from script setup
        for line in script.content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") && !trimmed.contains("Component") {
                imports.push_str(trimmed);
                imports.push('\n');
            }
        }
    }

    imports
}

/// Generate meta (default export).
fn generate_meta(art: &ArtDescriptor<'_>) -> String {
    let mut meta = String::default();

    // Build the title path
    let title = if let Some(ref category) = art.metadata.category {
        cstr!("{}/{}", category, art.metadata.title)
    } else {
        art.metadata.title.to_compact_string()
    };

    meta.push_str("const meta: Meta<typeof Component> = {\n");
    append!(meta, "  title: '{}',\n", escape_string(&title));
    meta.push_str("  component: Component,\n");

    // Add tags
    let mut tags: Vec<String> = vec!["autodocs".to_compact_string()];
    for tag in &art.metadata.tags {
        tags.push(tag.to_compact_string());
    }
    append!(
        meta,
        "  tags: [{}],\n",
        tags.iter()
            .map(|t| cstr!("'{}'", t))
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Add parameters for description
    if let Some(desc) = art.metadata.description {
        meta.push_str("  parameters: {\n");
        meta.push_str("    docs: {\n");
        meta.push_str("      description: {\n");
        append!(meta, "        component: '{}',\n", escape_string(desc));
        meta.push_str("      },\n");
        meta.push_str("    },\n");
        meta.push_str("  },\n");
    }

    meta.push_str("};\n\n");
    meta.push_str("export default meta;\n");
    meta.push_str("type Story = StoryObj<typeof meta>;\n");

    meta
}

/// Generate a story (named export) from a variant.
fn generate_story(variant: &ArtVariant<'_>, _art: &ArtDescriptor<'_>) -> String {
    let mut story = String::default();

    // Convert variant name to PascalCase for export name
    let export_name = to_pascal_case(variant.name);

    append!(story, "export const {export_name}: Story = {{\n");

    // Add name if different from export name
    if export_name != variant.name {
        append!(story, "  name: '{}',\n", escape_string(variant.name));
    }

    // Add args if present
    if !variant.args.is_empty() {
        story.push_str("  args: {\n");
        for (key, value) in &variant.args {
            let value_str = serde_json::to_string(value).unwrap_or_else(|_| "undefined".into());
            append!(story, "    {key}: {value_str},\n");
        }
        story.push_str("  },\n");
    }

    // Add render function with template
    story.push_str("  render: (args) => ({\n");
    story.push_str("    components: { Component },\n");
    story.push_str("    setup() {\n");
    story.push_str("      return { args };\n");
    story.push_str("    },\n");

    // Use the variant's template
    let template = variant.template.trim();
    append!(story, "    template: `{}`,\n", escape_template(template));

    story.push_str("  }),\n");

    // Add parameters for default story
    if variant.is_default {
        story.push_str("  parameters: {\n");
        story.push_str("    docs: {\n");
        story.push_str("      canvas: { sourceState: 'shown' },\n");
        story.push_str("    },\n");
        story.push_str("  },\n");
    }

    story.push_str("};\n");

    story
}

/// Convert a string to PascalCase.
fn to_pascal_case(s: &str) -> String {
    let mut result = String::default();
    for part in s
        .split(|c: char| c.is_whitespace() || c == '-' || c == '_')
        .filter(|p| !p.is_empty())
    {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            for uc in first.to_uppercase() {
                result.push(uc);
            }
            for ch in chars {
                result.push(ch);
            }
        }
    }
    result
}

/// Escape a string for JavaScript string literal.
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .into()
}

/// Escape a template string for JavaScript template literal.
fn escape_template(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${")
        .into()
}

#[cfg(test)]
mod tests {
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
}
