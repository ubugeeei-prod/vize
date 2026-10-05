//! Descriptor analysis orchestration shared by all SFC consumers.

use super::drawer::{analyze_scripts, apply_options_api_mode};
use super::resolved::{
    merge_resolved_props_into_croquis, merge_resolved_props_into_croquis_with_sources,
};
use super::{SfcCroquisAnalysis, SfcCroquisOptions, script_content_for_descriptor, unused};
use crate::types::SfcDescriptor;
use vize_atelier_core::RootNode;
use vize_carton::profile;
use vize_croquis::Drawer;

pub(super) struct DescriptorAnalysisMode {
    pub(super) options_api: bool,
    pub(super) legacy_vue2: bool,
    pub(super) include_script_content: bool,
}

pub(super) fn analyze_sfc_descriptor_resolved_impl(
    descriptor: &SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
    options: SfcCroquisOptions,
    mode: DescriptorAnalysisMode,
    resolve_filename: Option<&str>,
    sources: Option<&crate::script::TypeSourceSnapshot>,
) -> SfcCroquisAnalysis {
    let drawer_options = options.analyzer_options;
    let script_analyzed = drawer_options.analyze_script
        && (descriptor.script.is_some() || descriptor.script_setup.is_some());
    let mut summary = analyze_scripts(descriptor, options, mode.options_api, mode.legacy_vue2);
    if let Some(filename) = resolve_filename {
        match sources {
            Some(sources) => merge_resolved_props_into_croquis_with_sources(
                &mut summary,
                descriptor,
                filename,
                sources,
            ),
            None => merge_resolved_props_into_croquis(&mut summary, descriptor, filename),
        }
    }
    let drawer = Drawer::with_summary(drawer_options, summary, script_analyzed);
    let drawer = if options.unused_bindings {
        drawer.with_unused_bindings()
    } else {
        drawer
    };
    let mut drawer = apply_options_api_mode(drawer, mode.options_api, mode.legacy_vue2);

    if let Some(root) = template_ast {
        profile!("atelier.sfc.croquis.template", drawer.draw_template(root));
    }

    let mut croquis = drawer.finish();
    if options.unused_bindings {
        if descriptor.template.is_some() && template_ast.is_none() {
            croquis.unused_bindings.clear();
        } else {
            unused::apply_style_reads(&mut croquis, descriptor, options.template_is_derived);
        }
    }
    let (script_content, script_offset) = if mode.include_script_content {
        script_content_for_descriptor(descriptor, options)
    } else {
        (None, 0)
    };
    SfcCroquisAnalysis {
        croquis,
        script_content,
        script_offset,
    }
}
