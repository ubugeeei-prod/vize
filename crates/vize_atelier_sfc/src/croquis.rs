//! Shared Croquis analysis for SFC consumers.
//!
//! This module keeps descriptor-aware Croquis orchestration in one place so the
//! compiler, linter, type checker, and bindings do not each reinvent script
//! merging, generic extraction, and virtual-script offsets.

mod drawer;
#[path = "croquis/resolved_props.rs"]
mod resolved_props;

pub use resolved_props::merge_resolved_props_into_croquis;

use self::drawer::{analyze_scripts, apply_options_api_mode};
use crate::types::SfcDescriptor;
use vize_carton::{String, ToCompactString, cstr, profile};
use vize_croquis::{Croquis, Drawer, DrawerOptions};
use vize_relief::RootNode;

/// Options for descriptor-level Croquis analysis.
#[derive(Debug, Clone, Copy)]
pub struct SfcCroquisOptions {
    /// Low-level drawer options.
    pub analyzer_options: DrawerOptions,
    /// Merge `<script>` into the synthetic script used by downstream tools when
    /// a component also has `<script setup>`.
    pub merge_scripts: bool,
}

impl SfcCroquisOptions {
    /// Full analysis with split-script merging enabled.
    #[inline]
    pub const fn full() -> Self {
        Self {
            analyzer_options: DrawerOptions::full(),
            merge_scripts: true,
        }
    }

    /// Fast lint-oriented analysis.
    #[inline]
    pub const fn for_lint() -> Self {
        Self {
            analyzer_options: DrawerOptions::for_lint(),
            merge_scripts: true,
        }
    }

    /// Compilation-oriented analysis.
    #[inline]
    pub const fn for_compile() -> Self {
        Self {
            analyzer_options: DrawerOptions::for_compile(),
            merge_scripts: true,
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
            },
            merge_scripts: true,
        }
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

impl SfcCroquisAnalysis {
    #[inline]
    pub fn script_content_ref(&self) -> Option<&str> {
        self.script_content.as_deref()
    }
}

/// Analyze an SFC descriptor into a Croquis summary.
#[inline]
pub fn analyze_sfc_descriptor(
    descriptor: &SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
) -> Croquis {
    analyze_sfc_descriptor_with_context(descriptor, template_ast, options).croquis
}

/// Analyze an SFC descriptor and return matching script content/offset metadata.
pub fn analyze_sfc_descriptor_with_context(
    descriptor: &SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
) -> SfcCroquisAnalysis {
    analyze_sfc_descriptor_with_context_impl(descriptor, template_ast, options, false, false)
}

/// Analyze an SFC descriptor with Vue 3 Options API binding resolution enabled
/// (opt-in, standard build — no `legacy` feature required).
pub fn analyze_sfc_descriptor_with_context_options_api(
    descriptor: &SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
) -> SfcCroquisAnalysis {
    analyze_sfc_descriptor_with_context_impl(descriptor, template_ast, options, true, false)
}

/// Analyze an SFC descriptor with legacy Vue 2.7 / Nuxt 2 compatibility enabled
/// (implies Options API binding resolution plus Nuxt 2 template globals).
pub fn analyze_sfc_descriptor_with_context_legacy_vue2(
    descriptor: &SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
) -> SfcCroquisAnalysis {
    analyze_sfc_descriptor_with_context_impl(descriptor, template_ast, options, false, true)
}

fn analyze_sfc_descriptor_with_context_impl(
    descriptor: &SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
    options_api: bool,
    legacy_vue2: bool,
) -> SfcCroquisAnalysis {
    analyze_sfc_descriptor_resolved_impl(
        descriptor,
        template_ast,
        options,
        options_api,
        legacy_vue2,
        None,
        false,
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
    analyze_sfc_descriptor_resolved_impl(
        descriptor,
        template_ast,
        options,
        options_api,
        legacy_vue2,
        Some(filename),
        false,
    )
}

/// Resolve props after template traversal to preserve legacy Canon bytes.
///
/// The old batch type checker retained stale undefined-reference guards for
/// imported props because its resolver ran after Croquis drew the template.
/// This compatibility entry point keeps that exact output during the Atlas
/// migration while still performing one traversal and one resolution pass.
pub fn analyze_sfc_descriptor_resolved_for_canon_compatibility(
    descriptor: &SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
    options_api: bool,
    legacy_vue2: bool,
    filename: &str,
) -> SfcCroquisAnalysis {
    analyze_sfc_descriptor_resolved_impl(
        descriptor,
        template_ast,
        options,
        options_api,
        legacy_vue2,
        Some(filename),
        true,
    )
}

fn analyze_sfc_descriptor_resolved_impl(
    descriptor: &SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
    options_api: bool,
    legacy_vue2: bool,
    resolve_filename: Option<&str>,
    preserve_canon_after_template: bool,
) -> SfcCroquisAnalysis {
    let summary = analyze_scripts(descriptor, options, options_api, legacy_vue2);
    analyze_sfc_descriptor_with_script_analysis(
        descriptor,
        template_ast,
        options,
        options_api,
        legacy_vue2,
        resolve_filename,
        preserve_canon_after_template,
        summary,
    )
}

/// Finish descriptor/template analysis from an Atlas-cached script projection.
///
/// The script summary must have been produced for the requested Vue mode. This
/// entry point deliberately performs no JavaScript/TypeScript parse.
#[allow(clippy::too_many_arguments)]
pub(crate) fn analyze_sfc_descriptor_with_script_analysis(
    descriptor: &SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
    options_api: bool,
    legacy_vue2: bool,
    resolve_filename: Option<&str>,
    preserve_canon_after_template: bool,
    mut summary: Croquis,
) -> SfcCroquisAnalysis {
    let drawer_options = options.analyzer_options;
    let script_analyzed = drawer_options.analyze_script
        && (descriptor.script.is_some() || descriptor.script_setup.is_some());
    if !preserve_canon_after_template && let Some(filename) = resolve_filename {
        merge_resolved_props_into_croquis(&mut summary, descriptor, filename);
    }
    let drawer = Drawer::with_summary(drawer_options, summary, script_analyzed);
    let mut drawer = apply_options_api_mode(drawer, options_api, legacy_vue2);

    if let Some(root) = template_ast {
        profile!("atelier.sfc.croquis.template", drawer.draw_template(root));
    }

    let mut croquis = drawer.finish();
    if preserve_canon_after_template && let Some(filename) = resolve_filename {
        merge_resolved_props_into_croquis(&mut croquis, descriptor, filename);
    }
    let (script_content, script_offset) = script_content_for_descriptor(descriptor, options);
    SfcCroquisAnalysis {
        croquis,
        script_content,
        script_offset,
    }
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
mod tests {
    use super::{SfcCroquisOptions, analyze_sfc_descriptor_with_context};
    use crate::{SfcParseOptions, parse_sfc};

    #[test]
    fn split_scripts_share_one_synthetic_script_offset_space() {
        let source = r#"<script lang="ts">
import PlainCard from './PlainCard.vue'
export interface PlainProps { label: string }
</script>
<script setup lang="ts" generic="T">
import { ref } from 'vue'
const count = ref(0)
</script>
"#;
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        let analysis =
            analyze_sfc_descriptor_with_context(&descriptor, None, SfcCroquisOptions::full());
        let script = analysis.script_content_ref().unwrap();

        let plain_import = analysis
            .croquis
            .import_statements
            .iter()
            .find(|span| script[span.start as usize..span.end as usize].contains("PlainCard"));
        let setup_import = analysis
            .croquis
            .import_statements
            .iter()
            .find(|span| script[span.start as usize..span.end as usize].contains("{ ref }"));

        assert!(plain_import.is_some());
        assert!(setup_import.is_some());
        assert!(analysis.croquis.bindings.contains("PlainCard"));
        assert!(analysis.croquis.bindings.contains("count"));

        let count_span = analysis.croquis.binding_spans.get("count").unwrap();
        assert_eq!(
            &script[count_span.0 as usize..count_span.1 as usize],
            "count"
        );
    }
}
