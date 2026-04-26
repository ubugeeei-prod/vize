//! SSR compiler options.

use serde::{Deserialize, Serialize};
use vize_atelier_core::BindingMetadata;
use vize_carton::String;
use vize_croquis::Croquis;

/// SSR compiler options
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SsrCompilerOptions {
    /// Scope ID for scoped CSS (data-v-xxx)
    #[serde(default)]
    pub scope_id: Option<String>,

    /// Whether to preserve comments
    #[serde(default)]
    pub comments: bool,

    /// Current SFC component name for self-reference resolution
    #[serde(default)]
    pub component_name: Option<String>,

    /// Whether to inline template
    #[serde(default)]
    pub inline: bool,

    /// Whether is TypeScript
    #[serde(default)]
    pub is_ts: bool,

    /// Whether the template targets a custom renderer instead of the DOM.
    #[serde(default)]
    pub custom_renderer: bool,

    /// CSS variables to inject (from SFC <style> blocks with v-bind)
    #[serde(default)]
    pub ssr_css_vars: Option<String>,

    /// Binding metadata from script setup / script analysis
    #[serde(skip)]
    pub binding_metadata: Option<BindingMetadata>,

    /// Semantic analysis data from Croquis (optional, enhances transforms)
    #[serde(skip)]
    pub croquis: Option<Box<Croquis>>,
}

impl Clone for SsrCompilerOptions {
    fn clone(&self) -> Self {
        Self {
            scope_id: self.scope_id.clone(),
            comments: self.comments,
            component_name: self.component_name.clone(),
            inline: self.inline,
            is_ts: self.is_ts,
            custom_renderer: self.custom_renderer,
            ssr_css_vars: self.ssr_css_vars.clone(),
            binding_metadata: self.binding_metadata.clone(),
            // Croquis is consumed by the compiler; clones intentionally drop it.
            croquis: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SsrCompilerOptions;

    #[test]
    fn test_default_options() {
        let opts = SsrCompilerOptions::default();
        assert!(opts.scope_id.is_none());
        assert!(!opts.comments);
        assert!(opts.component_name.is_none());
        assert!(!opts.inline);
        assert!(!opts.is_ts);
        assert!(opts.ssr_css_vars.is_none());
        assert!(opts.binding_metadata.is_none());
        assert!(opts.croquis.is_none());
    }
}
