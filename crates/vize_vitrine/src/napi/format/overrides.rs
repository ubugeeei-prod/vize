//! Explicit Node formatter options applied over a base option set.

use super::{EndOfLineNapi, FormatOptionsNapi};
use napi::bindgen_prelude::{Error, Result, Status};
use vize_glyph::{
    ArrowParens, AttributeSortOrder, BlockSpacing, FormatOptions, FormatPreset, ObjectWrap,
    QuoteProps, TrailingComma,
};

/// Apply an options object on top of `options`: a preset first, then every
/// option it names, as command-line flags apply over a project config.
pub(in crate::napi) fn apply_overrides(
    options: &mut FormatOptions,
    opts: FormatOptionsNapi,
) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::{BlockSpacing, FormatOptionsNapi, apply_overrides};
    use vize_glyph::FormatOptions;

    fn apply(opts: FormatOptionsNapi) -> napi::Result<FormatOptions> {
        let mut options = FormatOptions::default();
        apply_overrides(&mut options, opts)?;
        Ok(options)
    }

    #[test]
    fn native_block_spacing_option_overrides_presets() {
        let prettier = apply(FormatOptionsNapi {
            preset: Some("prettier".into()),
            ..Default::default()
        })
        .expect("native prettier preset");
        assert_eq!(prettier.block_spacing, BlockSpacing::Preserve);

        let blank_line = apply(FormatOptionsNapi {
            preset: Some("prettier".into()),
            block_spacing: Some("blank-line".into()),
            ..Default::default()
        })
        .expect("native blank-line block spacing");
        assert_eq!(blank_line.block_spacing, BlockSpacing::BlankLine);

        let preserve = apply(FormatOptionsNapi {
            block_spacing: Some("preserve".into()),
            ..Default::default()
        })
        .expect("native preserved block spacing");
        assert_eq!(preserve.block_spacing, BlockSpacing::Preserve);

        assert!(
            apply(FormatOptionsNapi {
                block_spacing: Some("blank".into()),
                ..Default::default()
            })
            .is_err()
        );
    }
}
