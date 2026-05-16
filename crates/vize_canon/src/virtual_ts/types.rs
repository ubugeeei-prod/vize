//! Type definitions for virtual TypeScript generation.

use std::ops::Range;
use vize_carton::String;

/// A mapping from generated virtual TS position to SFC source position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VizeMapping {
    /// Byte range in the generated virtual TypeScript.
    pub gen_range: Range<usize>,
    /// Byte range in the original SFC source.
    pub src_range: Range<usize>,
}

/// A user-defined template global variable (e.g., `$t` from vue-i18n).
#[derive(Debug, Clone)]
pub struct TemplateGlobal {
    /// Variable name (e.g., "$t")
    pub name: String,
    /// TypeScript type annotation (e.g., "(...args: any[]) => string")
    pub type_annotation: String,
    /// Default value expression (e.g., "(() => '') as any")
    pub default_value: String,
}

/// Options for virtual TypeScript generation.
#[derive(Debug, Clone)]
pub struct VirtualTsOptions {
    /// Additional template globals beyond Vue core ($attrs, $slots, $refs, $emit).
    /// Use this to declare plugin globals like $t (vue-i18n), $route (vue-router), etc.
    pub template_globals: Vec<TemplateGlobal>,
    /// CSS module names from `<style module>` blocks (e.g., "$style", "$custom").
    pub css_modules: Vec<String>,
    /// Auto-import stub declarations (e.g., Nuxt composables).
    /// Each entry is a full TypeScript `declare function ...;` statement.
    pub auto_import_stubs: Vec<String>,
}

impl Default for VirtualTsOptions {
    fn default() -> Self {
        Self {
            template_globals: default_plugin_globals(),
            css_modules: Vec::new(),
            auto_import_stubs: Vec::new(),
        }
    }
}

/// Default plugin globals.
/// Returns empty by default. Configure via `vize.config.pkl` `globalTypes`
/// or `typeChecker.globalsFile`.
fn default_plugin_globals() -> Vec<TemplateGlobal> {
    vec![]
}

/// Output of virtual TypeScript generation.
#[derive(Debug)]
pub struct VirtualTsOutput {
    /// The generated TypeScript code.
    pub code: String,
    /// Source mappings from virtual TS positions to SFC positions.
    pub mappings: Vec<VizeMapping>,
}
