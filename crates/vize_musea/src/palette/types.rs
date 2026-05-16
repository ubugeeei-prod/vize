//! Palette type definitions.

use serde::{Deserialize, Serialize};
use vize_carton::{FxHashMap, String};

/// Control type for a prop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ControlKind {
    /// Text input control.
    Text,
    /// Number input control.
    Number,
    /// Boolean toggle/checkbox.
    Boolean,
    /// Range slider.
    Range,
    /// Select dropdown.
    Select,
    /// Radio button group.
    Radio,
    /// Color picker.
    Color,
    /// Date picker.
    Date,
    /// Object/JSON editor.
    Object,
    /// Array editor.
    Array,
    /// File upload.
    File,
    /// Raw/uneditable display.
    Raw,
}

impl Default for ControlKind {
    #[inline]
    fn default() -> Self {
        Self::Text
    }
}

/// Range configuration for number controls.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RangeConfig {
    /// Minimum value.
    pub min: f64,
    /// Maximum value.
    pub max: f64,
    /// Step increment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
}

impl Default for RangeConfig {
    #[inline]
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 100.0,
            step: Some(1.0),
        }
    }
}

/// Option for select/radio controls.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectOption {
    /// Display label.
    pub label: String,
    /// Actual value.
    pub value: serde_json::Value,
}

/// Control definition for a single prop.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropControl {
    /// Prop name.
    pub name: String,
    /// Control type.
    pub control: ControlKind,
    /// Default value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
    /// Description/label for the control.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the prop is required.
    #[serde(default)]
    pub required: bool,
    /// Options for select/radio controls.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub options: Vec<SelectOption>,
    /// Range configuration for number/range controls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<RangeConfig>,
    /// Group/category for organizing controls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

impl PropControl {
    /// Create a new text control.
    #[inline]
    pub fn text(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            control: ControlKind::Text,
            default_value: None,
            description: None,
            required: false,
            options: Vec::new(),
            range: None,
            group: None,
        }
    }

    /// Create a new number control.
    #[inline]
    pub fn number(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            control: ControlKind::Number,
            default_value: None,
            description: None,
            required: false,
            options: Vec::new(),
            range: None,
            group: None,
        }
    }

    /// Create a new boolean control.
    #[inline]
    pub fn boolean(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            control: ControlKind::Boolean,
            default_value: None,
            description: None,
            required: false,
            options: Vec::new(),
            range: None,
            group: None,
        }
    }

    /// Create a new select control.
    #[inline]
    pub fn select(name: impl Into<String>, options: Vec<SelectOption>) -> Self {
        Self {
            name: name.into(),
            control: ControlKind::Select,
            default_value: None,
            description: None,
            required: false,
            options,
            range: None,
            group: None,
        }
    }

    /// Create a new range control.
    #[inline]
    pub fn range(name: impl Into<String>, config: RangeConfig) -> Self {
        Self {
            name: name.into(),
            control: ControlKind::Range,
            default_value: None,
            description: None,
            required: false,
            options: Vec::new(),
            range: Some(config),
            group: None,
        }
    }

    /// Set default value.
    #[inline]
    pub fn with_default(mut self, value: serde_json::Value) -> Self {
        self.default_value = Some(value);
        self
    }

    /// Set description.
    #[inline]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set as required.
    #[inline]
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set group.
    #[inline]
    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }
}

/// Palette configuration for a component.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Palette {
    /// Component title.
    pub title: String,
    /// Controls for each prop.
    pub controls: Vec<PropControl>,
    /// Control groups.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub groups: Vec<String>,
    /// Props collected from all variants.
    #[serde(skip_serializing_if = "FxHashMap::is_empty", default)]
    pub all_values: FxHashMap<String, Vec<serde_json::Value>>,
}

impl Palette {
    /// Create a new empty palette.
    #[inline]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            controls: Vec::new(),
            groups: Vec::new(),
            all_values: FxHashMap::default(),
        }
    }

    /// Add a control.
    #[inline]
    pub fn add_control(&mut self, control: PropControl) {
        if let Some(ref group) = control.group {
            if !self.groups.contains(group) {
                self.groups.push(group.clone());
            }
        }
        self.controls.push(control);
    }

    /// Get controls by group.
    pub fn controls_by_group(&self, group: Option<&str>) -> Vec<&PropControl> {
        self.controls
            .iter()
            .filter(|c| c.group.as_deref() == group)
            .collect()
    }
}

/// Options for palette generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteOptions {
    /// Infer select options from variant values.
    #[serde(default = "default_true")]
    pub infer_options: bool,
    /// Minimum unique values to create select (vs text).
    #[serde(default = "default_min_select")]
    pub min_select_values: usize,
    /// Maximum unique values to create select (vs text).
    #[serde(default = "default_max_select")]
    pub max_select_values: usize,
    /// Group controls by prop type.
    #[serde(default)]
    pub group_by_type: bool,
}

impl Default for PaletteOptions {
    #[inline]
    fn default() -> Self {
        Self {
            infer_options: true,
            min_select_values: 2,
            max_select_values: 10,
            group_by_type: false,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_min_select() -> usize {
    2
}

fn default_max_select() -> usize {
    10
}

/// Palette generation output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteOutput {
    /// Generated palette configuration.
    pub palette: Palette,
    /// JSON representation for embedding.
    pub json: String,
    /// TypeScript interface for props.
    pub typescript: String,
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::{ControlKind, Palette, PropControl, SelectOption};

    #[test]
    fn test_control_builders() {
        let text = PropControl::text("label")
            .with_default(serde_json::json!("Hello"))
            .with_description("Button label")
            .required();

        assert_eq!(text.name, "label");
        assert_eq!(text.control, ControlKind::Text);
        assert!(text.required);
        assert_eq!(text.default_value, Some(serde_json::json!("Hello")));
    }

    #[test]
    fn test_select_control() {
        let options = vec![
            SelectOption {
                label: "Small".into(),
                value: serde_json::json!("sm"),
            },
            SelectOption {
                label: "Medium".into(),
                value: serde_json::json!("md"),
            },
            SelectOption {
                label: "Large".into(),
                value: serde_json::json!("lg"),
            },
        ];

        let select = PropControl::select("size", options);
        assert_eq!(select.control, ControlKind::Select);
        assert_eq!(select.options.len(), 3);
    }

    #[test]
    fn test_palette_groups() {
        let mut palette = Palette::new("Button");
        palette.add_control(PropControl::text("label").with_group("Content"));
        palette.add_control(PropControl::text("icon").with_group("Content"));
        palette.add_control(PropControl::boolean("disabled").with_group("State"));

        assert_eq!(palette.groups, vec!["Content", "State"]);
        assert_eq!(palette.controls_by_group(Some("Content")).len(), 2);
        assert_eq!(palette.controls_by_group(Some("State")).len(), 1);
    }
}
