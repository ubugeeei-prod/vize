//! Style compilation for SFC.
//!
//! Compiles all `<style>` blocks in an SFC, applying scoped CSS
//! transformations when needed.

use crate::types::{CssModuleMapping, SfcError, SfcStyleBlock, StyleCompileOptions};

use vize_carton::{String, profile};

pub(super) struct CompiledStyles {
    pub(super) css: String,
    pub(super) css_modules: Vec<CssModuleMapping>,
}

/// Helper to compile all style blocks
pub(super) fn compile_styles(
    styles: &[SfcStyleBlock],
    scope_id: &str,
    base_opts: &StyleCompileOptions,
    warnings: &mut Vec<SfcError>,
) -> CompiledStyles {
    if styles.is_empty() {
        return CompiledStyles {
            css: String::default(),
            css_modules: Vec::new(),
        };
    }

    let mut all_css = String::default();
    let mut css_modules: Vec<CssModuleMapping> = Vec::new();
    for style in styles {
        let style_opts = StyleCompileOptions {
            id: {
                let mut id = String::with_capacity(scope_id.len() + 7);
                id.push_str("data-v-");
                id.push_str(scope_id);
                id
            },
            scoped: style.scoped || base_opts.scoped,
            ..base_opts.clone()
        };
        match profile!(
            "atelier.sfc.style.block",
            crate::style::compile_style_with_modules(style, &style_opts)
        ) {
            Ok(style_result) => {
                if !all_css.is_empty() {
                    all_css.push('\n');
                }
                all_css.push_str(&style_result.code);
                if let Some(css_module) = style_result.css_module {
                    if let Some(existing) = css_modules
                        .iter_mut()
                        .find(|existing| existing.name == css_module.name)
                    {
                        existing.exports.extend(css_module.exports);
                    } else {
                        css_modules.push(css_module);
                    }
                }
            }
            Err(e) => warnings.push(e),
        }
    }
    CompiledStyles {
        css: all_css,
        css_modules,
    }
}
