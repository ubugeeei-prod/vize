//! Options for `vue/v-bind-style`.
//!
//! The style string is accepted as before. eslint-plugin-vue's tuple is
//! accepted as written: `["shorthand", { sameNameShorthand: "always" }]`.

use std::fmt;

use serde::de::{Error, IgnoredAny, IntoDeserializer, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use super::ConfigLintRuleOptions;

/// Preferred spelling for Vue `v-bind` directives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VBindStyle {
    /// Prefer `:class`.
    Shorthand,
    /// Prefer `v-bind:class`.
    Longform,
}

/// Policy for a binding whose value reads its own argument (`:foo="foo"`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VBindSameNameShorthand {
    /// Require the Vue 3.4 shorthand `:foo`.
    Always,
    /// Require the written value `:foo="foo"`.
    Never,
    /// Check neither spelling.
    #[default]
    Ignore,
}

/// The second item of eslint-plugin-vue's option tuple.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct VBindStyleOptions {
    /// Same-name binding policy. Absent means `ignore`.
    pub same_name_shorthand: VBindSameNameShorthand,
}

/// `linter.ruleOptions["vue/v-bind-style"]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum VBindStyleConfig {
    /// The style alone: `"shorthand"`.
    Style(VBindStyle),
    /// The tuple with its options: `["shorthand", { sameNameShorthand }]`.
    Tuple(VBindStyle, VBindStyleOptions),
    /// The tuple without options: `["shorthand"]`.
    StyleTuple((VBindStyle,)),
}

/// A list is the tuple and only an object is its options; a derived struct
/// would also read a list such as `["always"]` positionally.
///
/// The value is read in place rather than buffered first, so an error names
/// the position of the authored value, as the derived style enum always did.
impl<'de> Deserialize<'de> for VBindStyleConfig {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(VBindStyleConfigVisitor)
    }
}

struct VBindStyleConfigVisitor;

impl<'de> Visitor<'de> for VBindStyleConfigVisitor {
    type Value = VBindStyleConfig;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a v-bind style or [style, { sameNameShorthand }]")
    }

    fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
        VBindStyle::deserialize(value.into_deserializer()).map(VBindStyleConfig::Style)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut items: A) -> Result<Self::Value, A::Error> {
        let Some(style) = items.next_element::<VBindStyle>()? else {
            return Err(A::Error::custom(
                "a style list must be [style] or [style, { sameNameShorthand }]",
            ));
        };
        let options = items.next_element::<serde_json::Value>()?;
        if items.next_element::<IgnoredAny>()?.is_some() {
            return Err(A::Error::custom(
                "a style list must be [style] or [style, { sameNameShorthand }]",
            ));
        }
        match options {
            None => Ok(VBindStyleConfig::StyleTuple((style,))),
            Some(options) if options.is_object() => VBindStyleOptions::deserialize(options)
                .map(|options| VBindStyleConfig::Tuple(style, options))
                .map_err(A::Error::custom),
            Some(_) => Err(A::Error::custom(
                "the second item of a style list must be an options object",
            )),
        }
    }
}

impl VBindStyleConfig {
    /// The directive spelling.
    #[inline]
    pub const fn style(self) -> VBindStyle {
        match self {
            Self::Style(style) | Self::Tuple(style, _) | Self::StyleTuple((style,)) => style,
        }
    }

    /// The same-name binding policy; `ignore` unless the tuple sets it.
    #[inline]
    pub const fn same_name_shorthand(self) -> VBindSameNameShorthand {
        match self {
            Self::Tuple(_, options) => options.same_name_shorthand,
            Self::Style(_) | Self::StyleTuple(_) => VBindSameNameShorthand::Ignore,
        }
    }
}

impl ConfigLintRuleOptions {
    /// Configured spelling for `vue/v-bind-style`.
    #[inline]
    pub fn v_bind_style(&self) -> Option<VBindStyle> {
        self.v_bind_style.map(VBindStyleConfig::style)
    }

    /// Configured spelling and same-name policy for `vue/v-bind-style`.
    #[inline]
    pub fn v_bind_style_config(&self) -> Option<VBindStyleConfig> {
        self.v_bind_style
    }
}

#[cfg(test)]
mod tests {
    use super::super::ConfigLintRuleOptions;
    use super::{VBindSameNameShorthand, VBindStyle, VBindStyleConfig, VBindStyleOptions};

    fn parse(json: &str) -> Result<ConfigLintRuleOptions, serde_json::Error> {
        serde_json::from_str(json)
    }

    #[test]
    fn the_style_string_and_the_upstream_tuple_are_read() {
        for (json, style, same_name) in [
            (
                r#"{"vue/v-bind-style":"longform"}"#,
                VBindStyle::Longform,
                VBindSameNameShorthand::Ignore,
            ),
            (
                r#"{"vue/v-bind-style":["shorthand"]}"#,
                VBindStyle::Shorthand,
                VBindSameNameShorthand::Ignore,
            ),
            (
                r#"{"vue/v-bind-style":["shorthand",{}]}"#,
                VBindStyle::Shorthand,
                VBindSameNameShorthand::Ignore,
            ),
            (
                r#"{"vue/v-bind-style":["shorthand",{"sameNameShorthand":"always"}]}"#,
                VBindStyle::Shorthand,
                VBindSameNameShorthand::Always,
            ),
            (
                r#"{"vue/v-bind-style":["longform",{"sameNameShorthand":"never"}]}"#,
                VBindStyle::Longform,
                VBindSameNameShorthand::Never,
            ),
            (
                r#"{"vue/v-bind-style":["shorthand",{"sameNameShorthand":"ignore"}]}"#,
                VBindStyle::Shorthand,
                VBindSameNameShorthand::Ignore,
            ),
        ] {
            let options = parse(json).unwrap();
            let config = options.v_bind_style_config().unwrap();
            assert_eq!(config.style(), style, "{json}");
            assert_eq!(config.same_name_shorthand(), same_name, "{json}");
            assert_eq!(options.v_bind_style(), Some(style), "{json}");
        }
    }

    #[test]
    fn every_accepted_shape_serializes_back_to_itself() {
        for json in [
            r#""longform""#,
            r#"["shorthand"]"#,
            r#"["shorthand",{"sameNameShorthand":"always"}]"#,
        ] {
            let config: VBindStyleConfig = serde_json::from_str(json).unwrap();
            assert_eq!(serde_json::to_string(&config).unwrap(), json);
        }
        assert_eq!(
            serde_json::from_str::<VBindStyleConfig>(r#"["longform",{}]"#).unwrap(),
            VBindStyleConfig::Tuple(VBindStyle::Longform, VBindStyleOptions::default())
        );
    }

    #[test]
    fn a_refused_style_is_located_at_its_authored_value() {
        // `"invalid"` ends at column 29; the closing brace is column 30.
        assert_eq!(
            parse(r#"{"vue/v-bind-style":"invalid"}"#)
                .unwrap_err()
                .to_string(),
            "unknown variant `invalid`, expected `shorthand` or `longform` at line 1 column 29"
        );
        assert_eq!(
            parse(r#"{"vue/v-bind-style":["invalid"]}"#)
                .unwrap_err()
                .to_string(),
            "unknown variant `invalid`, expected `shorthand` or `longform` at line 1 column 30"
        );
    }

    #[test]
    fn a_later_layer_replaces_the_whole_option() {
        let mut options =
            parse(r#"{"vue/v-bind-style":["shorthand",{"sameNameShorthand":"always"}]}"#).unwrap();
        options.merge_from(&parse(r#"{"vue/v-bind-style":"longform"}"#).unwrap());
        assert_eq!(
            options.v_bind_style_config(),
            Some(VBindStyleConfig::Style(VBindStyle::Longform))
        );
        options.merge_from(&ConfigLintRuleOptions::default());
        assert_eq!(options.v_bind_style(), Some(VBindStyle::Longform));
    }

    #[test]
    fn shapes_outside_the_string_and_the_tuple_are_refused() {
        for json in [
            r#"{"vue/v-bind-style":"sometimes"}"#,
            r#"{"vue/v-bind-style":[]}"#,
            r#"{"vue/v-bind-style":["sometimes"]}"#,
            r#"{"vue/v-bind-style":[{"sameNameShorthand":"always"}]}"#,
            r#"{"vue/v-bind-style":["shorthand","longform"]}"#,
            r#"{"vue/v-bind-style":["shorthand",["always"]]}"#,
            r#"{"vue/v-bind-style":["shorthand",null]}"#,
            r#"{"vue/v-bind-style":["shorthand",{"sameNameShorthand":"sometimes"}]}"#,
            r#"{"vue/v-bind-style":["shorthand",{"sameName":"always"}]}"#,
            r#"{"vue/v-bind-style":["shorthand",{"sameNameShorthand":"always"},{}]}"#,
            r#"{"vue/v-bind-style":{"sameNameShorthand":"always"}}"#,
        ] {
            assert!(parse(json).is_err(), "accepted invalid option: {json}");
        }
        for (value, reason) in [
            (
                serde_json::json!(["shorthand", { "sameName": "always" }]),
                "unknown field `sameName`, expected `sameNameShorthand`",
            ),
            (
                serde_json::json!(["shorthand", ["always"]]),
                "the second item of a style list must be an options object",
            ),
            (
                serde_json::json!([]),
                "a style list must be [style] or [style, { sameNameShorthand }]",
            ),
        ] {
            assert_eq!(
                serde_json::from_value::<VBindStyleConfig>(value)
                    .unwrap_err()
                    .to_string(),
                reason
            );
        }
    }
}
