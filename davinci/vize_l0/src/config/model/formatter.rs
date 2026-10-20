//! Formatter config model.

use serde::{Deserialize, Serialize};

use crate::String;

/// Trailing comma mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrailingComma {
    None,
    Es5,
    #[default]
    All,
}

/// Arrow parens mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArrowParens {
    #[default]
    Always,
    Avoid,
}

/// End-of-line mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EndOfLine {
    #[default]
    Lf,
    Crlf,
    Cr,
    Auto,
}

/// Object property quoting mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QuoteProps {
    #[default]
    AsNeeded,
    Consistent,
    Preserve,
}

/// Attribute ordering strategy.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttributeSortOrder {
    #[default]
    Alphabetical,
    AsWritten,
}

/// Formatter settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct FormatterConfig {
    pub print_width: u32,
    pub tab_width: u8,
    pub use_tabs: bool,
    pub semi: bool,
    pub single_quote: bool,
    pub jsx_single_quote: bool,
    pub trailing_comma: TrailingComma,
    pub bracket_spacing: bool,
    pub bracket_same_line: bool,
    pub arrow_parens: ArrowParens,
    pub end_of_line: EndOfLine,
    pub quote_props: QuoteProps,
    pub single_attribute_per_line: bool,
    pub vue_indent_script_and_style: bool,
    pub sort_attributes: bool,
    pub attribute_sort_order: AttributeSortOrder,
    pub merge_bind_and_non_bind_attrs: bool,
    pub max_attributes_per_line: Option<u32>,
    pub attribute_groups: Option<Vec<Vec<String>>>,
    pub normalize_directive_shorthands: bool,
    pub sort_blocks: bool,
}

impl FormatterConfig {
    /// Returns true when the config matches the built-in defaults.
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            print_width: 100,
            tab_width: 2,
            use_tabs: false,
            semi: true,
            single_quote: false,
            jsx_single_quote: false,
            trailing_comma: TrailingComma::All,
            bracket_spacing: true,
            bracket_same_line: false,
            arrow_parens: ArrowParens::Always,
            end_of_line: EndOfLine::Lf,
            quote_props: QuoteProps::AsNeeded,
            single_attribute_per_line: false,
            vue_indent_script_and_style: false,
            sort_attributes: true,
            attribute_sort_order: AttributeSortOrder::Alphabetical,
            merge_bind_and_non_bind_attrs: false,
            max_attributes_per_line: None,
            attribute_groups: None,
            normalize_directive_shorthands: true,
            sort_blocks: true,
        }
    }
}

/// A named assignment of formatter options (#6098).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum FormatterPreset {
    /// The Vize defaults.
    #[default]
    Vize,
    /// Prettier's Vue layout.
    Prettier,
}

impl FormatterPreset {
    /// The options this preset assigns, keyed as in config files.
    pub fn options(self) -> serde_json::Value {
        let vize = self == Self::Vize;
        let entries = [
            (
                "printWidth",
                serde_json::Value::from(if vize { 100 } else { 80 }),
            ),
            ("sortAttributes", vize.into()),
            ("sortBlocks", vize.into()),
            ("normalizeDirectiveShorthands", vize.into()),
            ("normalizeCssQuotes", (!vize).into()),
            ("normalizeInlineStyles", (!vize).into()),
            ("normalizeVForParens", (!vize).into()),
            ("selfCloseVoidElements", (!vize).into()),
            ("reindentPreprocessorStyles", (!vize).into()),
        ];
        serde_json::Value::Object(
            entries
                .into_iter()
                .map(|(key, value)| (key.into(), value))
                .collect(),
        )
    }
}

/// Formatter options as authored, after `preset`, including options outside
/// [`FormatterConfig`].
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct FormatterProfile {
    pub preset: Option<FormatterPreset>,
    /// An object keyed as in config files: the preset's options, then the
    /// authored ones. `Null` when neither is set.
    pub options: serde_json::Value,
}

/// Retain extra formatter controls without changing the published exhaustive model.
#[derive(Debug, Clone, Default)]
pub(crate) struct RawFormatterConfig {
    pub(crate) config: FormatterConfig,
    pub(crate) sort_imports: Option<super::sort_imports::SortImportsSetting>,
    pub(crate) profile: FormatterProfile,
}

/// Every formatter key, typed so the loader refuses invalid values.
#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RawFormatterKeys {
    #[serde(flatten)]
    config: FormatterConfig,
    sort_imports: Option<super::sort_imports::SortImportsSetting>,
    preset: Option<FormatterPreset>,
    normalize_css_quotes: Option<bool>,
    normalize_inline_styles: Option<bool>,
    normalize_v_for_parens: Option<bool>,
    self_close_void_elements: Option<bool>,
    reindent_preprocessor_styles: Option<bool>,
}

impl RawFormatterConfig {
    /// Whether the config wrote no formatter option.
    pub(crate) fn is_unset(&self) -> bool {
        self.profile.options.is_null()
    }
}

impl<'de> Deserialize<'de> for RawFormatterConfig {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let serde_json::Value::Object(mut authored) = serde_json::Value::deserialize(deserializer)?
        else {
            return Err(D::Error::custom("formatter must be an object"));
        };
        // Pkl writes unset options as null.
        authored.retain(|_, value| !value.is_null());
        let preset = authored
            .get("preset")
            .map(FormatterPreset::deserialize)
            .transpose()
            .map_err(D::Error::custom)?;
        let sorting_only = authored.keys().all(|key| key == "sortImports");
        let mut options = preset.map_or_else(
            || serde_json::Value::Object(serde_json::Map::new()),
            FormatterPreset::options,
        );
        if let serde_json::Value::Object(assigned) = &mut options {
            assigned.extend(authored);
        }
        let keys = RawFormatterKeys::deserialize(&options).map_err(D::Error::custom)?;
        Ok(Self {
            config: keys.config,
            sort_imports: keys.sort_imports,
            profile: FormatterProfile {
                preset,
                options: if sorting_only {
                    serde_json::Value::Null
                } else {
                    options
                },
            },
        })
    }
}
