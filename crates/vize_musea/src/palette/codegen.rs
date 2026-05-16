//! Palette code generation from Art descriptors.

#![allow(clippy::disallowed_macros)]

use super::inference::infer_control_from_values;
use super::{Palette, PaletteOptions, PaletteOutput, PropControl};
use crate::types::ArtDescriptor;
use vize_carton::{append, cstr, FxHashMap, String, ToCompactString};

/// Generate palette configuration from an Art descriptor.
///
/// Collects all args from variants, infers control types,
/// and generates a complete palette configuration.
#[inline]
pub fn generate_palette(art: &ArtDescriptor<'_>, options: &PaletteOptions) -> PaletteOutput {
    let mut palette = Palette::new(art.metadata.title);

    // Collect all values for each prop across variants
    let mut all_values: FxHashMap<String, Vec<serde_json::Value>> = FxHashMap::default();

    for variant in &art.variants {
        for (key, value) in &variant.args {
            all_values
                .entry(key.to_compact_string())
                .or_default()
                .push(value.clone());
        }
    }

    // Store all values in palette for reference
    palette.all_values = all_values.clone();

    // Generate controls for each prop
    let mut prop_names: Vec<_> = all_values.keys().collect();
    prop_names.sort(); // Stable ordering

    for prop_name in prop_names {
        let values = &all_values[prop_name];
        let (control_kind, select_options, range_config) =
            infer_control_from_values(values, options);

        let mut control = PropControl {
            name: prop_name.clone(),
            control: control_kind,
            default_value: values.first().cloned(),
            description: None,
            required: false,
            options: select_options,
            range: range_config,
            group: if options.group_by_type {
                Some(cstr!("{:?}", control_kind))
            } else {
                None
            },
        };

        // Add group if grouping by type
        if options.group_by_type {
            control.group = Some(cstr!("{:?}", control.control));
        }

        palette.add_control(control);
    }

    // Generate JSON representation
    let json: String = serde_json::to_string_pretty(&palette)
        .unwrap_or_default()
        .into();

    // Generate TypeScript interface
    let typescript = generate_typescript_interface(&palette);

    PaletteOutput {
        palette,
        json,
        typescript,
    }
}

/// Generate TypeScript interface for props.
fn generate_typescript_interface(palette: &Palette) -> String {
    let mut ts = String::with_capacity(512);

    ts.push_str("export interface ");
    ts.push_str(&to_pascal_case(&palette.title));
    ts.push_str("Props {\n");

    for control in &palette.controls {
        ts.push_str("  ");
        ts.push_str(&control.name);

        if !control.required {
            ts.push('?');
        }

        ts.push_str(": ");
        ts.push_str(&control_to_ts_type(control));
        ts.push_str(";\n");
    }

    ts.push_str("}\n");

    ts
}

/// Convert control to TypeScript type.
fn control_to_ts_type(control: &PropControl) -> String {
    use super::ControlKind;

    match control.control {
        ControlKind::Text => "string".to_compact_string(),
        ControlKind::Number | ControlKind::Range => "number".to_compact_string(),
        ControlKind::Boolean => "boolean".to_compact_string(),
        ControlKind::Color => "string".to_compact_string(),
        ControlKind::Date => "string | Date".to_compact_string(),
        ControlKind::Select | ControlKind::Radio => {
            if control.options.is_empty() {
                "string".to_compact_string()
            } else {
                let joined = control
                    .options
                    .iter()
                    .map(|opt| match &opt.value {
                        serde_json::Value::String(s) => cstr!("'{}'", s),
                        serde_json::Value::Number(n) => n.to_compact_string(),
                        serde_json::Value::Bool(b) => b.to_compact_string(),
                        _ => "unknown".to_compact_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(" | ");
                joined.into()
            }
        }
        ControlKind::Array => "unknown[]".to_compact_string(),
        ControlKind::Object => "Record<string, unknown>".to_compact_string(),
        ControlKind::File => "File".to_compact_string(),
        ControlKind::Raw => "unknown".to_compact_string(),
    }
}

/// Convert string to PascalCase.
#[inline]
fn to_pascal_case(s: &str) -> String {
    let mut result = String::default();
    for word in s
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
    {
        let mut chars = word.chars();
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

/// Generate Vue component props definition.
#[allow(dead_code)]
pub fn generate_vue_props(palette: &Palette) -> String {
    let mut vue = String::with_capacity(512);

    vue.push_str("const props = defineProps<{\n");

    for control in &palette.controls {
        vue.push_str("  ");
        vue.push_str(&control.name);

        if !control.required {
            vue.push('?');
        }

        vue.push_str(": ");
        vue.push_str(&control_to_ts_type(control));
        vue.push('\n');
    }

    vue.push_str("}>()\n");

    vue
}

/// Generate Storybook argTypes definition.
#[allow(dead_code)]
pub fn generate_storybook_argtypes(palette: &Palette) -> String {
    use super::ControlKind;

    let mut sb = String::with_capacity(1024);

    sb.push_str("export const argTypes = {\n");

    for control in &palette.controls {
        sb.push_str("  ");
        sb.push_str(&control.name);
        sb.push_str(": {\n");

        // Control type
        sb.push_str("    control: { type: '");
        sb.push_str(match control.control {
            ControlKind::Text => "text",
            ControlKind::Number => "number",
            ControlKind::Boolean => "boolean",
            ControlKind::Range => "range",
            ControlKind::Select => "select",
            ControlKind::Radio => "radio",
            ControlKind::Color => "color",
            ControlKind::Date => "date",
            ControlKind::Object => "object",
            ControlKind::Array => "object",
            ControlKind::File => "file",
            ControlKind::Raw => "text",
        });
        sb.push('\'');

        // Range config
        if let Some(ref range) = control.range {
            append!(sb, ", min: {}, max: {}", range.min, range.max);
            if let Some(step) = range.step {
                append!(sb, ", step: {step}");
            }
        }

        sb.push_str(" },\n");

        // Options
        if !control.options.is_empty() {
            sb.push_str("    options: [");
            for (i, opt) in control.options.iter().enumerate() {
                if i > 0 {
                    sb.push_str(", ");
                }
                match &opt.value {
                    serde_json::Value::String(s) => append!(sb, "'{s}'"),
                    serde_json::Value::Number(n) => append!(sb, "{n}"),
                    serde_json::Value::Bool(b) => append!(sb, "{b}"),
                    _ => sb.push_str("null"),
                }
            }
            sb.push_str("],\n");
        }

        // Description
        if let Some(ref desc) = control.description {
            append!(sb, "    description: '{desc}',\n");
        }

        // Default value
        if let Some(ref default) = control.default_value {
            sb.push_str("    defaultValue: ");
            match default {
                serde_json::Value::String(s) => append!(sb, "'{s}'"),
                serde_json::Value::Number(n) => append!(sb, "{n}"),
                serde_json::Value::Bool(b) => append!(sb, "{b}"),
                _ => append!(sb, "{default}"),
            }
            sb.push_str(",\n");
        }

        // Table category (group)
        if let Some(ref group) = control.group {
            append!(sb, "    table: {{ category: '{group}' }},\n");
        }

        sb.push_str("  },\n");
    }

    sb.push_str("};\n");

    sb
}

#[cfg(test)]
#[allow(clippy::disallowed_methods, clippy::disallowed_macros)]
mod tests {
    use super::super::PaletteOptions;
    use super::{
        generate_palette, generate_storybook_argtypes, generate_typescript_interface,
        to_pascal_case, Palette, PropControl,
    };
    use crate::{parse_art, ArtParseOptions, Bump};

    #[test]
    fn test_generate_palette_basic() {
        let allocator = Bump::new();
        let source = r#"
<art title="Button">
  <variant name="Primary" args='{"variant":"primary","size":"md","disabled":false}'>
    <Button>Click</Button>
  </variant>
  <variant name="Secondary" args='{"variant":"secondary","size":"lg","disabled":true}'>
    <Button>Click</Button>
  </variant>
</art>
"#;

        let art = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
        let output = generate_palette(&art, &PaletteOptions::default());

        assert_eq!(output.palette.title, "Button");
        assert_eq!(output.palette.controls.len(), 3);

        // Check variant control is select with options
        let variant_ctrl = output
            .palette
            .controls
            .iter()
            .find(|c| c.name == "variant")
            .unwrap();
        assert_eq!(variant_ctrl.control, super::super::ControlKind::Select);
        assert_eq!(variant_ctrl.options.len(), 2);

        // Check disabled control is boolean
        let disabled_ctrl = output
            .palette
            .controls
            .iter()
            .find(|c| c.name == "disabled")
            .unwrap();
        assert_eq!(disabled_ctrl.control, super::super::ControlKind::Boolean);
    }

    #[test]
    fn test_generate_typescript_interface() {
        let mut palette = Palette::new("Button");
        palette.add_control(PropControl::text("label").required());
        palette.add_control(PropControl::boolean("disabled"));
        palette.add_control(PropControl::number("size"));

        let ts = generate_typescript_interface(&palette);

        insta::assert_snapshot!(ts.as_str());
    }

    #[test]
    fn test_generate_storybook_argtypes() {
        let mut palette = Palette::new("Button");
        palette.add_control(
            PropControl::select(
                "size",
                vec![
                    super::super::SelectOption {
                        label: "Small".into(),
                        value: serde_json::json!("sm"),
                    },
                    super::super::SelectOption {
                        label: "Large".into(),
                        value: serde_json::json!("lg"),
                    },
                ],
            )
            .with_default(serde_json::json!("sm")),
        );

        let argtypes = generate_storybook_argtypes(&palette);

        insta::assert_snapshot!(argtypes.as_str());
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("button"), "Button");
        assert_eq!(to_pascal_case("my-button"), "MyButton");
        assert_eq!(to_pascal_case("my_button"), "MyButton");
        assert_eq!(to_pascal_case("MyButton"), "MyButton");
    }
}
