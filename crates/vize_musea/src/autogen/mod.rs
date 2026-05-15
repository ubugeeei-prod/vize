//! Variant auto-generation module.
//!
//! Generates `.art.vue` variant definitions from component prop analysis.
//! Uses intelligent strategies instead of cartesian product to produce
//! meaningful, manageable variant sets.

#![allow(clippy::disallowed_macros)]

pub mod strategy;
pub mod types;

pub use strategy::generate_variants;
pub use types::{AutogenConfig, AutogenOutput, GeneratedVariant, PropDefinition};

use std::path::Path;
use vize_carton::{append, String, ToCompactString};

/// Generate an `.art.vue` file from prop definitions.
pub fn generate_art_file(
    component_path: &str,
    props: &[PropDefinition],
    config: &AutogenConfig,
) -> AutogenOutput {
    let component_name = extract_component_name(component_path);
    let variants = generate_variants(props, &component_name, config);
    let art_file_content = render_art_file(&component_name, component_path, &variants);

    AutogenOutput {
        variants,
        art_file_content,
        component_name,
    }
}

/// Extract component name from file path.
/// e.g., "./components/MyButton.vue" -> "MyButton"
fn extract_component_name(component_path: &str) -> String {
    Path::new(component_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Component")
        .to_compact_string()
}

/// Render the `.art.vue` file content from generated variants.
fn render_art_file(
    component_name: &str,
    component_path: &str,
    variants: &[GeneratedVariant],
) -> String {
    let mut output = String::default();

    // <art> block
    append!(
        output,
        "<art title=\"{component_name}\" component=\"{component_path}\">\n"
    );

    // Variants
    for variant in variants {
        let attrs = if variant.is_default {
            format!("name=\"{}\" default", variant.name)
        } else {
            format!("name=\"{}\"", variant.name)
        };

        append!(output, "  <variant {attrs}>\n");

        // Build component tag with props
        let props_str = render_props(&variant.props);
        if props_str.is_empty() {
            append!(output, "    <{component_name} />\n");
        } else {
            append!(output, "    <{component_name}\n");
            output.push_str(&props_str);
            output.push_str("    />\n");
        }

        output.push_str("  </variant>\n\n");
    }

    output.push_str("</art>\n\n");

    // Script setup
    output.push_str("<script setup lang=\"ts\">\n");
    append!(output, "import {component_name} from '{component_path}'\n");
    output.push_str("</script>\n");

    output
}

/// Render props as Vue template attributes.
#[allow(clippy::disallowed_types)]
fn render_props(props: &serde_json::Map<std::string::String, serde_json::Value>) -> String {
    let mut lines = Vec::new();

    for (name, value) in props {
        let attr: std::string::String = match value {
            serde_json::Value::String(s) => format!("      {name}=\"{s}\""),
            serde_json::Value::Bool(true) => format!("      {name}"),
            serde_json::Value::Bool(false) => format!("      :{name}=\"false\""),
            serde_json::Value::Number(n) => format!("      :{name}=\"{n}\""),
            serde_json::Value::Null => continue,
            other => {
                let json_str = serde_json::to_string(other).unwrap_or_default();
                let escaped = json_str.replace('"', "'");
                format!("      :{name}=\"{escaped}\"")
            }
        };
        lines.push(attr);
    }

    if lines.is_empty() {
        String::default()
    } else {
        let joined: std::string::String = lines.join("\n") + "\n";
        joined.into()
    }
}

#[cfg(test)]
#[allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
mod tests {
    use super::{
        extract_component_name, generate_art_file, render_props, AutogenConfig, PropDefinition,
    };
    use serde_json::json;

    #[test]
    fn test_extract_component_name() {
        assert_eq!(extract_component_name("./Button.vue"), "Button");
        assert_eq!(
            extract_component_name("../components/MyButton.vue"),
            "MyButton"
        );
        assert_eq!(extract_component_name("Input.vue"), "Input");
    }

    #[test]
    fn test_generate_art_file() {
        let props = vec![
            PropDefinition {
                name: "variant".into(),
                prop_type: "'primary' | 'secondary'".into(),
                required: true,
                default_value: Some(json!("primary")),
            },
            PropDefinition {
                name: "label".into(),
                prop_type: "string".into(),
                required: true,
                default_value: Some(json!("Click me")),
            },
            PropDefinition {
                name: "disabled".into(),
                prop_type: "boolean".into(),
                required: false,
                default_value: Some(json!(false)),
            },
        ];

        let config = AutogenConfig::default();
        let output = generate_art_file("./Button.vue", &props, &config);

        insta::assert_debug_snapshot!(output);
    }

    #[test]
    fn test_render_props() {
        let mut props = serde_json::Map::new();
        props.insert("label".to_string(), json!("Hello"));
        props.insert("disabled".to_string(), json!(true));
        props.insert("count".to_string(), json!(42));

        let rendered = render_props(&props);
        insta::assert_snapshot!(rendered.as_str());
    }

    #[test]
    fn test_render_props_edge_values() {
        let mut props = serde_json::Map::new();
        props.insert("empty".to_string(), json!(""));
        props.insert("disabled".to_string(), json!(false));
        props.insert(
            "config".to_string(),
            json!({ "size": "sm", "nested": true }),
        );
        props.insert("ignored".to_string(), serde_json::Value::Null);

        let rendered = render_props(&props);
        insta::assert_snapshot!(rendered.as_str());
    }
}
