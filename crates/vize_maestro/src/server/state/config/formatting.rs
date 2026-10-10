//! Convert the shared formatter model into native formatter options.

use vize_l0::config::FormatterConfig;

pub(super) fn format_options_from_config(config: &FormatterConfig) -> vize_glyph::FormatOptions {
    vize_glyph::FormatOptions {
        print_width: config.print_width,
        tab_width: config.tab_width,
        use_tabs: config.use_tabs,
        semi: config.semi,
        single_quote: config.single_quote,
        jsx_single_quote: config.jsx_single_quote,
        trailing_comma: match config.trailing_comma {
            vize_l0::config::TrailingComma::None => vize_glyph::TrailingComma::None,
            vize_l0::config::TrailingComma::Es5 => vize_glyph::TrailingComma::Es5,
            vize_l0::config::TrailingComma::All => vize_glyph::TrailingComma::All,
        },
        bracket_spacing: config.bracket_spacing,
        bracket_same_line: config.bracket_same_line,
        arrow_parens: match config.arrow_parens {
            vize_l0::config::ArrowParens::Always => vize_glyph::ArrowParens::Always,
            vize_l0::config::ArrowParens::Avoid => vize_glyph::ArrowParens::Avoid,
        },
        end_of_line: match config.end_of_line {
            vize_l0::config::EndOfLine::Lf => vize_glyph::EndOfLine::Lf,
            vize_l0::config::EndOfLine::Crlf => vize_glyph::EndOfLine::Crlf,
            vize_l0::config::EndOfLine::Cr => vize_glyph::EndOfLine::Cr,
            vize_l0::config::EndOfLine::Auto => vize_glyph::EndOfLine::Auto,
        },
        quote_props: match config.quote_props {
            vize_l0::config::QuoteProps::AsNeeded => vize_glyph::QuoteProps::AsNeeded,
            vize_l0::config::QuoteProps::Consistent => vize_glyph::QuoteProps::Consistent,
            vize_l0::config::QuoteProps::Preserve => vize_glyph::QuoteProps::Preserve,
        },
        single_attribute_per_line: config.single_attribute_per_line,
        vue_indent_script_and_style: config.vue_indent_script_and_style,
        sort_attributes: config.sort_attributes,
        attribute_sort_order: match config.attribute_sort_order {
            vize_l0::config::AttributeSortOrder::Alphabetical => {
                vize_glyph::AttributeSortOrder::Alphabetical
            }
            vize_l0::config::AttributeSortOrder::AsWritten => {
                vize_glyph::AttributeSortOrder::AsWritten
            }
        },
        merge_bind_and_non_bind_attrs: config.merge_bind_and_non_bind_attrs,
        max_attributes_per_line: config.max_attributes_per_line,
        attribute_groups: config.attribute_groups.clone(),
        normalize_directive_shorthands: config.normalize_directive_shorthands,
        sort_blocks: config.sort_blocks,
        skip_script_stabilization: false,
    }
}
