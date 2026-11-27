//! Projection of the shared options onto the Oxc JavaScript formatter.

use super::{ArrowParens, EndOfLine, FormatOptions, ObjectWrap, QuoteProps, TrailingComma};

impl FormatOptions {
    /// Convert to `oxc_formatter::JsFormatOptions`
    pub fn to_oxc_format_options(&self) -> oxc_formatter::JsFormatOptions {
        self.to_oxc_format_options_with_line_width(javascript_line_width(self.print_width))
    }

    pub(crate) fn to_oxc_format_options_with_line_width(
        &self,
        line_width: oxc_formatter_core::LineWidth,
    ) -> oxc_formatter::JsFormatOptions {
        // Oxc 0.142 renamed FormatOptions to JsFormatOptions; shared layout types
        // live in oxc_formatter_core and are not re-exported by oxc_formatter.
        use oxc_formatter::{
            ArrowParentheses, BracketSameLine, BracketSpacing, Expand, QuoteProperties, QuoteStyle,
            Semicolons, TrailingCommas,
        };
        use oxc_formatter_core::{IndentStyle, IndentWidth, LineEnding};

        oxc_formatter::JsFormatOptions {
            indent_style: if self.use_tabs {
                IndentStyle::Tab
            } else {
                IndentStyle::Space
            },
            indent_width: IndentWidth::try_from(self.tab_width).unwrap_or_default(),
            line_width,
            line_ending: match self.end_of_line {
                EndOfLine::Lf | EndOfLine::Auto => LineEnding::Lf,
                EndOfLine::Crlf => LineEnding::Crlf,
                EndOfLine::Cr => LineEnding::Cr,
            },
            quote_style: if self.single_quote {
                QuoteStyle::Single
            } else {
                QuoteStyle::Double
            },
            jsx_quote_style: if self.jsx_single_quote {
                QuoteStyle::Single
            } else {
                QuoteStyle::Double
            },
            quote_properties: match self.quote_props {
                QuoteProps::AsNeeded => QuoteProperties::AsNeeded,
                QuoteProps::Consistent => QuoteProperties::Consistent,
                QuoteProps::Preserve => QuoteProperties::Preserve,
            },
            expand: match self.object_wrap {
                ObjectWrap::Preserve => Expand::Auto,
                ObjectWrap::Collapse => Expand::Never,
            },
            semicolons: if self.semi {
                Semicolons::Always
            } else {
                Semicolons::AsNeeded
            },
            trailing_commas: match self.trailing_comma {
                TrailingComma::None => TrailingCommas::None,
                TrailingComma::Es5 => TrailingCommas::Es5,
                TrailingComma::All => TrailingCommas::All,
            },
            bracket_spacing: BracketSpacing::from(self.bracket_spacing),
            bracket_same_line: BracketSameLine::from(self.bracket_same_line),
            arrow_parentheses: match self.arrow_parens {
                ArrowParens::Always => ArrowParentheses::Always,
                ArrowParens::Avoid => ArrowParentheses::AsNeeded,
            },
            ..Default::default()
        }
    }
}

/// Preserve the formatter's existing fallback for widths outside its domain.
/// Narrowing first would reinterpret a large configured width modulo 65536.
pub(crate) fn javascript_line_width(width: u32) -> oxc_formatter_core::LineWidth {
    use oxc_formatter_core::LineWidth;

    // Apply the provider's existing domain before narrowing the integer.
    let min: u16 = LineWidth::MIN;
    let max: u16 = LineWidth::MAX;
    if !(u32::from(min)..=u32::from(max)).contains(&width) {
        return LineWidth::default();
    }
    // Both provider bounds are u16, so the admitted width narrows losslessly.
    LineWidth::try_from(width as u16).unwrap_or_default()
}
