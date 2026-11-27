//! NAPI bindings for Vue SFC formatting.

#![expect(
    clippy::disallowed_types,
    reason = "N-API values cross the boundary as std `String`s"
)]

use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use vize_glyph::{
    Allocator, ArrowParens, AttributeSortOrder, BlockSpacing, FormatOptions, FormatPreset,
    GlyphFormatter, ObjectWrap, QuoteProps, TrailingComma, VueVersion, resolve_sort_imports,
};

/// Existing formatter line-ending modes, validated at the Node boundary.
#[napi(string_enum = "lowercase")]
pub enum EndOfLineNapi {
    Lf,
    Crlf,
    Cr,
    Auto,
}

/// Format options for NAPI.
#[napi(object)]
#[derive(Default)]
pub struct FormatOptionsNapi {
    /// Existing formatter defaults; explicit options below take precedence.
    #[napi(ts_type = "'vize' | 'prettier' | 'compact' | 'documentation'")]
    pub preset: Option<String>,
    /// Explicit Vue version; omitted uses Vue 3.
    pub vue_version: Option<String>,
    /// Oxfmt-compatible import sorting; false disables it.
    #[napi(
        ts_type = "false | { partitionByNewline?: boolean; partitionByComment?: boolean; sortSideEffects?: boolean; order?: 'asc' | 'desc'; ignoreCase?: boolean; newlinesBetween?: boolean; internalPattern?: string[]; groups?: (string | string[] | { newlinesBetween: boolean })[]; customGroups?: { groupName: string; elementNamePattern?: string[]; selector?: string; modifiers?: string[] }[] }"
    )]
    pub sort_imports: Option<serde_json::Value>,
    pub print_width: Option<u32>,
    pub tab_width: Option<u8>,
    pub use_tabs: Option<bool>,
    pub semi: Option<bool>,
    pub single_quote: Option<bool>,
    /// Use single quotes in JSX attributes independently of JavaScript strings.
    pub jsx_single_quote: Option<bool>,
    /// Change when object properties are quoted.
    #[napi(ts_type = "'as-needed' | 'consistent' | 'preserve'")]
    pub quote_props: Option<String>,
    /// Keep objects multiline when a newline follows '{', or collapse objects that fit.
    #[napi(ts_type = "'preserve' | 'collapse'")]
    pub object_wrap: Option<String>,
    /// Change where trailing commas are printed.
    #[napi(ts_type = "'all' | 'es5' | 'none'")]
    pub trailing_comma: Option<String>,
    /// Change parentheses around a sole arrow function parameter.
    #[napi(ts_type = "'always' | 'avoid'")]
    pub arrow_parens: Option<String>,
    /// Print spaces inside object literal braces.
    pub bracket_spacing: Option<bool>,
    /// Put the closing bracket of a multi-line tag on its last attribute line.
    pub bracket_same_line: Option<bool>,
    /// Layout line endings; auto follows the first source terminator.
    pub end_of_line: Option<EndOfLineNapi>,
    pub sort_attributes: Option<bool>,
    /// Choose the order of attributes within each Vue priority group.
    #[napi(ts_type = "'alphabetical' | 'as-written'")]
    pub attribute_sort_order: Option<String>,
    /// Custom attribute priority groups within order-safe static attribute runs.
    pub attribute_groups: Option<Vec<Vec<String>>>,
    pub single_attribute_per_line: Option<bool>,
    /// Indent the contents of `<script>` and `<style>` blocks.
    pub vue_indent_script_and_style: Option<bool>,
    pub max_attributes_per_line: Option<u32>,
    pub normalize_directive_shorthands: Option<bool>,
    /// Order SFC blocks canonically; false preserves their authored order.
    pub sort_blocks: Option<bool>,
    /// Write one blank line between SFC blocks, or preserve the authored spacing.
    #[napi(ts_type = "'blank-line' | 'preserve'")]
    pub block_spacing: Option<String>,
    /// Re-quote CSS, SCSS and Less strings with the singleQuote preference.
    pub normalize_css_quotes: Option<bool>,
    /// Normalize one-line static style attributes.
    pub normalize_inline_styles: Option<bool>,
    /// Parenthesize v-for aliases when more than one alias is present.
    pub normalize_v_for_parens: Option<bool>,
    /// Close HTML void elements with `/>`.
    pub self_close_void_elements: Option<bool>,
    /// Re-indent SCSS and Less blocks by nesting depth.
    pub reindent_preprocessor_styles: Option<bool>,
}

/// Format result for NAPI.
#[napi(object)]
pub struct FormatResultNapi {
    pub code: String,
    pub changed: bool,
}

fn apply_options(opts: FormatOptionsNapi) -> Result<FormatOptions> {
    let mut options = FormatOptions::default();
    apply_overrides(&mut options, opts)?;
    Ok(options)
}

/// The Vue version an options object selects, if it names one.
pub(super) fn vue_version_override(opts: &FormatOptionsNapi) -> Result<Option<VueVersion>> {
    opts.vue_version
        .as_deref()
        .map(|version| {
            VueVersion::from_config_str(version)
                .map_err(|error| Error::new(Status::InvalidArg, error.to_string()))
        })
        .transpose()
}

/// The import sorting an options object selects, if it names one.
pub(super) fn sort_imports_override(
    opts: &FormatOptionsNapi,
) -> Result<Option<Option<vize_glyph::ImportSortOptions>>> {
    let Some(value) = opts.sort_imports.as_ref() else {
        return Ok(None);
    };
    let setting = serde_json::from_value::<vize_l0::config::SortImportsSetting>(value.clone())
        .map_err(|error| Error::new(Status::InvalidArg, error.to_string()))?;
    resolve_sort_imports(Some(&setting))
        .map(Some)
        .map_err(|error| Error::new(Status::InvalidArg, error.to_string()))
}

/// Apply an options object on top of `options`: a preset first, then every
/// option it names, as command-line flags apply over a project config.
pub(super) fn apply_overrides(options: &mut FormatOptions, opts: FormatOptionsNapi) -> Result<()> {
    if let Some(preset) = opts.preset.as_deref() {
        options.apply_preset(match preset {
            "vize" => FormatPreset::Vize,
            "prettier" => FormatPreset::Prettier,
            "compact" => FormatPreset::Compact,
            "documentation" => FormatPreset::Documentation,
            _ => {
                return Err(Error::new(
                    Status::InvalidArg,
                    "Unknown formatter preset; expected 'vize', 'prettier', 'compact', or 'documentation'",
                ));
            }
        });
    }

    if let Some(value) = opts.print_width {
        options.print_width = value;
    }
    if let Some(value) = opts.tab_width {
        options.tab_width = value;
    }
    if let Some(value) = opts.use_tabs {
        options.use_tabs = value;
    }
    if let Some(value) = opts.semi {
        options.semi = value;
    }
    if let Some(value) = opts.single_quote {
        options.single_quote = value;
    }
    if let Some(value) = opts.jsx_single_quote {
        options.jsx_single_quote = value;
    }
    if let Some(value) = opts.quote_props.as_deref() {
        options.quote_props = match value {
            "as-needed" => QuoteProps::AsNeeded,
            "consistent" => QuoteProps::Consistent,
            "preserve" => QuoteProps::Preserve,
            _ => {
                return Err(Error::new(
                    Status::InvalidArg,
                    "Unknown quoteProps; expected 'as-needed', 'consistent', or 'preserve'",
                ));
            }
        };
    }
    if let Some(value) = opts.object_wrap.as_deref() {
        options.object_wrap = match value {
            "preserve" => ObjectWrap::Preserve,
            "collapse" => ObjectWrap::Collapse,
            _ => {
                return Err(Error::new(
                    Status::InvalidArg,
                    "Unknown objectWrap; expected 'preserve' or 'collapse'",
                ));
            }
        };
    }
    if let Some(value) = opts.trailing_comma.as_deref() {
        options.trailing_comma = match value {
            "all" => TrailingComma::All,
            "es5" => TrailingComma::Es5,
            "none" => TrailingComma::None,
            _ => {
                return Err(Error::new(
                    Status::InvalidArg,
                    "Unknown trailingComma; expected 'all', 'es5', or 'none'",
                ));
            }
        };
    }
    if let Some(value) = opts.arrow_parens.as_deref() {
        options.arrow_parens = match value {
            "always" => ArrowParens::Always,
            "avoid" => ArrowParens::Avoid,
            _ => {
                return Err(Error::new(
                    Status::InvalidArg,
                    "Unknown arrowParens; expected 'always' or 'avoid'",
                ));
            }
        };
    }
    if let Some(value) = opts.bracket_spacing {
        options.bracket_spacing = value;
    }
    if let Some(value) = opts.bracket_same_line {
        options.bracket_same_line = value;
    }
    if let Some(value) = opts.end_of_line {
        options.end_of_line = match value {
            EndOfLineNapi::Lf => vize_glyph::EndOfLine::Lf,
            EndOfLineNapi::Crlf => vize_glyph::EndOfLine::Crlf,
            EndOfLineNapi::Cr => vize_glyph::EndOfLine::Cr,
            EndOfLineNapi::Auto => vize_glyph::EndOfLine::Auto,
        };
    }
    if let Some(value) = opts.sort_attributes {
        options.sort_attributes = value;
    }
    if let Some(value) = opts.attribute_sort_order.as_deref() {
        options.attribute_sort_order = match value {
            "alphabetical" => AttributeSortOrder::Alphabetical,
            "as-written" => AttributeSortOrder::AsWritten,
            _ => {
                return Err(Error::new(
                    Status::InvalidArg,
                    "Unknown attributeSortOrder; expected 'alphabetical' or 'as-written'",
                ));
            }
        };
    }
    if let Some(groups) = opts.attribute_groups {
        options.attribute_groups = Some(
            groups
                .into_iter()
                .map(|group| group.into_iter().map(Into::into).collect())
                .collect(),
        );
    }
    if let Some(value) = opts.single_attribute_per_line {
        options.single_attribute_per_line = value;
    }
    if let Some(value) = opts.vue_indent_script_and_style {
        options.vue_indent_script_and_style = value;
    }
    if let Some(value) = opts.max_attributes_per_line {
        options.max_attributes_per_line = Some(value);
    }
    if let Some(value) = opts.normalize_directive_shorthands {
        options.normalize_directive_shorthands = value;
    }
    if let Some(value) = opts.sort_blocks {
        options.sort_blocks = value;
    }
    if let Some(value) = opts.block_spacing.as_deref() {
        options.block_spacing = match value {
            "blank-line" => BlockSpacing::BlankLine,
            "preserve" => BlockSpacing::Preserve,
            _ => {
                return Err(Error::new(
                    Status::InvalidArg,
                    "Unknown blockSpacing; expected 'blank-line' or 'preserve'",
                ));
            }
        };
    }
    if let Some(value) = opts.normalize_css_quotes {
        options.normalize_css_quotes = value;
    }
    if let Some(value) = opts.normalize_inline_styles {
        options.normalize_inline_styles = value;
    }
    if let Some(value) = opts.normalize_v_for_parens {
        options.normalize_v_for_parens = value;
    }
    if let Some(value) = opts.self_close_void_elements {
        options.self_close_void_elements = value;
    }
    if let Some(value) = opts.reindent_preprocessor_styles {
        options.reindent_preprocessor_styles = value;
    }

    Ok(())
}

/// Format a Vue SFC source string.
#[napi(js_name = "formatSfc")]
pub fn format_sfc_napi(
    source: String,
    options: Option<FormatOptionsNapi>,
) -> Result<FormatResultNapi> {
    let options = options.unwrap_or_default();
    let vue_version = vue_version_override(&options)?.unwrap_or_default();
    let sorting = sort_imports_override(&options)?.flatten();
    let options = apply_options(options)?;
    let allocator = Allocator::with_capacity(source.len() * 2);
    let result = GlyphFormatter::new_with_vue_version(&options, &allocator, vue_version)
        .with_sort_imports(sorting.as_ref())
        .format(&source)
        .map_err(|error| Error::new(Status::GenericFailure, error.to_string()))?;

    Ok(FormatResultNapi {
        code: result.code.into(),
        changed: result.changed,
    })
}

#[cfg(test)]
mod tests;
