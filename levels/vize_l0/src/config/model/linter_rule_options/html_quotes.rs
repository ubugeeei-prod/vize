use serde::{Deserialize, Serialize};

/// Configuration accepted by `vue/html-quotes`.
///
/// A string selects the quote style. The object form also exposes upstream's
/// `avoidEscape` policy while retaining the default double-quote style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HtmlQuotesConfig {
    Style(HtmlQuoteStyle),
    Options(HtmlQuotesOptions),
}

impl HtmlQuotesConfig {
    #[inline]
    pub const fn style(self) -> HtmlQuoteStyle {
        match self {
            Self::Style(style) => style,
            Self::Options(options) => options.style,
        }
    }

    #[inline]
    pub const fn avoid_escape(self) -> bool {
        match self {
            Self::Style(_) => false,
            Self::Options(options) => options.avoid_escape,
        }
    }
}

/// Quote delimiter required for HTML attribute values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HtmlQuoteStyle {
    #[default]
    Double,
    Single,
}

/// Detailed `vue/html-quotes` options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct HtmlQuotesOptions {
    pub style: HtmlQuoteStyle,
    pub avoid_escape: bool,
}
