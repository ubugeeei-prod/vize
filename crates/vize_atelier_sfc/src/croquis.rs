//! Shared Croquis analysis for SFC consumers.
//!
//! This module keeps descriptor-aware Croquis orchestration in one place so the
//! compiler, linter, type checker, and bindings do not each reinvent script
//! merging, generic extraction, and virtual-script offsets.

mod drawer;
mod source_offsets;

use self::drawer::{analyze_scripts, apply_options_api_mode};
use crate::types::SfcDescriptor;
use vize_atelier_core::RootNode;
use vize_carton::{FxHashSet, String, ToCompactString, cstr, profile};
use vize_croquis::{Croquis, Drawer, DrawerOptions};

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
    )
}

fn analyze_sfc_descriptor_resolved_impl(
    descriptor: &SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
    options_api: bool,
    legacy_vue2: bool,
    resolve_filename: Option<&str>,
) -> SfcCroquisAnalysis {
    let drawer_options = options.analyzer_options;
    let script_analyzed = drawer_options.analyze_script
        && (descriptor.script.is_some() || descriptor.script_setup.is_some());
    let mut summary = analyze_scripts(descriptor, options, options_api, legacy_vue2);
    if let Some(filename) = resolve_filename {
        merge_resolved_props_into_croquis(&mut summary, descriptor, filename);
    }
    let drawer = Drawer::with_summary(drawer_options, summary, script_analyzed);
    let mut drawer = apply_options_api_mode(drawer, options_api, legacy_vue2);

    if let Some(root) = template_ast {
        profile!("atelier.sfc.croquis.template", drawer.draw_template(root));
    }

    let (script_content, script_offset) = script_content_for_descriptor(descriptor, options);
    SfcCroquisAnalysis {
        croquis: drawer.finish(),
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

/// Merge props resolved by the script compile context — which performs
/// cross-file and node_modules type resolution — into a Croquis summary.
///
/// Croquis alone cannot resolve props inherited through imported or heritage
/// types (`interface Props extends Omit<ImportedProps, ...>`), so
/// template-binding checks and virtual TS generation would treat those props
/// as undefined references. This mirrors the merge the compiler performs in
/// `compile.rs`.
pub fn merge_resolved_props_into_croquis(
    croquis: &mut Croquis,
    descriptor: &SfcDescriptor<'_>,
    filename: &str,
) {
    use crate::script::ScriptCompileContext;
    use crate::types::{BindingType, is_ts_lang};

    let Some(script_setup) = descriptor.script_setup.as_ref() else {
        return;
    };

    let mut ctx = ScriptCompileContext::new(&script_setup.content);
    if let Some(ref script) = descriptor.script {
        ctx.collect_types_from(&script.content);
    }
    if !filename.is_empty() {
        ctx.collect_imported_types_from_path(
            &script_setup.content,
            filename,
            is_ts_lang(script_setup.lang.as_deref()),
        );
        if let Some(ref script) = descriptor.script {
            ctx.collect_imported_types_from_path(
                &script.content,
                filename,
                is_ts_lang(script.lang.as_deref()),
            );
        }
    }
    ctx.analyze();

    let Some(type_args) = croquis
        .macros
        .define_props()
        .and_then(|call| call.type_args.as_deref())
    else {
        return;
    };
    let type_args = type_args
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
        .unwrap_or(type_args);
    let mut known: FxHashSet<_> = croquis
        .macros
        .props()
        .iter()
        .map(|prop| prop.name.clone())
        .collect();
    for prop in ctx.resolve_type_props(type_args) {
        if !known.insert(prop.name.clone()) {
            continue;
        }
        if !croquis.bindings.contains(prop.name.as_str()) {
            croquis.bindings.add(prop.name.as_str(), BindingType::Props);
        }
        croquis.macros.add_prop(prop);
    }
}

#[cfg(test)]
mod tests;
