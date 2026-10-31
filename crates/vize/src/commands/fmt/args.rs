//! Direct formatter command arguments and typed option values.

use super::options::FmtAttributeSortOrder;
use super::patterns::FORMAT_PATTERN_HELP;
use super::{FmtArrowParens, FmtEndOfLine, FmtPreset, FmtQuoteProps, FmtTrailingComma};
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
#[expect(clippy::disallowed_types, reason = "dependency API uses std String")]
pub struct FmtArgs {
    /// Vue/JS/TS/JSON/JSONC globs; YAML/Markdown formatting is not implemented.
    #[arg(help = FORMAT_PATTERN_HELP)]
    pub patterns: Vec<String>,

    /// Check formatting without writing (exit with error if files need formatting)
    #[arg(long)]
    pub check: bool,

    /// Write formatted output to files
    #[arg(short, long)]
    pub write: bool,

    /// Config file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Do not load a config file
    #[arg(long)]
    pub no_config: bool,

    /// Use single quotes instead of double quotes
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub single_quote: Option<bool>,

    /// Use single quotes in JSX attributes
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub jsx_single_quote: Option<bool>,

    /// Change when object properties are quoted
    #[arg(long, value_enum)]
    pub quote_props: Option<FmtQuoteProps>,

    /// Change where trailing commas are printed
    #[arg(long, value_enum)]
    pub trailing_comma: Option<FmtTrailingComma>,

    /// Change parentheses around a sole arrow function parameter
    #[arg(long, value_enum)]
    pub arrow_parens: Option<FmtArrowParens>,

    /// Print spaces inside object literal braces
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub bracket_spacing: Option<bool>,

    /// Put the closing bracket of a multi-line tag on its last attribute line
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub bracket_same_line: Option<bool>,

    /// Choose the line ending written by the formatter
    #[arg(long, value_enum)]
    pub end_of_line: Option<FmtEndOfLine>,

    /// Print width (line length) for formatting
    #[arg(long)]
    pub print_width: Option<u32>,

    /// Number of spaces per indentation level
    #[arg(long)]
    pub tab_width: Option<u8>,

    /// Use tabs instead of spaces for indentation
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub use_tabs: Option<bool>,

    /// Print semicolons at the ends of statements
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true, conflicts_with = "no_semi")]
    pub semi: Option<bool>,

    /// Do not print semicolons at the ends of statements
    #[arg(long, conflicts_with = "semi")]
    pub no_semi: bool,

    /// Sort HTML attributes in template
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub sort_attributes: Option<bool>,

    /// Choose the order of attributes within each Vue priority group
    #[arg(long, value_enum)]
    pub(in crate::commands) attribute_sort_order: Option<FmtAttributeSortOrder>,

    /// Put each HTML attribute on its own line
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub single_attribute_per_line: Option<bool>,

    /// Indent script and style block contents
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub vue_indent_script_and_style: Option<bool>,

    /// Maximum number of attributes per line before wrapping
    #[arg(long)]
    pub max_attributes_per_line: Option<u32>,
    /// Normalize directive shorthands (v-bind: → :, v-on: → @, v-slot: → #)
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub normalize_directive_shorthands: Option<bool>,
    /// Order SFC blocks canonically instead of preserving their authored order
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub sort_blocks: Option<bool>,
    /// Close HTML void elements such as `<img>` with `/>`
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub self_close_void_elements: Option<bool>,
    /// Normalize CSS, SCSS and Less string quotes using the string quote preference
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub normalize_css_quotes: Option<bool>,
    /// Normalize plain static style attributes as one-line CSS declarations
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub normalize_inline_styles: Option<bool>,
    /// Parenthesize multiple v-for aliases
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub normalize_v_for_parens: Option<bool>,
    /// Re-indent SCSS and Less block contents by nesting depth
    #[arg(long, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub reindent_preprocessor_styles: Option<bool>,
    /// Apply a named set of defaults before the flags above
    #[arg(long, value_enum)]
    pub preset: Option<FmtPreset>,

    /// Show detailed timing profile
    #[arg(long)]
    pub profile: bool,

    /// Slow file threshold in milliseconds for profile output
    #[arg(long, default_value = "100")]
    pub slow_threshold: u64,
}
