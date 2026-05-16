//! Markdown generation for individual Art components.

#![allow(clippy::disallowed_macros)]

use super::{DocOptions, DocOutput};
use crate::types::{ArtDescriptor, ArtStatus, ArtVariant};
use vize_carton::{append, cstr, String, ToCompactString};

/// Generate Markdown documentation for a single Art component.
///
/// Creates a complete documentation page with:
/// - Title and description
/// - Metadata (category, tags, status)
/// - Table of contents (for many variants)
/// - Variant documentation with templates
#[inline]
pub fn generate_component_doc(art: &ArtDescriptor<'_>, options: &DocOptions) -> DocOutput {
    let mut md = String::with_capacity(4096);

    // Title
    md.push_str("# ");
    md.push_str(art.metadata.title);
    md.push_str("\n\n");

    // Status badge if not ready
    if art.metadata.status != ArtStatus::Ready {
        md.push_str(&format_status_badge(art.metadata.status));
        md.push_str("\n\n");
    }

    // Description
    if let Some(desc) = art.metadata.description {
        md.push_str(desc);
        md.push_str("\n\n");
    }

    // Metadata section
    if options.include_metadata {
        md.push_str(&generate_metadata_section(art));
    }

    // Table of contents
    if options.include_toc && art.variants.len() >= options.toc_threshold {
        md.push_str(&generate_toc(&art.variants));
    }

    // Variants section
    md.push_str("## Variants\n\n");

    for variant in &art.variants {
        md.push_str(&generate_variant_doc(variant, options));
    }

    // Component path
    if let Some(component) = art.metadata.component {
        md.push_str("## Source\n\n");
        md.push_str("```\n");
        md.push_str(component);
        md.push_str("\n```\n\n");
    }

    // Generate filename
    let filename = cstr!("{}.md", slugify(art.metadata.title));

    DocOutput {
        markdown: md,
        filename,
        title: art.metadata.title.to_compact_string(),
        category: art.metadata.category.map(|s| s.to_compact_string()),
        variant_count: art.variants.len(),
    }
}

/// Generate Markdown documentation for a single variant.
#[inline]
pub fn generate_variant_doc(variant: &ArtVariant<'_>, options: &DocOptions) -> String {
    let mut md = String::with_capacity(512);

    // Variant heading with anchor
    md.push_str("### ");
    md.push_str(variant.name);

    // Default badge
    if variant.is_default {
        md.push_str(" `default`");
    }

    // Skip VRT badge
    if variant.skip_vrt {
        md.push_str(" `skip-vrt`");
    }

    md.push_str("\n\n");

    // Viewport info
    if let Some(ref viewport) = variant.viewport {
        append!(md, "**Viewport:** {}x{}", viewport.width, viewport.height);
        if let Some(scale) = viewport.device_scale_factor {
            append!(md, " @{:.1}x", scale);
        }
        md.push_str("\n\n");
    }

    // Args if present
    if !variant.args.is_empty() {
        md.push_str("**Args:**\n\n");
        md.push_str("| Prop | Value |\n");
        md.push_str("|------|-------|\n");
        for (key, value) in &variant.args {
            let value_str = match value {
                serde_json::Value::String(s) => format!("`\"{}\"`", s),
                serde_json::Value::Bool(b) => format!("`{}`", b),
                serde_json::Value::Number(n) => format!("`{}`", n),
                _ => format!("`{}`", value),
            };
            append!(md, "| `{key}` | {value_str} |\n");
        }
        md.push('\n');
    }

    // Template
    if options.include_templates && !variant.template.is_empty() {
        md.push_str("```vue\n");
        md.push_str(variant.template);
        md.push_str("\n```\n\n");
    }

    md.push_str("---\n\n");

    md
}

/// Generate metadata section with category, tags, etc.
fn generate_metadata_section(art: &ArtDescriptor<'_>) -> String {
    let mut md = String::default();

    let has_metadata = art.metadata.category.is_some()
        || !art.metadata.tags.is_empty()
        || art.metadata.order.is_some();

    if !has_metadata {
        return md;
    }

    md.push_str("| | |\n");
    md.push_str("|---|---|\n");

    if let Some(category) = art.metadata.category {
        append!(md, "| **Category** | `{category}` |\n");
    }

    if !art.metadata.tags.is_empty() {
        let tags: Vec<String> = art.metadata.tags.iter().map(|t| cstr!("`{}`", t)).collect();
        append!(md, "| **Tags** | {} |\n", tags.join(" "));
    }

    if let Some(order) = art.metadata.order {
        append!(md, "| **Order** | {order} |\n");
    }

    append!(md, "| **Variants** | {} |\n", art.variants.len());

    md.push('\n');

    md
}

/// Generate table of contents for variants.
fn generate_toc(variants: &[ArtVariant<'_>]) -> String {
    let mut md = String::default();

    md.push_str("## Table of Contents\n\n");

    for variant in variants {
        let anchor = slugify(variant.name);
        append!(md, "- [{}](#{})", variant.name, anchor);
        if variant.is_default {
            md.push_str(" *(default)*");
        }
        md.push('\n');
    }

    md.push('\n');

    md
}

/// Format status as a badge.
fn format_status_badge(status: ArtStatus) -> String {
    match status {
        ArtStatus::Draft => "> **Status:** 🚧 Draft".to_compact_string(),
        ArtStatus::Deprecated => "> **Status:** ⚠️ Deprecated".to_compact_string(),
        ArtStatus::Ready => String::default(),
    }
}

/// Convert a string to a URL-safe slug.
#[inline]
fn slugify(s: &str) -> String {
    let intermediate: String = s
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let joined = intermediate
        .as_str()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    joined.into()
}

#[cfg(test)]
mod tests {
    use super::{format_status_badge, slugify};
    use crate::types::ArtStatus;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("With Icon"), "with-icon");
        assert_eq!(slugify("my-button"), "my-button");
        assert_eq!(slugify("Button_Primary"), "button-primary");
    }

    #[test]
    fn test_format_status_badge() {
        insta::assert_snapshot!(format_status_badge(ArtStatus::Draft).as_str());
        insta::assert_snapshot!(format_status_badge(ArtStatus::Deprecated).as_str());
        assert!(format_status_badge(ArtStatus::Ready).is_empty());
    }
}
