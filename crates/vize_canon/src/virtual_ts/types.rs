//! Type definitions for virtual TypeScript generation.

use std::ops::Range;
use vize_carton::String;
use vize_carton::config::VueVersion;

/// A mapping from generated virtual TS position to SFC source position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VizeMapping {
    /// Byte range in the generated virtual TypeScript.
    pub gen_range: Range<usize>,
    /// Byte range in the original SFC source.
    pub src_range: Range<usize>,
    /// Sub-token spans inside `gen_range`, paired with the matching SFC
    /// sub-range. Used to map TypeScript diagnostics that point at a
    /// specific identifier inside a template expression back to its exact
    /// position in the Vue source.
    ///
    /// Empty by default. Populated by template-expression generators that
    /// know the inner positions (e.g. property access inside an
    /// interpolation). Consumers fall back to `gen_range` / `src_range`
    /// when no sub-span covers the diagnostic.
    pub sub_spans: Vec<VizeSubSpan>,
}

/// A sub-token span inside a `VizeMapping`'s generated/source ranges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VizeSubSpan {
    pub gen_range: Range<usize>,
    pub src_range: Range<usize>,
}

impl VizeMapping {
    /// Look up the SFC sub-range that contains `gen_offset`. Returns `None`
    /// when no sub-span covers it — callers should fall back to `src_range`.
    pub fn sub_span_for_gen(&self, gen_offset: usize) -> Option<&VizeSubSpan> {
        self.sub_spans
            .iter()
            .find(|span| span.gen_range.contains(&gen_offset))
    }
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
    /// Template identifiers declared outside the SFC virtual module.
    ///
    /// Nuxt auto-imported components are declared in a generated ambient file.
    /// Keeping their names here prevents the virtual TS generator from
    /// shadowing them with local `any` fallbacks, so their real props remain
    /// type-checkable.
    pub external_template_bindings: Vec<String>,
}

impl Default for VirtualTsOptions {
    fn default() -> Self {
        Self {
            template_globals: default_plugin_globals(),
            css_modules: Vec::new(),
            auto_import_stubs: Vec::new(),
            external_template_bindings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct VirtualTsCheckOptions {
    pub(crate) check_props: bool,
    pub(crate) check_template_bindings: bool,
    pub(crate) check_emits: bool,
}

impl VirtualTsCheckOptions {
    pub(crate) fn any_enabled(self) -> bool {
        self.check_props || self.check_template_bindings || self.check_emits
    }
}

impl Default for VirtualTsCheckOptions {
    fn default() -> Self {
        Self {
            check_props: true,
            check_template_bindings: true,
            check_emits: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct VirtualTsGenerationOptions {
    pub(crate) check_options: VirtualTsCheckOptions,
    /// Configured Vue dialect for this project (default [`VueVersion::V3`]).
    ///
    /// Threaded from `vue.version` in `vize.config` through the check runner so
    /// canon can later emit dialect-aware instance types (e.g. a Vue 2 `this`
    /// shape). Plumbing only today: the generator carries it but does not branch
    /// on it yet, so default-V3 output stays byte-identical.
    pub(crate) dialect: VueVersion,
    /// Resolve Vue 3 Options API template bindings (opt-in, standard build).
    pub(crate) options_api: bool,
    /// Legacy Vue 2.7 / Nuxt 2 (implies `options_api` plus Nuxt 2 globals).
    pub(crate) legacy_vue2: bool,
    /// Preserve Vue parser compatibility semantics when generating template
    /// type checks.
    pub(crate) template_syntax_quirks: bool,
    /// Preserve TypeScript's user-authored unused local/import diagnostics by
    /// avoiding broad synthetic setup-binding references.
    pub(crate) preserve_unused_diagnostics: bool,
    /// Hoist the shared preamble (ImportMeta augmentation, type helpers, and
    /// compiler-macro signatures) out of the generated module. Callers that
    /// enable this must make the shared ambient helpers file
    /// (`SHARED_PREAMBLE_FILE_NAME` / `SHARED_PREAMBLE_DTS`) part of the same
    /// TypeScript program. Off by default so standalone single-document
    /// consumers keep self-contained output.
    pub(crate) hoist_shared_preamble: bool,
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
