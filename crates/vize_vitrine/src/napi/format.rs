//! NAPI bindings for Vue SFC formatting.

#![expect(
    clippy::disallowed_types,
    reason = "N-API values cross the boundary as std `String`s"
)]

use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use vize_glyph::{
    Allocator, FormatOptions, FormatPreset, GlyphFormatter, VueVersion, resolve_sort_imports,
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
    /// Layout line endings; auto follows the first source terminator.
    pub end_of_line: Option<EndOfLineNapi>,
    pub sort_attributes: Option<bool>,
    pub single_attribute_per_line: Option<bool>,
    pub max_attributes_per_line: Option<u32>,
    pub normalize_directive_shorthands: Option<bool>,
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
    if let Some(value) = opts.single_attribute_per_line {
        options.single_attribute_per_line = value;
    }
    if let Some(value) = opts.max_attributes_per_line {
        options.max_attributes_per_line = Some(value);
    }
    if let Some(value) = opts.normalize_directive_shorthands {
        options.normalize_directive_shorthands = value;
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

    Ok(options)
}

/// Format a Vue SFC source string.
#[napi(js_name = "formatSfc")]
pub fn format_sfc_napi(
    source: String,
    options: Option<FormatOptionsNapi>,
) -> Result<FormatResultNapi> {
    let options = options.unwrap_or_default();
    let vue_version = options
        .vue_version
        .as_deref()
        .map(|version| {
            VueVersion::from_config_str(version)
                .map_err(|error| Error::new(Status::InvalidArg, error.to_string()))
        })
        .transpose()?
        .unwrap_or_default();
    let sorting = options
        .sort_imports
        .as_ref()
        .map(|value| serde_json::from_value::<vize_l0::config::SortImportsSetting>(value.clone()))
        .transpose()
        .map_err(|error| Error::new(Status::InvalidArg, error.to_string()))?;
    let sorting = resolve_sort_imports(sorting.as_ref())
        .map_err(|error| Error::new(Status::InvalidArg, error.to_string()))?;
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
}
