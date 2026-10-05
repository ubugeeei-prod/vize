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

/// Match Vue's SFC slotted-style detection, including the legacy function form.
pub(super) fn scope_flags(styles: &[SfcStyleBlock]) -> (bool, bool, bool) {
    (
        !styles.is_empty(),
        styles.iter().any(|style| style.scoped),
        styles.iter().any(|style| {
            style.scoped
                && (style.content.contains(":slotted(") || style.content.contains("::v-slotted("))
        }),
    )
}

impl CompiledStyles {
    /// Production script and SSR values use the same hashed names as CSS.
    pub(super) fn with_css_var_names(
        mut self,
        vars: &[std::borrow::Cow<'_, str>],
        scope_id: &str,
        filename: &str,
        is_prod: bool,
    ) -> Self {
        if is_prod {
            for expression in vars {
                let from = vize_carton::cstr!(
                    "var(--{})",
                    crate::css::scoped_v_bind_name(scope_id, expression)
                );
                let to = vize_carton::cstr!(
                    "var(--{})",
                    crate::css::prod_scoped_v_bind_name(filename, expression)
                );
                self.css = self.css.replace(from.as_str(), to.as_str()).into();
            }
        }
        self
    }
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
            // Full-SFC callers pass the component's aggregate scoped flag.
            // Scope belongs to each original block, so an ordinary sibling
            // must stay global even when another block is scoped.
            scoped: style.scoped,
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
