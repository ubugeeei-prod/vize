//! SSR compiler options.

use serde::{Deserialize, Serialize};
use vize_atelier_core::BindingMetadata;
use vize_atelier_core::lane::ScriptTables;
use vize_l0::String;
use vize_l0::config::VueVersion;

/// SSR compiler options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SsrCompilerOptions {
    /// Scope ID for scoped CSS (data-v-xxx)
    #[serde(default)]
    pub scope_id: Option<String>,

    /// Whether to preserve comments
    #[serde(default)]
    pub comments: bool,

    /// Enable experimental Vue in-tag comments (`// ...`) inside opening tags.
    #[serde(default)]
    pub experimental_in_tag_comments: bool,

    /// Enable experimental `v-match` / `v-when` patterned template desugaring.
    #[serde(default)]
    pub experimental_patterned_template: bool,

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

    /// The component hydrates as a Vapor component. Its markup keeps the
    /// anchors the Vapor runtime claims: the empty `v-if` placeholder where
    /// comments are otherwise dropped, a fragment around each Suspense slot,
    /// and nested fragments inside `TransitionGroup`.
    ///
    /// Left out of the serialized options while it is off, so option
    /// payloads recorded before it existed still describe the default.
    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    pub vapor: bool,

    /// CSS variables to inject (from SFC <style> blocks with v-bind)
    #[serde(default)]
    pub ssr_css_vars: Option<String>,

    /// Vue dialect resolved once per file from `vue.version`. Defaults to
    /// [`VueVersion::V3`]; threaded into the parser/transform options so
    /// legacy-capable builds can resolve dialect behavior. Internal-only, so it
    /// is not part of the serialized option surface.
    #[serde(skip)]
    pub dialect: VueVersion,

    /// Binding metadata from script setup / script analysis
    #[serde(skip)]
    pub binding_metadata: Option<BindingMetadata>,

    /// `<script setup>` reactivity and component registrations beyond the
    /// binding metadata.
    #[serde(skip)]
    pub script_tables: Option<ScriptTables>,
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
        assert!(opts.script_tables.is_none());
    }
}

/// Experimental SSR codegen options kept separate from [`SsrCompilerOptions`]
/// so existing Rust struct literals remain source-compatible.
#[derive(Debug, Clone, Default)]
pub struct SsrCompilerExperimentalOptions {
    /// Current SFC component name for self-reference resolution.
    pub component_name: Option<String>,
    /// Treat the reserved `<Self>` tag as a reference to the current SFC.
    pub self_component: bool,
    /// Generate a Source Map v3 document for SSR render code.
    pub source_map: bool,
    /// Filename recorded in the Source Map v3 `file` and `sources` fields.
    pub source_map_filename: Option<String>,
}
