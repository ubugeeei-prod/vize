//! Shared Croquis analysis for SFC consumers.
//!
//! This module keeps descriptor-aware Croquis orchestration in one place so the
//! compiler, linter, type checker, and bindings do not each reinvent script
//! merging, generic extraction, and virtual-script offsets.

mod analysis;
mod drawer;
mod occurrences;
mod resolved;
mod script_demand;
mod source_offsets;
mod unused;

use self::analysis::{DescriptorAnalysisMode, analyze_sfc_descriptor_resolved_impl};
use crate::types::SfcDescriptor;
pub use occurrences::analyze_sfc_descriptor_with_occurrences;
pub use resolved::{
    merge_resolved_props_into_croquis, merge_resolved_props_into_croquis_with_sources,
};
use vize_atelier_core::RootNode;
use vize_carton::{String, ToCompactString, cstr};
use vize_croquis::{Croquis, DrawerOptions};

/// Options for descriptor-level Croquis analysis.
#[derive(Debug, Clone, Copy)]
pub struct SfcCroquisOptions {
    /// Low-level drawer options.
    pub analyzer_options: DrawerOptions,
    /// Merge `<script>` into the synthetic script used by downstream tools when
    /// a component also has `<script setup>`.
    pub merge_scripts: bool,
    /// Populate unused bindings only for a consumer that demands them.
    pub unused_bindings: bool,
    /// The supplied template AST is a proven dialect-derived HTML view.
    pub template_is_derived: bool,
}

impl SfcCroquisOptions {
    /// Full analysis with split-script merging enabled.
    #[inline]
    pub const fn full() -> Self {
        Self {
            analyzer_options: DrawerOptions::full(),
            merge_scripts: true,
            unused_bindings: false,
            template_is_derived: false,
        }
    }

    /// Lint demand: script, scopes, usage, and undefined refs. Hoists and
    /// template-expression collection stay off.
    #[inline]
    pub const fn lint_demand() -> Self {
        Self {
            analyzer_options: DrawerOptions::for_lint(),
            merge_scripts: true,
            unused_bindings: false,
            template_is_derived: false,
        }
    }

    /// Fast lint-oriented analysis.
    #[inline]
    pub const fn for_lint() -> Self {
        Self::lint_demand()
    }

    /// Compilation-oriented analysis.
    #[inline]
    pub const fn for_compile() -> Self {
        Self {
            analyzer_options: DrawerOptions::compile_demand(),
            merge_scripts: true,
            unused_bindings: false,
            template_is_derived: false,
        }
    }

    /// Script-only analysis for declaration generation.
    #[inline]
    pub const fn for_declaration() -> Self {
        Self {
            analyzer_options: DrawerOptions {
                analyze_script: true,
                analyze_template_scopes: false,
                track_usage: false,
                detect_undefined: false,
                analyze_hoisting: false,
                collect_template_expressions: false,
                experimental_patterned_template: false,
            },
            merge_scripts: true,
            unused_bindings: false,
            template_is_derived: false,
        }
    }

    /// Demand setup binding usage; ordinary compiler routes leave it off.
    pub const fn with_unused_bindings(mut self) -> Self {
        self.unused_bindings = true;
        self
    }

    /// Preserve authored dialect metadata while admitting its derived AST.
    pub const fn with_derived_template(mut self) -> Self {
        self.template_is_derived = true;
        self
    }

    /// Use only the active Vue script block instead of merging split scripts.
    #[inline]
    pub const fn without_script_merge(mut self) -> Self {
        self.merge_scripts = false;
        self
    }
}

impl Default for SfcCroquisOptions {
    fn default() -> Self {
        Self::full()
    }
}

/// Descriptor-level analysis plus the script view that matches its offsets.
#[derive(Debug)]
pub struct SfcCroquisAnalysis {
    pub croquis: Croquis,
    pub script_content: Option<String>,
    pub script_offset: u32,
}

/// Analyze an SFC descriptor into a Croquis summary.
#[inline]
pub fn analyze_sfc_descriptor(
    descriptor: &SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
) -> Croquis {
    analyze_sfc_descriptor_with_context_impl(descriptor, template_ast, options, false, false, false)
        .croquis
}

/// Analyze an SFC descriptor and return matching script content/offset metadata.
pub fn analyze_sfc_descriptor_with_context(
    descriptor: &SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
) -> SfcCroquisAnalysis {
    analyze_sfc_descriptor_with_context_impl(descriptor, template_ast, options, false, false, true)
}

/// Analyze an SFC descriptor with Vue 3 Options API binding resolution enabled
/// (opt-in, standard build — no `legacy` feature required).
pub fn analyze_sfc_descriptor_with_context_options_api(
    descriptor: &SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
) -> SfcCroquisAnalysis {
    analyze_sfc_descriptor_with_context_impl(descriptor, template_ast, options, true, false, true)
}

/// Analyze an SFC descriptor with legacy Vue 2.7 / Nuxt 2 compatibility enabled
/// (implies Options API binding resolution plus Nuxt 2 template globals).
pub fn analyze_sfc_descriptor_with_context_legacy_vue2(
    descriptor: &SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
) -> SfcCroquisAnalysis {
    analyze_sfc_descriptor_with_context_impl(descriptor, template_ast, options, false, true, true)
}

fn analyze_sfc_descriptor_with_context_impl(
    descriptor: &SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
    options_api: bool,
    legacy_vue2: bool,
    include_script_content: bool,
) -> SfcCroquisAnalysis {
    analyze_sfc_descriptor_resolved_impl::<false>(
        descriptor,
        template_ast,
        options,
        DescriptorAnalysisMode {
            options_api,
            legacy_vue2,
            include_script_content,
        },
        None,
        None,
        None,
    )
}

/// Analyze an SFC descriptor with externally-resolved props merged in before
/// template analysis.
///
/// Croquis alone cannot resolve props inherited through imported or heritage
/// types (`interface Props extends Omit<ImportedProps, ...>`); the script
/// compile context can (cross-file and node_modules type resolution), and the
/// merge must land before the template pass or its undefined-reference
/// detection flags those props as editor-only false positives that
/// `vize check` never reported.
pub fn analyze_sfc_descriptor_resolved(
    descriptor: &SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
    options_api: bool,
    legacy_vue2: bool,
    filename: &str,
) -> SfcCroquisAnalysis {
    analyze_sfc_descriptor_resolved_impl::<false>(
        descriptor,
        template_ast,
        options,
        DescriptorAnalysisMode {
            options_api,
            legacy_vue2,
            include_script_content: true,
        },
        Some(filename),
        None,
        None,
    )
}

/// Analyze a publication against one immutable source snapshot shared by
/// compatibility props and the scoped public type world.
pub fn analyze_sfc_descriptor_resolved_with_sources(
    descriptor: &SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
    options_api: bool,
    legacy_vue2: bool,
    filename: &str,
    sources: &crate::script::TypeSourceSnapshot,
) -> SfcCroquisAnalysis {
    analyze_sfc_descriptor_resolved_impl::<false>(
        descriptor,
        template_ast,
        options,
        DescriptorAnalysisMode {
            options_api,
            legacy_vue2,
            include_script_content: true,
        },
        Some(filename),
        Some(sources),
        None,
    )
}

/// Build the script content view that matches `analyze_sfc_descriptor`.
pub fn script_content_for_descriptor(
    descriptor: &SfcDescriptor<'_>,
    options: SfcCroquisOptions,
) -> (Option<String>, u32) {
    match (descriptor.script.as_ref(), descriptor.script_setup.as_ref()) {
        (Some(script), Some(script_setup)) if options.merge_scripts => (
            Some(cstr!("{}\n{}", script.content, script_setup.content)),
            script.loc.start as u32,
        ),
        (_, Some(script_setup)) => (
            Some(script_setup.content.to_compact_string()),
            script_setup.loc.start as u32,
        ),
        (Some(script), None) => (
            Some(script.content.to_compact_string()),
            script.loc.start as u32,
        ),
        (None, None) => (None, 0),
    }
}

#[cfg(test)]
mod tests;
