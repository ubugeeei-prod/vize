//! SFC compilation implementation.
//!
//! This is the main entry point for compiling Vue Single File Components.
//! Following the Vue.js core structure, template/script/style compilation
//! is delegated to specialized modules.

mod bindings;
mod helpers;
mod normal_script;
mod styles;
#[cfg(test)]
mod tests;

use crate::compile_script::artifacts::{erase_artifact_macro_statements, extract_macro_artifacts};
use crate::compile_script::lazy_hydration::transform_lazy_hydration_macros;
use crate::compile_script::{TemplateParts, compile_script_setup_inline_with_context};
use crate::compile_template::{
    TemplateBlockCompileContext, compile_template_block, compile_template_block_vapor,
    extract_template_parts, extract_template_parts_full,
};
use crate::rewrite_default::rewrite_default;
use crate::script::ScriptCompileContext;
use crate::types::{
    BindingType, SfcCompileOptions, SfcCompileResult, SfcDescriptor, SfcError, SfcMacroArtifact,
};
use vize_atelier_core::TemplateSyntaxMode;

use self::bindings::{
    collect_normal_script_bindings, croquis_to_legacy_bindings, merge_normal_script_bindings,
};
use self::helpers::{
    demote_v_model_reactive_const_bindings, extract_component_name, generate_scope_id,
};
use self::normal_script::extract_normal_script_content;
use self::styles::compile_styles;

// Re-export ScriptCompileResult for public API
pub use crate::compile_script::ScriptCompileResult;
use vize_carton::{String, ToCompactString, cstr, profile};

fn create_vapor_ssr_fallback_warning(descriptor: &SfcDescriptor) -> SfcError {
    SfcError {
        message: "SFC Vapor SSR is not supported yet; falling back to standard SSR output."
            .to_compact_string(),
        code: Some("VAPOR_SSR_FALLBACK".to_compact_string()),
        loc: descriptor
            .template
            .as_ref()
            .map(|template| template.loc.clone()),
    }
}

fn create_v_model_reactive_const_warning(
    script_setup: &crate::types::SfcScriptBlock<'_>,
    binding_name: &str,
) -> SfcError {
    let mut message = String::from("`v-model` cannot update the const reactive binding `");
    message.push_str(binding_name);
    message.push_str("`. The compiler transformed it to `let` so the update can work.");

    SfcError {
        message,
        code: Some("V_MODEL_CONST_REACTIVE_DEMOTED".to_compact_string()),
        loc: Some(script_setup.loc.clone()),
    }
}

fn create_standalone_import_warning() -> SfcError {
    SfcError {
        message: "Standalone SFC output still contains non-Vue ES module imports; CDN evaluation requires those dependencies to be provided separately."
            .to_compact_string(),
        code: Some("STANDALONE_EXTERNAL_IMPORT".to_compact_string()),
        loc: None,
    }
}

fn is_ts_lang(lang: Option<&str>) -> bool {
    matches!(lang, Some("ts" | "tsx"))
}

fn rewrite_client_render_for_sfc_main(template_code: &str) -> String {
    if template_code.contains("export function render(") {
        return template_code
            .replacen("export function render(", "function _sfc_render(", 1)
            .to_compact_string();
    }

    if template_code.contains("function render(") {
        return template_code
            .replacen("function render(", "function _sfc_render(", 1)
            .to_compact_string();
    }

    template_code.to_compact_string()
}

fn extract_descriptor_macro_artifacts(descriptor: &SfcDescriptor) -> Vec<SfcMacroArtifact> {
    let mut artifacts = Vec::new();

    if let Some(script) = descriptor.script.as_ref() {
        artifacts.extend(extract_macro_artifacts(&script.content, script.loc.start));
    }
    if let Some(script_setup) = descriptor.script_setup.as_ref() {
        artifacts.extend(extract_macro_artifacts(
            &script_setup.content,
            script_setup.loc.start,
        ));
    }

    artifacts.sort_by_key(|artifact| artifact.start);
    artifacts
}

fn trim_trailing_newlines(code: &mut String) {
    while code.ends_with('\n') {
        code.pop();
    }
}

fn runtime_module_name(_options: &SfcCompileOptions) -> &str {
    "vue"
}

fn runtime_global_name(_options: &SfcCompileOptions) -> &str {
    "Vue"
}

fn rewrite_runtime_import_line(
    trimmed: &str,
    runtime_module_name: &str,
    runtime_global_name: &str,
) -> Option<String> {
    let rest = trimmed.strip_prefix("import {")?;
    let (specifiers, rest) = rest.split_once("} from ")?;
    let source = rest.trim().trim_end_matches(';');
    let expected_double = cstr!("\"{runtime_module_name}\"");
    let expected_single = cstr!("'{runtime_module_name}'");
    if source != expected_double && source != expected_single {
        return None;
    }

    let bindings: Vec<_> = specifiers
        .split(',')
        .filter_map(|specifier| {
            let specifier = specifier.trim();
            let specifier = specifier.strip_prefix("type ").unwrap_or(specifier).trim();
            if specifier.is_empty() {
                return None;
            }

            if let Some((imported, local)) = specifier.split_once(" as ") {
                Some(cstr!("{}: {}", imported.trim(), local.trim()))
            } else {
                Some(specifier.to_compact_string())
            }
        })
        .collect();

    if bindings.is_empty() {
        return Some(String::default());
    }

    let mut joined = String::default();
    for (index, binding) in bindings.iter().enumerate() {
        if index > 0 {
            joined.push_str(", ");
        }
        joined.push_str(binding);
    }

    Some(cstr!("const {{ {} }} = {}", joined, runtime_global_name))
}

fn rewrite_module_sfc_to_standalone(
    code: &str,
    runtime_module_name: &str,
    runtime_global_name: &str,
) -> (String, bool) {
    let mut output = String::with_capacity(code.len());
    let mut has_external_imports = false;

    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("import ") {
            if let Some(rewritten) =
                rewrite_runtime_import_line(trimmed, runtime_module_name, runtime_global_name)
            {
                if !rewritten.is_empty() {
                    output.push_str(&rewritten);
                    output.push('\n');
                }
                continue;
            }
            has_external_imports = true;
        }

        let mut rewritten = line
            .replace("export function render(", "function render(")
            .replace("export function ssrRender(", "function ssrRender(");
        if let Some(index) = rewritten.find("export default")
            && rewritten[..index].trim().is_empty()
        {
            rewritten.replace_range(index..index + "export default".len(), "return");
        }
        output.push_str(&rewritten);
        output.push('\n');
    }

    (output, has_external_imports)
}

fn finalize_output_mode(
    code: &mut String,
    warnings: &mut Vec<SfcError>,
    options: &SfcCompileOptions,
) {
    if !options.script.inline_template {
        return;
    }

    let (rewritten, has_external_imports) = rewrite_module_sfc_to_standalone(
        code,
        runtime_module_name(options),
        runtime_global_name(options),
    );
    *code = rewritten;

    if has_external_imports {
        warnings.push(create_standalone_import_warning());
    }
}

/// Compile an SFC descriptor into JavaScript and CSS
pub fn compile_sfc(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
) -> Result<SfcCompileResult, SfcError> {
    compile_sfc_inner(descriptor, options, TemplateSyntaxMode::Standard)
}

/// Compile an SFC descriptor with Vue parser quirk compatibility.
#[deprecated(note = "use compile_sfc_with_template_syntax instead")]
pub fn compile_sfc_with_vue_parser_quirks(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
) -> Result<SfcCompileResult, SfcError> {
    compile_sfc_inner(descriptor, options, TemplateSyntaxMode::Quirks)
}

/// Compile an SFC descriptor with an explicit template syntax mode.
#[doc(hidden)]
pub fn compile_sfc_with_template_syntax(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
) -> Result<SfcCompileResult, SfcError> {
    compile_sfc_inner(descriptor, options, template_syntax)
}

fn compile_sfc_inner(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
) -> Result<SfcCompileResult, SfcError> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut code = String::default();
    let mut css = None;
    let macro_artifacts = extract_descriptor_macro_artifacts(descriptor);

    let filename = if options.parse.filename.is_empty() {
        options.script.id.as_deref().unwrap_or("anonymous.vue")
    } else {
        options.parse.filename.as_str()
    };
    let source_filename = options.script.id.as_deref().unwrap_or(filename);

    let has_styles = !descriptor.styles.is_empty();
    let has_scoped = descriptor.styles.iter().any(|s| s.scoped);
    // Use externally-provided scope ID if available, otherwise generate from filename.
    // The external scope ID ensures consistency with JS-side SHA-256 generation.
    // Template/script-only SFCs do not need the hash.
    let needs_scope_id =
        has_styles || !descriptor.css_vars.is_empty() || options.scope_id.is_some();
    let scope_id = if needs_scope_id {
        options
            .scope_id
            .clone()
            .unwrap_or_else(|| generate_scope_id(filename))
    } else {
        String::default()
    };

    let vapor_requested = options.vapor
        || descriptor
            .script_setup
            .as_ref()
            .map(|s| s.attrs.contains_key("vapor"))
            .unwrap_or(false)
        || descriptor
            .script
            .as_ref()
            .map(|s| s.attrs.contains_key("vapor"))
            .unwrap_or(false);

    // Vapor components currently render on the client. For SSR we fall back to
    // the standard VDOM compiler and let the client hydrate with Vapor output.
    if descriptor.template.is_some() && options.template.ssr && vapor_requested {
        warnings.push(create_vapor_ssr_fallback_warning(descriptor));
    }
    let is_vapor = !options.template.ssr && vapor_requested;

    // is_ts controls output format:
    // - true: output TypeScript (add `: any` annotations, defineComponent wrapper)
    // - false: output JavaScript (strip TypeScript syntax from TS sources)
    // Source language detection is tracked separately in the script/setup branches below.
    let is_ts = options.script.is_ts || options.template.is_ts;
    let template_is_ts = options.template.is_ts
        || descriptor
            .script_setup
            .as_ref()
            .is_some_and(|s| is_ts_lang(s.lang.as_deref()))
        || descriptor
            .script
            .as_ref()
            .is_some_and(|s| is_ts_lang(s.lang.as_deref()));

    // Extract component name from filename
    let component_name = extract_component_name(filename);

    // Determine output mode based on script type
    let has_script_setup = descriptor.script_setup.is_some();
    let has_script = descriptor.script.is_some();
    let has_template = descriptor.template.is_some();

    // Case 1: Template only - just output render function
    if !has_script && !has_script_setup && has_template {
        let template = descriptor.template.as_ref().unwrap();
        let template_result = if is_vapor {
            profile!(
                "atelier.sfc.template.vapor",
                compile_template_block_vapor(
                    template,
                    &scope_id,
                    has_scoped,
                    None,
                    options.template.custom_renderer,
                    template_syntax,
                )
            )
        } else {
            // Enable hoisting for template-only SFCs (hoisted consts go at module level)
            let mut template_opts = options.template.clone();
            let mut dom_opts = template_opts.compiler_options.take().unwrap_or_default();
            dom_opts.hoist_static = true;
            template_opts.compiler_options = Some(dom_opts);
            // Also pass scope IDs to the client template compiler. Vue's runtime
            // normally propagates __scopeId, but wrapper components such as NuxtLink
            // can otherwise lose parent scoped attrs before the final DOM root.
            profile!(
                "atelier.sfc.template.compile",
                compile_template_block(
                    template,
                    &template_opts,
                    TemplateBlockCompileContext {
                        scope_id: &scope_id,
                        apply_scope_id: has_scoped,
                        has_scoped,
                        is_ts: template_is_ts,
                        inline: false,
                        component_name: Some(&component_name),
                        bindings: None,
                        croquis: None,
                    },
                    template_syntax,
                )
            )
        };

        match template_result {
            Ok(template_output) => {
                warnings.extend(template_output.warnings);
                code = template_output.code;
                if is_vapor {
                    code.push_str("const _sfc_main = { __vapor: true }\n");
                    code.push_str("_sfc_main.render = render\n");
                    code.push_str("export default _sfc_main\n");
                } else if options.template.ssr {
                    code.push_str("const _sfc_main = {}\n");
                    code.push_str("_sfc_main.ssrRender = ssrRender\n");
                    code.push_str("export default _sfc_main\n");
                }
            }
            // Previously this just collected the error into a local vec
            // and continued, returning Ok with empty code — so callers
            // wrote a 0-byte module and exited 0 (#958). Propagate the
            // template error up so the build/CLI surfaces it.
            Err(e) => return Err(e),
        }

        // Compile styles
        let all_css = profile!(
            "atelier.sfc.styles",
            compile_styles(&descriptor.styles, &scope_id, &options.style, &mut warnings)
        );
        if !all_css.is_empty() {
            css = Some(all_css);
        }

        finalize_output_mode(&mut code, &mut warnings, &options);
        trim_trailing_newlines(&mut code);

        return Ok(SfcCompileResult {
            code,
            css,
            map: None,
            errors,
            warnings,
            bindings: None,
            macro_artifacts,
        });
    }

    // Case 2: Script (non-setup) + Template - rewrite default and compile template
    if has_script && !has_script_setup {
        let script = descriptor.script.as_ref().unwrap();
        let lazy_hydration_transform = transform_lazy_hydration_macros(&script.content);
        let script_source = lazy_hydration_transform
            .as_ref()
            .map(|result| result.code.as_str())
            .unwrap_or(&script.content);
        let script_content = erase_artifact_macro_statements(script_source)
            .unwrap_or_else(|| script_source.to_compact_string());

        // Check if source script is TypeScript
        let source_is_ts = script
            .lang
            .as_ref()
            .is_some_and(|l| l == "ts" || l == "tsx");

        // Rewrite `export default` to `const _sfc_main = ...`
        // Parse as TypeScript if source is TypeScript
        let (rewritten_script, _has_default) = profile!(
            "atelier.sfc.normal_script.rewrite_default",
            rewrite_default(&script_content, "_sfc_main", source_is_ts)
        );

        // Transpile TypeScript to JavaScript if needed
        let mut final_script = if source_is_ts && !is_ts {
            profile!(
                "atelier.sfc.normal_script.ts_to_js",
                crate::compile_script::typescript::transform_typescript_to_js(&rewritten_script)
            )
        } else {
            rewritten_script
        };
        if let Some(transform) = lazy_hydration_transform {
            let mut script_with_preamble = transform.preamble;
            script_with_preamble.push_str(&final_script);
            final_script = script_with_preamble;
        }

        // Compile template if present
        if has_template {
            let template = descriptor.template.as_ref().unwrap();
            let template_result = if is_vapor {
                profile!(
                    "atelier.sfc.template.vapor",
                    compile_template_block_vapor(
                        template,
                        &scope_id,
                        has_scoped,
                        None,
                        options.template.custom_renderer,
                        template_syntax,
                    )
                )
            } else {
                let mut template_opts = options.template.clone();
                let mut dom_opts = template_opts.compiler_options.take().unwrap_or_default();
                dom_opts.hoist_static = true;
                template_opts.compiler_options = Some(dom_opts);

                // Also pass scope IDs to the client template compiler. Vue's runtime
                // normally propagates __scopeId, but wrapper components such as NuxtLink
                // can otherwise lose parent scoped attrs before the final DOM root.
                profile!(
                    "atelier.sfc.template.compile",
                    compile_template_block(
                        template,
                        &template_opts,
                        TemplateBlockCompileContext {
                            scope_id: &scope_id,
                            apply_scope_id: has_scoped,
                            has_scoped,
                            is_ts: template_is_ts,
                            inline: false,
                            component_name: Some(&component_name),
                            bindings: None,
                            croquis: None,
                        },
                        template_syntax,
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
                        code.push_str(&template_code);
                    } else {
                        let template_code = rewrite_client_render_for_sfc_main(&template_code);
                        code.push_str(&template_code);
                    }
                    code.push_str(&final_script);
                    code.push('\n');

                    // Export the component with render attached
                    if is_vapor {
                        code.push_str("_sfc_main.__vapor = true\n");
                    }
                    if options.template.ssr {
                        code.push_str("_sfc_main.ssrRender = ssrRender\n");
                    } else if is_vapor {
                        code.push_str("_sfc_main.render = render\n");
                    } else {
                        code.push_str("_sfc_main.render = _sfc_render\n");
                    }
                    code.push_str("export default _sfc_main\n");
                }
                Err(e) => {
                    errors.push(e);
                    // Fall back to just the script
                    code = final_script.clone();
                    code.push('\n');
                }
            }
        } else {
            // No template - just output rewritten script and export
            code.push_str(&final_script);
            if is_vapor {
                code.push_str("\n_sfc_main.__vapor = true");
            }
            code.push_str("\nexport default _sfc_main\n");
        }

        // Compile styles
        let all_css = profile!(
            "atelier.sfc.styles",
            compile_styles(&descriptor.styles, &scope_id, &options.style, &mut warnings)
        );
        if !all_css.is_empty() {
            css = Some(all_css);
        }

        finalize_output_mode(&mut code, &mut warnings, &options);
        trim_trailing_newlines(&mut code);

        return Ok(SfcCompileResult {
            code,
            css,
            map: None,
            errors,
            warnings,
            bindings: None,
            macro_artifacts,
        });
    }

    // Case 3: Script setup with inline template
    // If we reach here without script_setup, it means the SFC has no content
    let script_setup = match descriptor.script_setup.as_ref() {
        Some(s) => s,
        None => {
            return Err(SfcError {
                message:
                    "At least one <template> or <script> is required in a single file component."
                        .to_compact_string(),
                code: None,
                loc: None,
            });
        }
    };

    // Extract normal script content if present (for type definitions, imports, etc.)
    // When both <script> and <script setup> exist, normal script content should be preserved
    // (except for export default which is handled by script setup)
    let normal_script_content = if has_script {
        let script = descriptor.script.as_ref().unwrap();
        // Check if source is TypeScript
        let source_is_ts = script
            .lang
            .as_ref()
            .is_some_and(|l| l == "ts" || l == "tsx");
        Some(profile!(
            "atelier.sfc.normal_script.extract",
            extract_normal_script_content(&script.content, source_is_ts, is_ts)
        ))
    } else {
        None
    };

    let lazy_hydration_transform = transform_lazy_hydration_macros(&script_setup.content);
    let mut script_setup_content = lazy_hydration_transform
        .as_ref()
        .map(|result| result.code.clone())
        .unwrap_or_else(|| script_setup.content.to_compact_string());

    // 1. Croquis parser: rich analysis with ReactivityTracker
    let mut croquis = profile!(
        "atelier.sfc.script_setup.croquis",
        crate::script::analyze_script_setup_to_summary(&script_setup_content)
    );
    let mut script_bindings = croquis_to_legacy_bindings(&croquis.bindings);

    // 2. ScriptCompileContext: needed for macro span info and TypeScript type resolution
    //    (Croquis doesn't resolve type references like `defineProps<Props>()`)
    let mut ctx = profile!(
        "atelier.sfc.script_context.new",
        ScriptCompileContext::new(&script_setup_content)
    );

    // Merge type definitions from normal <script> block so that
    // defineProps<TypeRef>() can resolve types defined there.
    if has_script {
        let script = descriptor.script.as_ref().unwrap();
        profile!(
            "atelier.sfc.script_context.collect_normal_types",
            ctx.collect_types_from(&script.content)
        );
    }
    profile!(
        "atelier.sfc.script_context.collect_setup_import_types",
        ctx.collect_imported_types_from_path(&script_setup_content, source_filename)
    );
    if has_script {
        let script = descriptor.script.as_ref().unwrap();
        profile!(
            "atelier.sfc.script_context.collect_normal_import_types",
            ctx.collect_imported_types_from_path(&script.content, source_filename)
        );
    }
    profile!("atelier.sfc.script_context.analyze", ctx.analyze());

    // 3. Merge Props bindings from ScriptCompileContext (type resolution fallback)
    //    Croquis can't resolve interface references, so we take Props from the legacy analyzer
    for (name, bt) in &ctx.bindings.bindings {
        if matches!(bt, BindingType::Props | BindingType::PropsAliased) {
            script_bindings.bindings.entry(name.clone()).or_insert(*bt);
        }
    }
    for (local, key) in &ctx.bindings.props_aliases {
        script_bindings
            .props_aliases
            .entry(local.clone())
            .or_insert_with(|| key.clone());
    }

    // Register $emit or __emit binding when defineEmits is used, so the template
    // compiler knows not to prefix it with _ctx.
    if let Some(ref emits_macro) = ctx.macros.define_emits {
        if let Some(ref binding_name) = emits_macro.binding_name {
            // e.g., const emit = defineEmits([...]) -> emit is setup const
            script_bindings
                .bindings
                .entry(binding_name.clone())
                .or_insert(BindingType::SetupConst);
        } else {
            // defineEmits([...]) without assignment -> $emit is exposed in setup args
            script_bindings
                .bindings
                .entry("$emit".to_compact_string())
                .or_insert(BindingType::SetupConst);
        }
    }

    // Register bindings from normal <script> block.
    // When both <script> and <script setup> exist, top-level imports and
    // declarations from the normal script are accessible in the template.
    // This enables proper component resolution (e.g., `import { Form as PForm }`)
    // and identifier prefix resolution (avoiding incorrect `_ctx.` prefix).
    if has_script {
        let script = descriptor.script.as_ref().unwrap();
        let normal_script_bindings = profile!(
            "atelier.sfc.normal_script.register_bindings",
            collect_normal_script_bindings(&script.content)
        );
        merge_normal_script_bindings(&mut script_bindings, &normal_script_bindings);
        merge_normal_script_bindings(&mut ctx.bindings, &normal_script_bindings);
    }

    if let Some(template) = &descriptor.template {
        let demoted_ids = profile!(
            "atelier.sfc.script_setup.demote_v_model_reactive_consts",
            demote_v_model_reactive_const_bindings(
                &template.content,
                script_setup.lang.as_deref(),
                &mut script_setup_content,
                &mut ctx,
                &mut script_bindings,
                &mut croquis,
            )
        );

        for binding_name in demoted_ids {
            warnings.push(create_v_model_reactive_const_warning(
                script_setup,
                &binding_name,
            ));
        }
    }

    let source_is_ts = script_setup
        .lang
        .as_ref()
        .is_some_and(|l| l == "ts" || l == "tsx");

    // Compile template with bindings (if present) to get the render function
    let template_result = if let Some(template) = &descriptor.template {
        if is_vapor {
            Some(profile!(
                "atelier.sfc.template.vapor",
                compile_template_block_vapor(
                    template,
                    &scope_id,
                    has_scoped,
                    Some(&script_bindings),
                    options.template.custom_renderer,
                    template_syntax,
                )
            ))
        } else {
            // Also pass scope IDs to the client template compiler. Vue's runtime
            // normally propagates __scopeId, but wrapper components such as NuxtLink
            // can otherwise lose parent scoped attrs before the final DOM root.
            Some(profile!(
                "atelier.sfc.template.compile",
                compile_template_block(
                    template,
                    &options.template,
                    TemplateBlockCompileContext {
                        scope_id: &scope_id,
                        apply_scope_id: has_scoped,
                        has_scoped,
                        is_ts: template_is_ts,
                        inline: true,
                        component_name: Some(&component_name),
                        bindings: Some(&script_bindings),
                        croquis: Some(croquis),
                    },
                    template_syntax,
                )
            ))
        }
    } else {
        None
    };

    if let Some(Ok(template_output)) = &template_result {
        warnings.extend(template_output.warnings.clone());
    }

    // Extract template parts for inline mode (imports, hoisted, preamble, render_body)
    let (
        template_imports,
        template_hoisted,
        template_render_fn,
        template_render_fn_name,
        template_preamble,
        render_body,
    ) = match &template_result {
        Some(Ok(template_output)) => {
            let template_code = &template_output.code;
            if is_vapor || options.template.ssr {
                let (imports, hoisted, render_fn, render_fn_name) = profile!(
                    "atelier.sfc.template.extract_parts_full",
                    extract_template_parts_full(template_code)
                );
                (
                    imports,
                    hoisted,
                    render_fn,
                    render_fn_name,
                    String::default(),
                    String::default(),
                )
            } else {
                let (imports, hoisted, preamble, body, render_fn_name) = profile!(
                    "atelier.sfc.template.extract_parts",
                    extract_template_parts(template_code)
                );
                (
                    imports,
                    hoisted,
                    String::default(),
                    render_fn_name,
                    preamble,
                    body,
                )
            }
        }
        Some(Err(e)) => {
            errors.push(e.clone());
            (
                String::default(),
                String::default(),
                String::default(),
                "",
                String::default(),
                String::default(),
            )
        }
        None => (
            String::default(),
            String::default(),
            String::default(),
            "",
            String::default(),
            String::default(),
        ),
    };

    // Compile script setup using inline mode to match Vue's @vue/compiler-sfc output format:
    // 1. Template imports (from "vue")
    // 2. User imports
    // 3. Hoisted literal consts (module-level)
    // 4. export default { __name, props?, emits?, setup(__props) { ... return (_ctx, _cache) => { ... } } }
    let script_result = profile!(
        "atelier.sfc.script_setup.inline_compile",
        compile_script_setup_inline_with_context(
            ctx,
            &script_setup_content,
            &component_name,
            is_ts,
            source_is_ts,
            is_vapor,
            TemplateParts {
                imports: &template_imports,
                hoisted: &template_hoisted,
                render_fn: &template_render_fn,
                render_fn_name: template_render_fn_name,
                preamble: &template_preamble,
                render_body: &render_body,
                render_is_block: is_vapor,
            },
            normal_script_content.as_deref(),
            &descriptor.css_vars,
            &scope_id,
            filename,
            options.template.is_prod,
        )
    )?;

    // The inline mode compile_script_setup_inline generates a complete output
    // including imports, hoisted vars, and `export default { ... }` with inline render
    if let Some(transform) = lazy_hydration_transform {
        code.push_str(&transform.preamble);
    }
    code.push_str(&script_result.code);

    // Compile styles
    let all_css = profile!(
        "atelier.sfc.styles",
        compile_styles(&descriptor.styles, &scope_id, &options.style, &mut warnings)
    );
    if !all_css.is_empty() {
        css = Some(all_css);
    }

    finalize_output_mode(&mut code, &mut warnings, &options);
    trim_trailing_newlines(&mut code);

    Ok(SfcCompileResult {
        code,
        css,
        map: None,
        errors,
        warnings,
        bindings: script_result.bindings,
        macro_artifacts,
    })
}
