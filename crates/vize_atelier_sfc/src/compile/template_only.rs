//! Case 1 of SFC compilation: a template with no `<script>` of either kind.
//!
//! Split out of `super::compile_sfc_inner` so that function stays inside the
//! per-file source-length budget while the other two cases keep their shape.

use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_carton::{String, profile};

use crate::compile_template::{
    TemplateBlockCompileContext, compile_template_block, compile_template_block_vapor,
};
use crate::types::{
    SfcCompileOptions, SfcCompileResult, SfcDescriptor, SfcError, SfcMacroArtifact,
};

use super::helpers::trim_trailing_newlines;
use super::output_module::{
    RenderFunctionName, append_component_render_export, finalize_output_mode,
};
use super::styles::CompiledStyles;

/// Everything the template-only path reads from the shared prelude.
pub(super) struct TemplateOnlyInput<'a> {
    pub(super) descriptor: &'a SfcDescriptor<'a>,
    pub(super) options: &'a SfcCompileOptions,
    pub(super) custom_elements: &'a CustomElementMatcher,
    pub(super) template_syntax: TemplateSyntaxMode,
    pub(super) codegen_options: &'a CodegenOptions,
    pub(super) compiled_styles: &'a CompiledStyles,
    pub(super) component_name: &'a str,
    pub(super) scope_id: &'a str,
    pub(super) has_scoped: bool,
    pub(super) is_vapor: bool,
    pub(super) template_is_ts: bool,
}

pub(super) fn compile_template_only(
    input: TemplateOnlyInput<'_>,
    css: Option<String>,
    errors: Vec<SfcError>,
    mut warnings: Vec<SfcError>,
    macro_artifacts: Vec<SfcMacroArtifact>,
) -> Result<SfcCompileResult, SfcError> {
    let template = input.descriptor.template.as_ref().unwrap();
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
                    is_ts: input.template_is_ts,
                    inline: false,
                    component_name: Some(input.component_name),
                    bindings: None,
                    croquis: None,
                },
                input.template_syntax,
                input.codegen_options,
            )
        )
    } else {
        let template_opts = input.options.template.clone();
        // Also pass scope IDs to the client template compiler. Vue's runtime
        // normally propagates __scopeId, but wrapper components such as NuxtLink
        // can otherwise lose parent scoped attrs before the final DOM root.
        profile!(
            "atelier.sfc.template.compile",
            compile_template_block(
                &template_allocator,
                template,
                &template_opts,
                input.custom_elements,
                TemplateBlockCompileContext {
                    scope_id: input.scope_id,
                    apply_scope_id: input.has_scoped,
                    has_scoped: input.has_scoped,
                    is_ts: input.template_is_ts,
                    inline: false,
                    component_name: Some(input.component_name),
                    bindings: None,
                    croquis: None,
                },
                input.template_syntax,
                input.codegen_options,
            )
        )
    };

    // Previously this just collected the error into a local vec
    // and continued, returning Ok with empty code — so callers
    // wrote a 0-byte module and exited 0 (#958). Propagate the
    // template error up so the build/CLI surfaces it.
    let template_output = template_result?;
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

    finalize_output_mode(
        &mut code,
        &mut warnings,
        input.options,
        input.codegen_options,
    );
    trim_trailing_newlines(&mut code);

    Ok(SfcCompileResult {
        code,
        css,
        map: None,
        errors,
        warnings,
        bindings: None,
        macro_artifacts,
    })
}
