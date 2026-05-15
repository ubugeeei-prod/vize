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
