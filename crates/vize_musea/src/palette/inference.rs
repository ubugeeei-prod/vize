//! Control type inference from prop values.
//!
//! This module infers the appropriate control type based on:
//! - Value type (string, number, boolean, etc.)
//! - Value patterns (color codes, dates, etc.)
//! - Multiple values across variants (for select inference)

use super::{ControlKind, PaletteOptions, RangeConfig, SelectOption};
use vize_carton::{FxHashSet, String};

/// Infer control type from a single value.
#[inline]
pub fn infer_control_type(value: &serde_json::Value) -> ControlKind {
    match value {
        serde_json::Value::Bool(_) => ControlKind::Boolean,
        serde_json::Value::Number(_) => ControlKind::Number,
        serde_json::Value::String(s) => infer_string_control(s),
        serde_json::Value::Array(_) => ControlKind::Array,
        serde_json::Value::Object(_) => ControlKind::Object,
        serde_json::Value::Null => ControlKind::Text,
    }
}

/// Infer control type from string value patterns.
#[inline]
fn infer_string_control(s: &str) -> ControlKind {
    // Check for color patterns
    if is_color_value(s) {
        return ControlKind::Color;
    }

    // Check for date patterns
    if is_date_value(s) {
        return ControlKind::Date;
    }

    ControlKind::Text
}

/// Check if string looks like a color value.
#[inline]
fn is_color_value(s: &str) -> bool {
    // Hex colors: #RGB, #RRGGBB, #RRGGBBAA
    if let Some(hex) = s.strip_prefix('#') {
        let len = hex.len();
        return (len == 3 || len == 4 || len == 6 || len == 8)
            && hex.chars().all(|c| c.is_ascii_hexdigit());
    }

    // RGB/RGBA/HSL/HSLA functions (case-insensitive prefix check)
    {
        let s_bytes = s.as_bytes();
        let starts_with_ci = |prefix: &[u8]| {
            s_bytes.len() >= prefix.len()
                && s_bytes[..prefix.len()]
                    .iter()
                    .zip(prefix)
                    .all(|(a, b)| a.to_ascii_lowercase() == *b)
        };
        if starts_with_ci(b"rgb(")
            || starts_with_ci(b"rgba(")
            || starts_with_ci(b"hsl(")
            || starts_with_ci(b"hsla(")
        {
            return true;
        }
    }

    // Named colors (common ones)
    s.eq_ignore_ascii_case("red")
        || s.eq_ignore_ascii_case("green")
        || s.eq_ignore_ascii_case("blue")
        || s.eq_ignore_ascii_case("white")
        || s.eq_ignore_ascii_case("black")
        || s.eq_ignore_ascii_case("yellow")
        || s.eq_ignore_ascii_case("orange")
        || s.eq_ignore_ascii_case("purple")
        || s.eq_ignore_ascii_case("pink")
        || s.eq_ignore_ascii_case("gray")
        || s.eq_ignore_ascii_case("grey")
        || s.eq_ignore_ascii_case("cyan")
        || s.eq_ignore_ascii_case("magenta")
        || s.eq_ignore_ascii_case("transparent")
}

/// Check if string looks like a date value.
#[inline]
fn is_date_value(s: &str) -> bool {
    // ISO date format: YYYY-MM-DD
    if s.len() == 10 {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() == 3 {
            return parts[0].len() == 4
                && parts[1].len() == 2
                && parts[2].len() == 2
                && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()));
        }
    }

    // ISO datetime: YYYY-MM-DDTHH:MM:SS
    if s.len() >= 19 && s.contains('T') {
        let parts: Vec<&str> = s.split('T').collect();
        if parts.len() == 2 {
            return is_date_value(parts[0]);
        }
    }

    false
}

/// Infer control type from multiple values (across variants).
pub fn infer_control_from_values(
    values: &[serde_json::Value],
    options: &PaletteOptions,
) -> (ControlKind, Vec<SelectOption>, Option<RangeConfig>) {
    if values.is_empty() {
        return (ControlKind::Text, Vec::new(), None);
    }

    // Determine base type from first non-null value
    let base_type = values
        .iter()
        .find(|v| !v.is_null())
        .map(infer_control_type)
        .unwrap_or(ControlKind::Text);

    // For strings, check if we should create a select
    if base_type == ControlKind::Text && options.infer_options {
        let unique_values = collect_unique_strings(values);
        let count = unique_values.len();

        if count >= options.min_select_values && count <= options.max_select_values {
            let select_options: Vec<SelectOption> = unique_values
                .into_iter()
                .map(|v| SelectOption {
                    label: humanize_label(&v),
                    value: serde_json::json!(v),
                })
                .collect();

            return (ControlKind::Select, select_options, None);
        }
    }

    // For numbers, check if we should create a range
    if base_type == ControlKind::Number {
        if let Some(range) = infer_number_range(values) {
            return (ControlKind::Range, Vec::new(), Some(range));
        }
    }

    (base_type, Vec::new(), None)
}

/// Collect unique string values.
#[allow(clippy::disallowed_types)]
fn collect_unique_strings(values: &[serde_json::Value]) -> Vec<String> {
    let mut seen: FxHashSet<std::string::String> = FxHashSet::default();
    let mut result = Vec::new();

    for value in values {
        if let serde_json::Value::String(s) = value {
            if seen.insert(s.clone()) {
                result.push(String::from(s.as_str()));
            }
        }
    }

    result
}

/// Try to infer a reasonable range from number values.
fn infer_number_range(values: &[serde_json::Value]) -> Option<RangeConfig> {
    let numbers: Vec<f64> = values.iter().filter_map(|v| v.as_f64()).collect();

    if numbers.len() < 2 {
        return None;
    }

    let min = numbers.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = numbers.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    // Only create range if there's meaningful variation
    if (max - min).abs() < f64::EPSILON {
        return None;
    }

    // Infer step from differences
    let mut diffs: Vec<f64> = numbers
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .filter(|d| *d > f64::EPSILON)
        .collect();
    diffs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let step = diffs.first().copied();

    // Extend range slightly for flexibility
    let range_extend = (max - min) * 0.1;

    Some(RangeConfig {
        min: (min - range_extend).floor(),
        max: (max + range_extend).ceil(),
        step,
    })
}

/// Convert a value to a human-readable label.
#[inline]
fn humanize_label(s: &str) -> String {
    // Handle common patterns
    let result = s
        // camelCase to spaces
        .chars()
        .fold(String::default(), |mut acc, c| {
            if c.is_uppercase() && !acc.is_empty() {
                acc.push(' ');
            }
            acc.push(c);
            acc
        });

    // snake_case/kebab-case to spaces
    let result: String = result.replace(['_', '-'], " ").into();

    // Capitalize first letter
    let mut chars = result.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        humanize_label, infer_control_from_values, infer_control_type, ControlKind, PaletteOptions,
    };

    #[test]
    fn test_infer_boolean() {
        assert_eq!(
            infer_control_type(&serde_json::json!(true)),
            ControlKind::Boolean
        );
        assert_eq!(
            infer_control_type(&serde_json::json!(false)),
            ControlKind::Boolean
        );
    }

    #[test]
    fn test_infer_number() {
        assert_eq!(
            infer_control_type(&serde_json::json!(42)),
            ControlKind::Number
        );
        assert_eq!(
            infer_control_type(&serde_json::json!(1.5)),
            ControlKind::Number
        );
    }

    #[test]
    fn test_infer_color() {
        assert_eq!(
            infer_control_type(&serde_json::json!("#ff0000")),
            ControlKind::Color
        );
        assert_eq!(
            infer_control_type(&serde_json::json!("#FFF")),
            ControlKind::Color
        );
        assert_eq!(
            infer_control_type(&serde_json::json!("rgb(255, 0, 0)")),
            ControlKind::Color
        );
        assert_eq!(
            infer_control_type(&serde_json::json!("red")),
            ControlKind::Color
        );
    }

    #[test]
    fn test_infer_date() {
        assert_eq!(
            infer_control_type(&serde_json::json!("2024-01-15")),
            ControlKind::Date
        );
        assert_eq!(
            infer_control_type(&serde_json::json!("2024-01-15T10:30:00")),
            ControlKind::Date
        );
    }

    #[test]
    fn test_infer_select_from_values() {
        let values = vec![
            serde_json::json!("sm"),
            serde_json::json!("md"),
            serde_json::json!("lg"),
        ];

        let (kind, options, _) = infer_control_from_values(&values, &PaletteOptions::default());

        assert_eq!(kind, ControlKind::Select);
        assert_eq!(options.len(), 3);
    }

    #[test]
    fn test_infer_range_from_values() {
        let values = vec![
            serde_json::json!(10),
            serde_json::json!(20),
            serde_json::json!(30),
            serde_json::json!(40),
        ];

        let (kind, _, range) = infer_control_from_values(&values, &PaletteOptions::default());

        assert_eq!(kind, ControlKind::Range);
        assert!(range.is_some());
    }

    #[test]
    fn test_humanize_label() {
        assert_eq!(humanize_label("primaryColor"), "Primary Color");
        assert_eq!(humanize_label("font_size"), "Font size");
        assert_eq!(humanize_label("is-active"), "Is active");
        assert_eq!(humanize_label("sm"), "Sm");
    }
}
