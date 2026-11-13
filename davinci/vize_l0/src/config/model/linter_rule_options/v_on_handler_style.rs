//! Options for `vue/v-on-handler-style`.
//!
//! eslint-plugin-vue's first option is accepted as written: `"inline"`,
//! `"inline-function"`, `["method", "inline"]` or `["method",
//! "inline-function"]`. `"method"` is Patina's default and allows only a
//! method reference. The object form adds upstream's second option,
//! `ignoreIncludesComment`.

use serde::{Deserialize, Deserializer, Serialize, de::Error};

/// One way to write a `v-on` handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VOnHandlerKind {
    /// `@click="handler"`.
    Method,
    /// `@click="handler()"`.
    Inline,
    /// `@click="() => handler()"`.
    InlineFunction,
}

/// The allowed handler styles: a preferred one, and with `"method"` first, the
/// style allowed where a method reference cannot be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "AllowedRepr", into = "AllowedRepr")]
pub struct VOnHandlerAllowed {
    preferred: VOnHandlerKind,
    fallback: Option<VOnHandlerKind>,
}

impl VOnHandlerAllowed {
    /// Patina's default: only a method reference.
    pub const METHOD: Self = Self {
        preferred: VOnHandlerKind::Method,
        fallback: None,
    };

    /// The preferred style.
    #[inline]
    pub const fn preferred(self) -> VOnHandlerKind {
        self.preferred
    }

    /// The style allowed where a method reference cannot be used.
    #[inline]
    pub const fn fallback(self) -> Option<VOnHandlerKind> {
        self.fallback
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum AllowedRepr {
    One(VOnHandlerKind),
    Pair(Vec<VOnHandlerKind>),
}

impl TryFrom<AllowedRepr> for VOnHandlerAllowed {
    type Error = &'static str;

    fn try_from(repr: AllowedRepr) -> Result<Self, Self::Error> {
        match repr {
            AllowedRepr::One(preferred) => Ok(Self {
                preferred,
                fallback: None,
            }),
            AllowedRepr::Pair(pair) => match pair.as_slice() {
                [
                    VOnHandlerKind::Method,
                    fallback @ (VOnHandlerKind::Inline | VOnHandlerKind::InlineFunction),
                ] => Ok(Self {
                    preferred: VOnHandlerKind::Method,
                    fallback: Some(*fallback),
                }),
                _ => Err(
                    "a style list must be [\"method\", \"inline\"] or [\"method\", \"inline-function\"]",
                ),
            },
        }
    }
}

impl From<VOnHandlerAllowed> for AllowedRepr {
    fn from(allowed: VOnHandlerAllowed) -> Self {
        match allowed.fallback {
            None => Self::One(allowed.preferred),
            Some(fallback) => Self::Pair(vec![allowed.preferred, fallback]),
        }
    }
}

/// The object form of `vue/v-on-handler-style`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct VOnHandlerStyleOptions {
    /// Allowed styles. Absent means `"method"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<VOnHandlerAllowed>,
    /// Leave a handler that contains a comment unreported where the preferred
    /// style is `"method"`.
    pub ignore_includes_comment: bool,
}

/// `linter.ruleOptions["vue/v-on-handler-style"]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum VOnHandlerStyleConfig {
    /// The style alone.
    Style(VOnHandlerAllowed),
    /// The style and `ignoreIncludesComment`.
    Options(VOnHandlerStyleOptions),
}

/// Only an object is the options form; a struct would also accept a list such
/// as `["method"]` positionally.
impl<'de> Deserialize<'de> for VOnHandlerStyleConfig {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        if value.is_object() {
            VOnHandlerStyleOptions::deserialize(value)
                .map(Self::Options)
                .map_err(D::Error::custom)
        } else {
            VOnHandlerAllowed::deserialize(value)
                .map(Self::Style)
                .map_err(D::Error::custom)
        }
    }
}

impl VOnHandlerStyleConfig {
    /// The allowed styles.
    #[inline]
    pub fn allowed(self) -> VOnHandlerAllowed {
        match self {
            Self::Style(allowed) => allowed,
            Self::Options(options) => options.style.unwrap_or(VOnHandlerAllowed::METHOD),
        }
    }

    /// Whether a handler that contains a comment is left alone.
    #[inline]
    pub fn ignore_includes_comment(self) -> bool {
        match self {
            Self::Style(_) => false,
            Self::Options(options) => options.ignore_includes_comment,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{VOnHandlerKind, VOnHandlerStyleConfig};

    fn parse(value: serde_json::Value) -> Result<VOnHandlerStyleConfig, serde_json::Error> {
        serde_json::from_value(value)
    }

    #[test]
    fn accepts_every_upstream_first_option_and_the_object_form() {
        let cases = [
            (
                serde_json::json!("method"),
                VOnHandlerKind::Method,
                None,
                false,
            ),
            (
                serde_json::json!("inline"),
                VOnHandlerKind::Inline,
                None,
                false,
            ),
            (
                serde_json::json!("inline-function"),
                VOnHandlerKind::InlineFunction,
                None,
                false,
            ),
            (
                serde_json::json!(["method", "inline"]),
                VOnHandlerKind::Method,
                Some(VOnHandlerKind::Inline),
                false,
            ),
            (
                serde_json::json!(["method", "inline-function"]),
                VOnHandlerKind::Method,
                Some(VOnHandlerKind::InlineFunction),
                false,
            ),
            (
                serde_json::json!({ "style": ["method", "inline-function"], "ignoreIncludesComment": true }),
                VOnHandlerKind::Method,
                Some(VOnHandlerKind::InlineFunction),
                true,
            ),
            (
                serde_json::json!({ "ignoreIncludesComment": true }),
                VOnHandlerKind::Method,
                None,
                true,
            ),
        ];
        for (value, preferred, fallback, ignore) in cases {
            let config = parse(value).unwrap();
            assert_eq!(
                (
                    config.allowed().preferred(),
                    config.allowed().fallback(),
                    config.ignore_includes_comment()
                ),
                (preferred, fallback, ignore)
            );
        }
    }

    #[test]
    fn refuses_lists_upstream_refuses() {
        for value in [
            serde_json::json!(["inline", "method"]),
            serde_json::json!(["method"]),
            serde_json::json!(["method", "method"]),
            serde_json::json!(["method", "inline", "inline-function"]),
            serde_json::json!("handler"),
            serde_json::json!({ "style": "inline", "other": true }),
        ] {
            assert_eq!(parse(value.clone()).ok(), None, "{value}");
        }
    }

    #[test]
    fn round_trips_through_serialization() {
        let config = parse(serde_json::json!(["method", "inline"])).unwrap();
        assert_eq!(
            serde_json::to_value(config).unwrap(),
            serde_json::json!(["method", "inline"])
        );
    }
}
