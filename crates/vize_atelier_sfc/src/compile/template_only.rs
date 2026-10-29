//! Case 1 of SFC compilation: a template with no `<script>` of either kind.
//!
//! Split out of `super::compile_sfc_inner` so that function stays inside the
//! per-file source-length budget while the other two cases keep their shape.

use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_carton::{String, profile};
use vize_l0::dump::capture::StageCapture;
use vize_l1_to_l2::lower::pug::PugSourceMap;
use vize_l4::module::RenderFunctionName;

use crate::compile_template::{
    TemplateBlockCompileContext, compile_template_block_vapor, compile_template_block_with_capture,
};
use crate::module_map::{Runs, module_map_value};
use crate::types::{
    SfcCompileOptions, SfcCompileResult, SfcError, SfcMacroArtifact, SfcTemplateBlock,
};

use super::helpers::trim_trailing_newlines;
use super::output_module::{append_component_render_export, finalize_output_mode};
use super::styles::CompiledStyles;

/// Everything the template-only path reads from the shared prelude.
pub(super) struct TemplateOnlyInput<'a, 'c> {
    pub(super) template: &'a SfcTemplateBlock<'a>,
    pub(super) source: &'a str,
    pub(super) filename: &'a str,
    pub(super) pug_source_map: Option<&'a PugSourceMap>,
    pub(super) options: &'a SfcCompileOptions,
    pub(super) custom_elements: &'a CustomElementMatcher,
    pub(super) template_syntax: TemplateSyntaxMode,
    pub(super) codegen_options: &'a CodegenOptions,
    pub(super) compiled_styles: &'a CompiledStyles,
    pub(super) component_name: &'a str,
    pub(super) scope_id: &'a str,
    pub(super) has_scoped: bool,
    pub(super) slotted: bool,
    pub(super) is_vapor: bool,
    pub(super) template_is_ts: bool,
    pub(super) experimental_self_component: bool,
    pub(super) capture: Option<&'c mut StageCapture>,
}

pub(super) fn compile_template_only(
    mut input: TemplateOnlyInput<'_, '_>,
    css: Option<String>,
    errors: Vec<SfcError>,
    mut warnings: Vec<SfcError>,
    macro_artifacts: Vec<SfcMacroArtifact>,
) -> Result<SfcCompileResult, SfcError> {
    let template = input.template;
    // P1-11: this worker's arena, reset and reused between files.
    let template_allocator = vize_carton::pool::acquire();
    let template_result = if input.is_vapor {
        profile!(
            "atelier.sfc.template.vapor",
            compile_template_block_vapor(
                &template_allocator,
                template,
                &input.options.template,
                input.custom_elements,
                TemplateBlockCompileContext {
                    scope_id: input.scope_id,
                    apply_scope_id: input.has_scoped,
                    has_scoped: input.has_scoped,
                    slotted: input.slotted,
                    is_ts: input.template_is_ts,
                    inline: false,
                    component_name: Some(input.component_name),
                    experimental_self_component: input.experimental_self_component,
                    bindings: None,
                    croquis: None,
                    collect_map: input.pug_source_map.is_some(),
                    authored_source: Some(input.source),
                    pug_source_map: input.pug_source_map,
                },
                input.template_syntax,
                input.codegen_options,
                input.capture.as_deref_mut(),
            )
        )
    } else {
        let template_opts = input.options.template.clone();
        // Also pass scope IDs to the client template compiler. Vue's runtime
        // normally propagates __scopeId, but wrapper components such as NuxtLink
        // can otherwise lose parent scoped attrs before the final DOM root.
        profile!(
            "atelier.sfc.template.compile",
            compile_template_block_with_capture(
                &template_allocator,
                template,
                &template_opts,
                input.custom_elements,
                TemplateBlockCompileContext {
                    scope_id: input.scope_id,
                    apply_scope_id: input.has_scoped,
                    has_scoped: input.has_scoped,
                    slotted: input.slotted,
                    is_ts: input.template_is_ts,
                    inline: false,
                    component_name: Some(input.component_name),
                    experimental_self_component: input.experimental_self_component,
                    bindings: None,
                    croquis: None,
                    collect_map: input.pug_source_map.is_some(),
                    authored_source: Some(input.source),
                    pug_source_map: input.pug_source_map,
                },
                input.template_syntax,
                input.codegen_options,
                input.capture.as_deref_mut(),
            )
        )
    };

    // Previously this just collected the error into a local vec
    // and continued, returning Ok with empty code — so callers
    // wrote a 0-byte module and exited 0 (#958). Propagate the
    // template error up so the build/CLI surfaces it.
    let template_output = template_result?;
    let template_map = template_output.map;
    warnings.extend(template_output.warnings);
    let mut code = {
        let mut code = template_output.code;
        if input.is_vapor {
            code.push_str("const _sfc_main = { __vapor: true }\n");
            append_component_render_export(
                &mut code,
                "_sfc_main",
                RenderFunctionName::Render,
                &input.compiled_styles.css_modules,
            );
        } else if input.options.template.ssr {
            code.push_str("const _sfc_main = {}\n");
            append_component_render_export(
                &mut code,
                "_sfc_main",
                RenderFunctionName::SsrRender,
                &input.compiled_styles.css_modules,
            );
        } else if !input.compiled_styles.css_modules.is_empty() {
            code.push_str("const _sfc_main = {}\n");
            append_component_render_export(
                &mut code,
                "_sfc_main",
                RenderFunctionName::Render,
                &input.compiled_styles.css_modules,
            );
        }
        code
    };

    let rewrite = finalize_output_mode(
        &mut code,
        &mut warnings,
        input.options,
        input.codegen_options,
    );
    trim_trailing_newlines(&mut code);

    let map = template_map.and_then(|map| {
        let authored = authored_template_points(&map, template.loc.start, input.pug_source_map);
        let authored = match &rewrite {
            Some(rewrite) => rewrite.compose(&authored),
            None => authored,
        };
        module_map_value(&code, authored, input.filename, input.source)
    });

    Ok(SfcCompileResult {
        code,
        css,
        map,
        errors,
        warnings,
        bindings: None,
        macro_artifacts,
        dependencies: Vec::new(),
    })
}

fn authored_template_points(
    generated: &Runs,
    template_start: usize,
    pug_source_map: Option<&PugSourceMap>,
) -> Runs {
    let mut authored = Runs::default();
    for &(output, source) in generated.points() {
        let source = pug_source_map.map_or(source, |map| map.offset_to_pug(source as u32) as usize);
        authored.point(output, template_start + source);
    }
    authored
}
