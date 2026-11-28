//! Case 2 of SFC compilation: a `<script>` without `<script setup>`, with or
//! without a template. The default export is rewritten and the template, if
//! any, is compiled beside it.

use vize_carton::{String, profile};
use vize_l0::dump::capture::StageCapture;
use vize_l4::module::RenderFunctionName;

use crate::compile_template::{compile_template_block_vapor, compile_template_block_with_capture};
use crate::types::{SfcCompileResult, SfcError, SfcScriptBlock};

use super::helpers::{source_type_for_script_lang, trim_trailing_newlines};
use super::output_module::{
    append_component_render_export, append_css_modules_assignment, finalize_output_mode,
    rewrite_client_render_for_sfc_main,
};
use super::prelude::{Outputs, Prelude, Target};
use super::{bindings, module_trace, stage_capture};

pub(super) fn compile(
    prelude: &Prelude<'_>,
    target: &Target<'_>,
    script: &SfcScriptBlock<'_>,
    Outputs {
        css,
        mut warnings,
        macro_artifacts,
    }: Outputs,
    mut capture: Option<&mut StageCapture>,
) -> Result<SfcCompileResult, SfcError> {
    let descriptor = prelude.descriptor;
    let codegen_options = prelude.codegen_options;
    let options = target.options;
    let is_vapor = target.is_vapor;
    let compiled_styles = &prelude.styles.compiled;
    let has_template = descriptor.template.is_some();
    let mut errors = Vec::new();
    let mut code = String::default();
    let (lazy_hydration_transform, script_content, script_runs) = module_trace::prepared_script(
        script,
        codegen_options.source_map,
        prelude.nuxt_page_meta,
        codegen_options.runtime_module_name.as_str(),
    );

    // The source script's grammar: TypeScript, and JSX for `tsx`/`jsx`.
    let source_type = source_type_for_script_lang(script.lang.as_deref());

    // A script-only SFC needs no mutable component binding unless the
    // compiler must attach generated metadata. Preserve its default export
    // just as compiler-sfc does, including in SSR output.
    let requires_generated_component =
        has_template || is_vapor || !compiled_styles.css_modules.is_empty();
    let (mut final_script, script_runs, needs_component_binding) = module_trace::script_module(
        &script_content,
        script_runs,
        source_type,
        prelude.is_ts,
        requires_generated_component,
    );
    // The script's position in the module, for its source-map provenance.
    let mut script_at = 0;
    if let Some(transform) = lazy_hydration_transform {
        script_at = transform.preamble.len();
        let mut script_with_preamble = transform.preamble;
        script_with_preamble.push_str(&final_script);
        final_script = script_with_preamble;
    }

    let options_api_bindings = has_template
        .then(|| bindings::options_api(&script_content, script.lang.as_deref()))
        .flatten();

    // Compile template if present
    if let Some(template) = descriptor.template.as_ref() {
        let template_allocator = vize_carton::pool::acquire();
        let template_result = if is_vapor {
            profile!(
                "atelier.sfc.template.vapor",
                compile_template_block_vapor(
                    &template_allocator,
                    template,
                    &options.template,
                    prelude.custom_elements,
                    prelude
                        .template_context
                        .block(false, options_api_bindings.as_ref(), None),
                    prelude.template_syntax,
                    codegen_options,
                    capture.as_deref_mut(),
                )
            )
        } else {
            let mut template_opts = options.template.clone();
            if prelude.auto_ssr_css_vars {
                crate::css::transform::vars::inject_ssr_values(
                    &mut template_opts,
                    &descriptor.css_vars,
                    &prelude.styles.scope_id,
                    options_api_bindings.as_ref(),
                );
            }

            // Also pass scope IDs to the client template compiler. Vue's runtime
            // normally propagates __scopeId, but wrapper components such as NuxtLink
            // can otherwise lose parent scoped attrs before the final DOM root.
            profile!(
                "atelier.sfc.template.compile",
                compile_template_block_with_capture(
                    &template_allocator,
                    template,
                    &template_opts,
                    prelude.custom_elements,
                    prelude
                        .template_context
                        .block(false, options_api_bindings.as_ref(), None),
                    prelude.template_syntax,
                    codegen_options,
                    capture.as_deref_mut(),
                )
            )
        };

        match template_result {
            Ok(template_output) => {
                warnings.extend(template_output.warnings);
                let template_code = template_output.code;
                // Build output matching Vue's compiler-sfc:
                // 1. Full template output (imports + hoisted + function _sfc_render(...))
                // 2. Rewritten script
                // 3. _sfc_main.render = _sfc_render / _sfc_main.ssrRender = ssrRender
                // 4. export default _sfc_main
                if is_vapor || options.template.ssr {
                    // Vapor / SSR keep the render block first, then the script.
                    script_at += template_code.len();
                    code.push_str(&template_code);
                    code.push_str(&final_script);
                    code.push('\n');
                } else {
                    // Client render: match @vue/compiler-sfc / plugin-vue ordering —
                    // the <script> block (and its own imports) comes first, then the
                    // template render block. This keeps user imports at the top of the
                    // module instead of after the generated render function.
                    let template_code = rewrite_client_render_for_sfc_main(&template_code);
                    code.push_str(&final_script);
                    code.push('\n');
                    code.push_str(&template_code);
                    if !code.ends_with('\n') {
                        code.push('\n');
                    }
                }

                // Export the component with render attached
                if is_vapor || options.template.vapor {
                    code.push_str("_sfc_main.__vapor = true\n");
                }
                let render = if options.template.ssr {
                    RenderFunctionName::SsrRender
                } else if is_vapor {
                    RenderFunctionName::Render
                } else {
                    RenderFunctionName::ComponentRender
                };
                append_component_render_export(
                    &mut code,
                    "_sfc_main",
                    render,
                    &compiled_styles.css_modules,
                );
            }
            Err(e) => {
                errors.push(e);
                stage_capture::unavailable(capture, "template-error");
                // Fall back to just the script
                code = final_script.clone();
                code.push('\n');
            }
        }
    } else {
        // Without generated metadata the original script already exports
        // its component; only metadata-bearing scripts need `_sfc_main`.
        code.push_str(&final_script);
        if is_vapor || options.template.vapor {
            code.push_str("\n_sfc_main.__vapor = true");
        }
        if !compiled_styles.css_modules.is_empty() {
            code.push('\n');
            append_css_modules_assignment(&mut code, "_sfc_main", &compiled_styles.css_modules);
        }
        if needs_component_binding {
            code.push_str("\nexport default _sfc_main\n");
        }
    }

    let rewrite = finalize_output_mode(&mut code, &mut warnings, options, codegen_options);
    // compiler-sfc returns the authored script body, including its final
    // newline, when no component rewrite is needed. Generated module
    // shapes still use the usual trimmed output convention.
    if needs_component_binding {
        trim_trailing_newlines(&mut code);
    }

    Ok(SfcCompileResult {
        map: module_trace::module_map(
            &code,
            script_runs,
            script_at,
            rewrite,
            descriptor,
            prelude.filename,
        ),
        code,
        css,
        errors,
        warnings,
        bindings: None,
        macro_artifacts,
        dependencies: Vec::new(),
    })
}
