//! Shared Croquis analysis for SFC consumers.
//!
//! This module keeps descriptor-aware Croquis orchestration in one place so the
//! compiler, linter, type checker, and bindings do not each reinvent script
//! merging, generic extraction, and virtual-script offsets.

use crate::types::SfcDescriptor;
use vize_atelier_core::RootNode;
use vize_carton::{String, ToCompactString, cstr, profile};
use vize_croquis::{Analyzer, AnalyzerOptions, Croquis};

/// Options for descriptor-level Croquis analysis.
#[derive(Debug, Clone, Copy)]
pub struct SfcCroquisOptions {
    /// Low-level analyzer options.
    pub analyzer_options: AnalyzerOptions,
    /// Merge `<script>` into the synthetic script used by downstream tools when
    /// a component also has `<script setup>`.
    pub merge_scripts: bool,
}

impl SfcCroquisOptions {
    /// Full analysis with split-script merging enabled.
    #[inline]
    pub const fn full() -> Self {
        Self {
            analyzer_options: AnalyzerOptions::full(),
            merge_scripts: true,
        }
    }

    /// Fast lint-oriented analysis.
    #[inline]
    pub const fn for_lint() -> Self {
        Self {
            analyzer_options: AnalyzerOptions::for_lint(),
            merge_scripts: true,
        }
    }

    /// Compilation-oriented analysis.
    #[inline]
    pub const fn for_compile() -> Self {
        Self {
            analyzer_options: AnalyzerOptions::for_compile(),
            merge_scripts: true,
        }
    }

    /// Script-only analysis for declaration generation.
    #[inline]
    pub const fn for_declaration() -> Self {
        Self {
            analyzer_options: AnalyzerOptions {
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

/// Apply the Options API / legacy binding-resolution mode to an analyzer.
/// `legacy_vue2` implies Options API resolution and additionally pulls in the
/// Nuxt 2 globals; `options_api` alone resolves only the Options API bindings.
fn apply_options_api_mode(analyzer: Analyzer, options_api: bool, legacy_vue2: bool) -> Analyzer {
    if legacy_vue2 {
        analyzer.with_legacy_vue2()
    } else if options_api {
        analyzer.with_options_api()
    } else {
        analyzer
    }
}

fn analyze_sfc_descriptor_with_context_impl(
    descriptor: &SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
    options_api: bool,
    legacy_vue2: bool,
) -> SfcCroquisAnalysis {
    let script_analyzed = options.analyzer_options.analyze_script
        && (descriptor.script.is_some() || descriptor.script_setup.is_some());
    let summary = analyze_scripts(descriptor, options, options_api, legacy_vue2);
    let analyzer = Analyzer::with_summary(options.analyzer_options, summary, script_analyzed);
    let mut analyzer = apply_options_api_mode(analyzer, options_api, legacy_vue2);

    if let Some(root) = template_ast {
        profile!(
            "atelier.sfc.croquis.template",
            analyzer.analyze_template(root)
        );
    }

    let (script_content, script_offset) = script_content_for_descriptor(descriptor, options);
    SfcCroquisAnalysis {
        croquis: analyzer.finish(),
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

fn analyze_scripts(
    descriptor: &SfcDescriptor<'_>,
    options: SfcCroquisOptions,
    options_api: bool,
    legacy_vue2: bool,
) -> Croquis {
    if !options.analyzer_options.analyze_script {
        return Croquis::new();
    }

    match (descriptor.script.as_ref(), descriptor.script_setup.as_ref()) {
        (Some(script), Some(script_setup)) if options.merge_scripts => {
            let plain_analyzer = Analyzer::with_options(options.analyzer_options);
            let mut plain_analyzer =
                apply_options_api_mode(plain_analyzer, options_api, legacy_vue2);
            profile!(
                "atelier.sfc.croquis.script_plain",
                plain_analyzer.analyze_script_plain(script.content.as_ref())
            );
            let plain = plain_analyzer.finish();

            let setup_analyzer = Analyzer::with_options(options.analyzer_options);
            let mut setup_analyzer =
                apply_options_api_mode(setup_analyzer, options_api, legacy_vue2);
            let generic = script_setup
                .attrs
                .get("generic")
                .map(|value| value.as_ref());
            profile!(
                "atelier.sfc.croquis.script_setup",
                setup_analyzer
                    .analyze_script_setup_with_generic(script_setup.content.as_ref(), generic)
            );

            let mut summary = setup_analyzer.finish();
            let setup_offset = script.content.len() as u32 + 1;
            summary.shift_script_offsets(setup_offset);
            summary.merge_plain_script(plain);
            summary
        }
        (_, Some(script_setup)) => {
            let analyzer = Analyzer::with_options(options.analyzer_options);
            let mut analyzer = apply_options_api_mode(analyzer, options_api, legacy_vue2);
            let generic = script_setup
                .attrs
                .get("generic")
                .map(|value| value.as_ref());
            profile!(
                "atelier.sfc.croquis.script_setup",
                analyzer.analyze_script_setup_with_generic(script_setup.content.as_ref(), generic)
            );
            analyzer.finish()
        }
        (Some(script), None) => {
            let analyzer = Analyzer::with_options(options.analyzer_options);
            let mut analyzer = apply_options_api_mode(analyzer, options_api, legacy_vue2);
            profile!(
                "atelier.sfc.croquis.script_plain",
                analyzer.analyze_script_plain(script.content.as_ref())
            );
            analyzer.finish()
        }
        (None, None) => Croquis::new(),
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
