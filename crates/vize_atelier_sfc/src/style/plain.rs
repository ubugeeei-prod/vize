//! Block output that skips CSS-module compilation.

use vize_carton::{String, ToCompactString};

use crate::types::{SfcStyleBlock, StyleCompileOptions};

pub(super) fn plain_style_output(style: &SfcStyleBlock, options: &StyleCompileOptions) -> String {
    let (mut output, _) = crate::css::transform_css_v_bind(&style.content, Some(&options.id));
    // Apply scoped transformation if needed
    if style.scoped || options.scoped {
        output = crate::css::scope_mode::scope_style_css(&output, &options.id);
    }
    // Trim if requested
    if options.trim {
        output = output.trim().to_compact_string();
    }
    output
}

/// Preprocessor source is compiled by the bundler, not by plain CSS parsing.
pub(super) fn is_preprocessor_lang(lang: Option<&str>) -> bool {
    matches!(lang, Some("scss" | "sass" | "less" | "stylus" | "styl"))
}
