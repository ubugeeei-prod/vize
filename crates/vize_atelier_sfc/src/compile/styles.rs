//! Style compilation for SFC.
//!
//! Compiles all `<style>` blocks in an SFC, applying scoped CSS
//! transformations when needed.

use crate::types::{SfcError, SfcStyleBlock, StyleCompileOptions};

use vize_carton::{profile, String};
/// Helper to compile all style blocks
pub(super) fn compile_styles(
    styles: &[SfcStyleBlock],
    scope_id: &str,
    base_opts: &StyleCompileOptions,
    warnings: &mut Vec<SfcError>,
) -> String {
    if styles.is_empty() {
        return String::default();
    }

    let mut all_css = String::default();
    for style in styles {
        let style_opts = StyleCompileOptions {
            id: {
                let mut id = String::with_capacity(scope_id.len() + 7);
                id.push_str("data-v-");
                id.push_str(scope_id);
                id
            },
            scoped: style.scoped,
            ..base_opts.clone()
        };
        match profile!(
            "atelier.sfc.style.block",
            crate::style::compile_style(style, &style_opts)
        ) {
            Ok(style_css) => {
                if !all_css.is_empty() {
                    all_css.push('\n');
                }
                all_css.push_str(&style_css);
            }
            Err(e) => warnings.push(e),
        }
    }
    all_css
}
