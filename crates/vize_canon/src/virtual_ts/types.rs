//! Type definitions for virtual TypeScript generation.

mod checks;
#[cfg(feature = "native")]
pub(crate) use checks::ResolveStyleClassNames;
pub(crate) use checks::VirtualTsCheckOptions;

use vize_carton::{FxHashSet, String, config::VueVersion, cstr};

use super::semantic_links::VizeSemanticLink;

/// The generator's span-link rows are the rows of the one projection mapping
/// model, owned by [`super::mapping`].
pub use super::mapping::{VizeMapping, VizeSubSpan};

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

/// Marks a [`TemplateGlobal`] synthesized from an authored `<style module>`
/// block. The marker stays in the otherwise-unused fallback-value field so
/// per-SFC CSS-module metadata does not expand the public `VirtualTsOptions`
/// struct (which downstream users construct directly).
pub(crate) const CSS_MODULE_GLOBAL_MARKER: &str = "__vize_css_module_global";

impl TemplateGlobal {
    pub(crate) fn context_type_annotation(&self) -> String {
        if self.default_value == CSS_MODULE_GLOBAL_MARKER {
            self.type_annotation.clone()
        } else {
            cstr!("__Global<'{}', {}>", self.name, self.type_annotation)
        }
    }
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
    /// Value binding names the framework auto-imports into every SFC, when
    /// their declarations live outside this file.
    ///
    /// The batch virtual project materializes [`Self::auto_import_stubs`] into
    /// one program-wide ambient file and clears the per-file list. Template
    /// scope still has to unwrap those bindings' refs — an auto-import is a
    /// `<script setup>` import once the framework transform runs — so the names
    /// are carried here instead (#4146).
    pub auto_import_bindings: Vec<String>,
    /// Template identifiers declared outside the SFC virtual module.
    ///
    /// Nuxt auto-imported components are declared in a generated ambient file.
    /// Keeping their names here prevents the virtual TS generator from
    /// shadowing them with local `any` fallbacks, so their real props remain
    /// type-checkable.
    pub external_template_bindings: Vec<String>,
    /// Ambient declaration files that editor virtual documents must load.
    /// Paths are emitted as triple-slash references before generated code.
    pub reference_paths: Vec<String>,
    /// Resolve `$`-prefixed template instance globals by reading them off the
    /// component public instance instead of declaring them with a permissive
    /// `any` fallback.
    ///
    /// Set only when the project publishes an authoritative ambient declaration
    /// graph for those names — Nuxt's generated `.nuxt` types. A name that graph
    /// does not declare then reports the `TS2339` `vue-tsc` reports for it,
    /// instead of silently resolving to `any`, and a name it does declare
    /// resolves to its real declared type rather than a widened stand-in.
    pub strict_instance_globals: bool,
}

impl Default for VirtualTsOptions {
    fn default() -> Self {
        Self {
            template_globals: default_plugin_globals(),
            css_modules: Vec::new(),
            auto_import_stubs: Vec::new(),
            auto_import_bindings: Vec::new(),
            external_template_bindings: Vec::new(),
            reference_paths: Vec::new(),
            strict_instance_globals: false,
        }
    }
}

impl VirtualTsOptions {
    /// Every framework auto-import *value* binding available to an SFC.
    ///
    /// [`Self::auto_import_stubs`] carries the declarations inline for
    /// single-file callers (editor diagnostics, `type_check_sfc`); the batch
    /// virtual project moves them into one ambient file and leaves the names in
    /// [`Self::auto_import_bindings`]. Both spellings resolve here so template
    /// scope behaves identically in either path (#4146).
    pub(crate) fn auto_import_binding_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .auto_import_stubs
            .iter()
            .filter_map(|stub| typed_value_binding(stub))
            .map(String::from)
            .collect();
        names.extend(self.auto_import_bindings.iter().cloned());
        names.sort_unstable();
        names.dedup();
        names
    }

    pub(crate) fn has_auto_import_binding_name(&self, name: &str) -> bool {
        self.auto_import_bindings
            .iter()
            .any(|binding| binding.as_str() == name)
            || self
                .auto_import_stubs
                .iter()
                .filter_map(|stub| typed_value_binding(stub))
                .any(|binding| binding == name)
    }
}

/// The declared name of a stub that introduces a *value* binding carrying a
/// real type annotation.
///
/// `declare function` stubs are skipped: a function is never a ref, so an
/// unwrap shadow would only add output. So is the degraded `declare const X:
/// any;` fallback vize emits for a framework project with no generated types —
/// `__U<any>` is `any`, so shadowing it cannot change a single diagnostic.
/// `$`-prefixed names belong to the template context, which declares them
/// itself.
fn typed_value_binding(stub: &str) -> Option<&str> {
    let rest = ["declare const ", "declare let ", "declare var "]
        .into_iter()
        .find_map(|prefix| stub.strip_prefix(prefix))?;
    let (name, annotation) = rest.split_once(':')?;
    let name = name.trim();
    let annotation = annotation.trim().trim_end_matches(';').trim();
    if annotation.is_empty() || annotation == "any" || !is_plain_identifier(name) {
        return None;
    }
    Some(name)
}

fn is_plain_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}

pub(crate) const DEFAULT_LIB_REFERENCES: &[&str] = &["es2022", "dom", "dom.iterable"];

pub(crate) fn emit_lib_reference_directives(output: &mut String, lib_references: &[&str]) {
    for lib in lib_references {
        let lib = lib.trim();
        if lib.is_empty() || !is_safe_ts_lib_reference(lib) {
            continue;
        }
        output.push_str("/// <reference lib=\"");
        output.push_str(lib);
        output.push_str("\" />\n");
    }
}

pub(crate) fn is_safe_ts_lib_reference(lib: &str) -> bool {
    lib.bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct VirtualTsGenerationOptions<'a> {
    pub(crate) check_options: VirtualTsCheckOptions,
    /// Configured Vue dialect for this project (default [`VueVersion::V3`]).
    ///
    /// Threaded from `vue.version` in `vize.config` through the check runner so
    /// canon can emit dialect-aware virtual TypeScript while keeping default-V3
    /// output byte-identical.
    pub(crate) dialect: VueVersion,
    /// Resolve Vue 3 Options API template bindings (opt-in, standard build).
    pub(crate) options_api: bool,
    /// Preserve the typed authored default component in the public instance.
    /// Declaration-producing callers enable this until batch baselines move.
    pub(crate) preserve_authored_component: bool,
    /// Public-facing component symbol used by Content Mapper hover responses.
    /// `None` preserves the internal default export used by batch projections.
    pub(crate) component_name: Option<&'a str>,
    /// Authored file stem used for implicit recursive component resolution.
    pub(crate) self_component_name: Option<&'a str>,
    /// Preserve symbol links from component listeners to authored emit keys.
    pub(crate) preserve_event_navigation: bool,
    /// Legacy Vue 2.7 / Nuxt 2 (implies `options_api` plus Nuxt 2 globals).
    pub(crate) legacy_vue2: bool,
    /// Preserve Vue parser compatibility semantics when generating template
    /// type checks.
    pub(crate) template_syntax_quirks: bool,
    /// Preserve TypeScript's user-authored unused local/import diagnostics by
    /// avoiding broad synthetic setup-binding references.
    pub(crate) preserve_unused_diagnostics: bool,
    /// Extra names referenced by template-like custom blocks. Used only by
    /// unused-local preservation so non-template blocks can mark setup
    /// bindings as read without opting into full template type checks.
    pub(crate) extra_template_referenced_names: Option<&'a FxHashSet<String>>,
    /// Hoist the shared preamble (ImportMeta augmentation, type helpers, and
    /// compiler-macro signatures) out of the generated module. Callers that
    /// enable this must make the shared ambient helpers file
    /// (`SHARED_PREAMBLE_FILE_NAME` / `SHARED_PREAMBLE_DTS`) part of the same
    /// TypeScript program. Off by default so standalone single-document
    /// consumers keep self-contained output.
    pub(crate) hoist_shared_preamble: bool,
    /// Override standard library triple-slash references for this generation.
    pub(crate) lib_references: Option<&'a [&'a str]>,
    /// Avoid introducing a `vite/client` type dependency in arbitrary projects.
    pub(crate) omit_vite_client_reference: bool,
    /// Synthetic merged-script offset and SFC-absolute source offset of
    /// `<script setup>`. Split-script analysis joins the two blocks with one
    /// newline, while the authored closing/opening tags occupy more bytes.
    pub(crate) split_script_setup_offsets: Option<(usize, usize)>,
    /// End offsets of script blocks with parse errors. Prevent the next block
    /// or generated helpers from becoming part of an unfinished expression.
    pub(crate) script_syntax_boundaries: &'a [usize],
    pub(crate) experimental_strict_slot_children: bool,
}

impl VirtualTsGenerationOptions<'_> {
    pub(crate) fn script_source_offset(self, script_offset: u32, offset: usize) -> usize {
        if let Some((synthetic_setup_start, source_setup_start)) = self.split_script_setup_offsets
            && offset >= synthetic_setup_start
        {
            return source_setup_start.saturating_add(offset - synthetic_setup_start);
        }
        (script_offset as usize).saturating_add(offset)
    }

    /// A normal-script component contributes its public instance; beside
    /// setup, an object default contributes options only. Both still retain
    /// the authored value so a non-object default keeps its exact type.
    pub(crate) fn authored_default(
        self,
        declared_default_alias: bool,
        has_script_setup: bool,
    ) -> AuthoredDefaultKind {
        match (
            self.preserve_authored_component && declared_default_alias,
            has_script_setup,
        ) {
            (false, _) => AuthoredDefaultKind::None,
            (true, true) => AuthoredDefaultKind::Options,
            (true, false) => AuthoredDefaultKind::Component,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthoredDefaultKind {
    None,
    Options,
    Component,
}

/// Default plugin globals.
/// Returns empty by default. Configure via `vize.config.pkl` `globalTypes`
/// or `typeChecker.globalsFile`.
fn default_plugin_globals() -> Vec<TemplateGlobal> {
    vec![]
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;

/// Output of virtual TypeScript generation.
#[derive(Debug)]
pub struct VirtualTsOutput {
    /// The generated TypeScript code.
    pub code: String,
    /// Source mappings from virtual TS positions to SFC positions.
    pub mappings: Vec<VizeMapping>,
    /// Stable semantic links between generated ranges.
    pub semantic_links: Vec<VizeSemanticLink>,
}
