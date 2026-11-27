//! NAPI bindings for Vue SFC formatting.

#![expect(
    clippy::disallowed_types,
    reason = "N-API values cross the boundary as std `String`s"
)]

use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use vize_glyph::{Allocator, FormatOptions, GlyphFormatter, VueVersion, resolve_sort_imports};

mod overrides;
pub(super) use overrides::apply_overrides;

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
mod tests {
    use super::{FormatOptionsNapi, apply_options, format_sfc_napi};
    use vize_glyph::{ArrowParens, AttributeSortOrder, QuoteProps, TrailingComma};

    #[test]
    fn native_sfc_sorting_uses_the_authored_full_reference() {
        let source = include_str!("../../../vize_glyph/tests/fixtures/sort-imports/UserCard.vue");
        let expected =
            include_str!("../../../vize_glyph/tests/fixtures/sort-imports/UserCard.sorted.vue");
        let output = format_sfc_napi(
            source.into(),
            Some(FormatOptionsNapi {
                sort_imports: Some(serde_json::json!({})),
                ..Default::default()
            }),
        )
        .expect("native formatting");
        assert_eq!(output.code, expected);
        assert!(output.changed);
    }

    #[test]
    fn native_normalization_options_override_presets_in_both_directions() {
        let enabled = apply_options(FormatOptionsNapi {
            normalize_css_quotes: Some(true),
            normalize_inline_styles: Some(true),
            normalize_v_for_parens: Some(true),
            self_close_void_elements: Some(true),
            reindent_preprocessor_styles: Some(true),
            ..Default::default()
        })
        .expect("native options");
        assert!(enabled.normalize_css_quotes);
        assert!(enabled.normalize_inline_styles);
        assert!(enabled.normalize_v_for_parens);
        assert!(enabled.self_close_void_elements);
        assert!(enabled.reindent_preprocessor_styles);

        let disabled = apply_options(FormatOptionsNapi {
            preset: Some("prettier".into()),
            normalize_css_quotes: Some(false),
            normalize_inline_styles: Some(false),
            normalize_v_for_parens: Some(false),
            self_close_void_elements: Some(false),
            reindent_preprocessor_styles: Some(false),
            ..Default::default()
        })
        .expect("native preset override");
        assert!(!disabled.normalize_css_quotes);
        assert!(!disabled.normalize_inline_styles);
        assert!(!disabled.normalize_v_for_parens);
        assert!(!disabled.self_close_void_elements);
        assert!(!disabled.reindent_preprocessor_styles);
    }

    #[test]
    fn native_sfc_body_indentation_is_an_explicit_opt_in() {
        let defaults = apply_options(FormatOptionsNapi::default()).expect("native defaults");
        assert!(!defaults.vue_indent_script_and_style);

        let indented = apply_options(FormatOptionsNapi {
            preset: Some("vize".into()),
            vue_indent_script_and_style: Some(true),
            ..Default::default()
        })
        .expect("native indentation option");
        assert!(indented.vue_indent_script_and_style);
    }

    #[test]
    fn native_attribute_order_is_typed_and_overrides_presets() {
        let options = apply_options(FormatOptionsNapi {
            preset: Some("vize".into()),
            attribute_sort_order: Some("as-written".into()),
            ..Default::default()
        })
        .expect("native attribute order option");
        assert_eq!(options.attribute_sort_order, AttributeSortOrder::AsWritten);

        let error = apply_options(FormatOptionsNapi {
            attribute_sort_order: Some("source".into()),
            ..Default::default()
        })
        .expect_err("unknown attribute order policy");
        assert_eq!(error.status, napi::Status::InvalidArg);
        assert_eq!(
            error.reason,
            "Unknown attributeSortOrder; expected 'alphabetical' or 'as-written'"
        );
    }

    #[test]
    fn native_script_quote_options_are_typed_and_independent() {
        let options = apply_options(FormatOptionsNapi {
            single_quote: Some(false),
            jsx_single_quote: Some(true),
            quote_props: Some("preserve".into()),
            ..Default::default()
        })
        .expect("native script quote options");
        assert!(!options.single_quote);
        assert!(options.jsx_single_quote);
        assert_eq!(options.quote_props, QuoteProps::Preserve);

        let error = apply_options(FormatOptionsNapi {
            quote_props: Some("keep".into()),
            ..Default::default()
        })
        .expect_err("unknown quote policy");
        assert_eq!(error.status, napi::Status::InvalidArg);
        assert_eq!(
            error.reason,
            "Unknown quoteProps; expected 'as-needed', 'consistent', or 'preserve'"
        );
    }

    #[test]
    fn native_trailing_comma_option_is_typed_and_overrides_presets() {
        let options = apply_options(FormatOptionsNapi {
            preset: Some("prettier".into()),
            trailing_comma: Some("none".into()),
            ..Default::default()
        })
        .expect("native trailing comma option");
        assert_eq!(options.trailing_comma, TrailingComma::None);

        let error = apply_options(FormatOptionsNapi {
            trailing_comma: Some("never".into()),
            ..Default::default()
        })
        .expect_err("unknown trailing comma policy");
        assert_eq!(error.status, napi::Status::InvalidArg);
        assert_eq!(
            error.reason,
            "Unknown trailingComma; expected 'all', 'es5', or 'none'"
        );
    }

    #[test]
    fn native_arrow_parens_option_is_typed_and_overrides_presets() {
        let options = apply_options(FormatOptionsNapi {
            preset: Some("vize".into()),
            arrow_parens: Some("avoid".into()),
            ..Default::default()
        })
        .expect("native arrow parentheses option");
        assert_eq!(options.arrow_parens, ArrowParens::Avoid);

        let error = apply_options(FormatOptionsNapi {
            arrow_parens: Some("preserve".into()),
            ..Default::default()
        })
        .expect_err("unknown arrow parentheses policy");
        assert_eq!(error.status, napi::Status::InvalidArg);
        assert_eq!(
            error.reason,
            "Unknown arrowParens; expected 'always' or 'avoid'"
        );
    }

    #[test]
    fn native_bracket_layout_options_override_presets() {
        let options = apply_options(FormatOptionsNapi {
            preset: Some("compact".into()),
            bracket_spacing: Some(false),
            bracket_same_line: Some(false),
            ..Default::default()
        })
        .expect("native bracket layout options");
        assert!(!options.bracket_spacing);
        assert!(!options.bracket_same_line);
    }

    #[test]
    fn native_block_order_option_overrides_presets() {
        let preserve = apply_options(FormatOptionsNapi {
            preset: Some("vize".into()),
            sort_blocks: Some(false),
            ..Default::default()
        })
        .expect("native authored block order");
        assert!(!preserve.sort_blocks);

        let canonical = apply_options(FormatOptionsNapi {
            preset: Some("compact".into()),
            sort_blocks: Some(true),
            ..Default::default()
        })
        .expect("native canonical block order");
        assert!(canonical.sort_blocks);
    }
}
