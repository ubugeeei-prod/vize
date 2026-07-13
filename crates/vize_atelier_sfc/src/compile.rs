//! SFC compilation implementation.
//!
//! This is the main entry point for compiling Vue Single File Components.
//! Following the Vue.js core structure, template/script/style compilation
//! is delegated to specialized modules.

mod bindings;
mod empty_component;
pub(crate) mod fallbacks;
mod graph;
mod helpers;
mod normal_script;
pub(crate) mod output_module;
mod shared_syntax;
mod source_maps;
mod styles;
#[cfg(test)]
mod tests;

use crate::compile_script::artifacts::{erase_artifact_macro_statements, extract_macro_artifacts};
use crate::compile_script::lazy_hydration::transform_lazy_hydration_macros;
use crate::compile_script::props::is_valid_identifier;
use crate::compile_script::{TemplateParts, compile_script_setup_inline_with_context};
use crate::compile_template::{TemplateBlockCompileContext, VaporTemplateCompileContext};
use crate::rewrite_default::rewrite_default;
use crate::script::ScriptCompileContext;
use crate::types::{
    BindingMetadata, BindingType, SfcCompileOptions, SfcCompileResult, SfcDescriptor, SfcError,
    SfcMacroArtifact,
};
use vize_relief::CodegenOptions;
use vize_relief::TemplateSyntaxMode;

use self::bindings::{
    collect_normal_script_bindings, croquis_to_legacy_bindings, merge_normal_script_bindings,
};
use self::fallbacks::apply_vapor_ssr_fallback;
pub(crate) use self::graph::{
    GraphRenderMapping, GraphRenderModule, compile_sfc_with_graph_render,
};
use self::helpers::{
    demote_v_model_reactive_const_bindings, extract_component_name, generate_scope_id,
};
use self::normal_script::extract_normal_script_content;
pub(crate) use self::normal_script::extract_normal_script_content_from_program;
use self::output_module::{
    RenderFunctionName, append_component_render_export, append_css_modules_assignment,
    finalize_output_mode, rewrite_client_render_for_sfc_main,
};
pub use self::shared_syntax::compile_sfc_with_shared_syntax;
use self::shared_syntax::{compile_template_block, compile_template_block_vapor};
use self::source_maps::{
    SourceMapComposition, compose_template_source_map, record_template_source_map_fact,
};
use self::styles::compile_styles;

// Re-export ScriptCompileResult for public API
pub use crate::compile_script::ScriptCompileResult;
use vize_carton::{String, ToCompactString, profile};

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

pub(crate) fn is_ts_lang(lang: Option<&str>) -> bool {
    matches!(lang, Some("ts" | "tsx"))
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

/// Compile an SFC descriptor into JavaScript and CSS
pub fn compile_sfc(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
) -> Result<SfcCompileResult, SfcError> {
    compile_sfc_inner(
        descriptor,
        options,
        TemplateSyntaxMode::Standard,
        None,
        CodegenOptions::default(),
    )
}

/// Compile an SFC descriptor with Vue parser quirk compatibility.
#[deprecated(note = "use compile_sfc_with_template_syntax instead")]
pub fn compile_sfc_with_vue_parser_quirks(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
) -> Result<SfcCompileResult, SfcError> {
    compile_sfc_inner(
        descriptor,
        options,
        TemplateSyntaxMode::Quirks,
        None,
        CodegenOptions::default(),
    )
}

/// Compile an SFC descriptor with an explicit template syntax mode.
#[doc(hidden)]
pub fn compile_sfc_with_template_syntax(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
) -> Result<SfcCompileResult, SfcError> {
    compile_sfc_inner(
        descriptor,
        options,
        template_syntax,
        None,
        CodegenOptions::default(),
    )
}

/// Compile an SFC with adapter-provided codegen defaults.
///
/// This keeps emission-only binding settings out of the public SFC/DOM option
/// structs while reusing the compiler core's canonical [`CodegenOptions`].
#[doc(hidden)]
pub fn compile_sfc_with_template_syntax_and_codegen_options(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
    codegen_options: CodegenOptions,
) -> Result<SfcCompileResult, SfcError> {
    compile_sfc_inner(descriptor, options, template_syntax, None, codegen_options)
}

fn compile_sfc_inner(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
    template_syntax: TemplateSyntaxMode,
    shared_syntax: Option<&vize_relief::ReliefArtifact>,
    codegen_options: CodegenOptions,
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
    // External scope IDs preserve JS-side SHA-256 consistency; otherwise derive one when needed.
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

    let compiled_styles = profile!(
        "atelier.sfc.styles",
        compile_styles(&descriptor.styles, &scope_id, &options.style, &mut warnings)
    );
    if !compiled_styles.css.is_empty() {
        css = Some(compiled_styles.css.clone());
    }

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

    // SSR falls back to VDOM before the client hydrates with Vapor output.
    if descriptor.template.is_some() {
        apply_vapor_ssr_fallback(descriptor, &options, vapor_requested, &mut warnings);
    }
    let is_vapor = !options.template.ssr && vapor_requested;

    // Output TypeScript when either script or template compilation requests it.
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

    let component_name = extract_component_name(filename);
    let vapor_context = VaporTemplateCompileContext {
        scope_id: &scope_id,
        has_scoped,
        options: &options.template,
        template_syntax,
        codegen_options: &codegen_options,
    };

    let has_script_setup = descriptor.script_setup.is_some();
    let has_script = descriptor.script.is_some();
    let has_template = descriptor.template.is_some();

    if !has_script && !has_script_setup && has_template {
        let template = descriptor.template.as_ref().unwrap();
        let source_map;
        let template_result = if is_vapor {
            profile!(
                "atelier.sfc.template.vapor",
                compile_template_block_vapor(template, vapor_context, None, shared_syntax)
            )
        } else {
            let mut template_opts = options.template.clone();
            let mut dom_opts = template_opts.compiler_options.take().unwrap_or_default();
            dom_opts.hoist_static = true;
            template_opts.compiler_options = Some(dom_opts);
            // Preserve scoped attrs across wrappers such as NuxtLink.
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
                    shared_syntax,
                    &codegen_options,
                )
            )
        };

        match template_result {
            Ok(template_output) => {
                source_map = record_template_source_map_fact(
                    &template_output,
                    SourceMapComposition::Composed,
                );
                warnings.extend(template_output.warnings);
                code = template_output.code;
                if is_vapor {
                    code.push_str("const _sfc_main = { __vapor: true }\n");
                    append_component_render_export(
                        &mut code,
                        "_sfc_main",
                        RenderFunctionName::Render,
                        &compiled_styles.css_modules,
                    );
                } else if options.template.ssr {
                    code.push_str("const _sfc_main = {}\n");
                    append_component_render_export(
                        &mut code,
                        "_sfc_main",
                        RenderFunctionName::SsrRender,
                        &compiled_styles.css_modules,
                    );
                } else if !compiled_styles.css_modules.is_empty() {
                    code.push_str("const _sfc_main = {}\n");
                    append_component_render_export(
                        &mut code,
                        "_sfc_main",
                        RenderFunctionName::Render,
                        &compiled_styles.css_modules,
                    );
                }
            }
            // Previously this just collected the error into a local vec
            // and continued, returning Ok with empty code — so callers
            // wrote a 0-byte module and exited 0 (#958). Propagate the
            // template error up so the build/CLI surfaces it.
            Err(e) => return Err(e),
        }

        finalize_output_mode(&mut code, &mut warnings, &options, &codegen_options);
        trim_trailing_newlines(&mut code);
        let map = compose_template_source_map(source_map, &code, descriptor, filename);

        return Ok(SfcCompileResult {
            code,
            css,
            map,
            errors,
            warnings,
            bindings: None,
            macro_artifacts,
        });
    }

    if has_script && !has_script_setup {
        let script = descriptor.script.as_ref().unwrap();
        let lazy_hydration_transform = transform_lazy_hydration_macros(&script.content);
        let script_source = lazy_hydration_transform
            .as_ref()
            .map(|result| result.code.as_str())
            .unwrap_or(&script.content);
        let script_content = erase_artifact_macro_statements(script_source)
            .unwrap_or_else(|| script_source.to_compact_string());

        let source_is_ts = is_ts_lang(script.lang.as_deref());

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

        // Resolve Options API template bindings (data / computed / methods /
        // props / inject) from the plain `<script>` so the template compiler can
        // emit the render-function prefixes Vue's compiler-sfc uses
        // (`$data.`, `$options.`, `$props.`) instead of falling back to `_ctx.`
        // for everything. This mirrors `@vue/compiler-sfc`, which feeds the
        // analyzed Options API bindingMetadata to template codegen.
        //
        // Only the Options API member kinds are forwarded. `@vue/compiler-sfc`
        // does NOT register top-level imports, module-local consts, or
        // `components: {}` registrations from a non-`<script setup>` block as
        // template bindings — Croquis assigns those `SetupConst`/`LiteralConst`,
        // which would otherwise rewrite locally-registered components to
        // `$setup.Foo` instead of leaving them for `_resolveComponent("Foo")`.
        let options_api_bindings: Option<BindingMetadata> = if has_template {
            // Parse the `<script>` once for Options API binding extraction only —
            // this lighter `parse_script_with_options` path skips the full
            // template/reactivity Croquis analysis the `<script setup>` path runs.
            let parsed = profile!(
                "atelier.sfc.normal_script.options_api_bindings",
                vize_croquis::script_parser::parse_script_with_options_and_jsx(
                    &script_content,
                    vize_croquis::script_parser::ScriptParserOptions {
                        options_api: true,
                        legacy_vue2: false,
                    },
                    script
                        .lang
                        .as_deref()
                        .is_some_and(|lang| matches!(lang.trim(), "tsx" | "jsx")),
                )
            );
            let mut bindings = BindingMetadata::default();
            for (name, bt) in parsed.bindings.iter() {
                let bt: BindingType = bt.into();
                // Forward only the unambiguous Options API member kinds. Croquis
                // assigns `SetupConst`/`LiteralConst` to top-level imports and
                // module-local consts and `SetupMaybeRef` to `setup()` returns —
                // none of which `@vue/compiler-sfc` registers as template
                // bindings for a non-`<script setup>` block (forwarding them would
                // rewrite locally-registered components to `$setup.Foo` instead of
                // leaving them for `_resolveComponent("Foo")`).
                if matches!(
                    bt,
                    BindingType::Data
                        | BindingType::Options
                        | BindingType::Props
                        | BindingType::PropsAliased
                ) {
                    bindings.bindings.insert(name.to_compact_string(), bt);
                }
            }
            for (local, key) in &parsed.bindings.props_aliases {
                bindings
                    .props_aliases
                    .insert(local.to_compact_string(), key.to_compact_string());
            }
            (!bindings.bindings.is_empty()).then_some(bindings)
        } else {
            None
        };

        let mut source_map = None;
        if has_template {
            let template = descriptor.template.as_ref().unwrap();
            let template_result = if is_vapor {
                profile!(
                    "atelier.sfc.template.vapor",
                    compile_template_block_vapor(template, vapor_context, None, shared_syntax)
                )
            } else {
                let mut template_opts = options.template.clone();
                let mut dom_opts = template_opts.compiler_options.take().unwrap_or_default();
                dom_opts.hoist_static = true;
                template_opts.compiler_options = Some(dom_opts);

                // Preserve scoped attrs across wrappers such as NuxtLink.
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
                            bindings: options_api_bindings.as_ref(),
                            croquis: None,
                        },
                        template_syntax,
                        shared_syntax,
                        &codegen_options,
                    )
                )
            };

            match template_result {
                Ok(template_output) => {
                    source_map = record_template_source_map_fact(
                        &template_output,
                        SourceMapComposition::Skipped,
                    );
                    warnings.extend(template_output.warnings);
                    let template_code = template_output.code;
                    // Match compiler-sfc: template module, script, render attachment, export.
                    if is_vapor || options.template.ssr {
                        // Vapor / SSR keep the render block first, then the script.
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
                    if is_vapor {
                        code.push_str("_sfc_main.__vapor = true\n");
                    }
                    let render = if options.template.ssr {
                        RenderFunctionName::SsrRender
                    } else if is_vapor {
                        RenderFunctionName::Render
                    } else {
                        RenderFunctionName::SfcRender
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
            if !compiled_styles.css_modules.is_empty() {
                code.push('\n');
                append_css_modules_assignment(&mut code, "_sfc_main", &compiled_styles.css_modules);
            }
            code.push_str("\nexport default _sfc_main\n");
        }

        finalize_output_mode(&mut code, &mut warnings, &options, &codegen_options);
        trim_trailing_newlines(&mut code);
        let map = compose_template_source_map(source_map, &code, descriptor, filename);

        return Ok(SfcCompileResult {
            code,
            css,
            map,
            errors,
            warnings,
            bindings: None,
            macro_artifacts,
        });
    }

    let Some(script_setup) = descriptor.script_setup.as_ref() else {
        return Ok(empty_component::compile_empty_component(
            empty_component::EmptyComponentContext {
                is_vapor,
                compiled_styles: &compiled_styles,
                css,
                errors,
                warnings,
                macro_artifacts,
                options: &options,
                codegen_options: &codegen_options,
            },
        ));
    };

    // Extract normal script content if present (for type definitions, imports, etc.)
    // When both <script> and <script setup> exist, normal script content should be preserved
    // (except for export default which is handled by script setup)
    let normal_script_content = if has_script {
        let script = descriptor.script.as_ref().unwrap();
        // Check if source is TypeScript
        let source_is_ts = is_ts_lang(script.lang.as_deref());
        Some(profile!(
            "atelier.sfc.normal_script.extract",
            extract_normal_script_content(&script.content, source_is_ts, is_ts)
        ))
    } else {
        None
    };

    let lazy_hydration_transform = transform_lazy_hydration_macros(&script_setup.content);
    let script_setup_source = lazy_hydration_transform
        .as_ref()
        .map(|result| result.code.clone())
        .unwrap_or_else(|| script_setup.content.to_compact_string());
    let script_setup_content = erase_artifact_macro_statements(&script_setup_source)
        .unwrap_or_else(|| script_setup_source);

    // Parse the script setup once. Croquis binding analysis, the macro
    // context analysis, and (unless v-model demotion rewrites the content
    // below) the inline compiler's statement sectioning all reuse this AST
    // instead of independently re-parsing identical content. `None` means the
    // parser panicked; each stage then falls back to its legacy parse path,
    // which reproduces the historical panicked behavior.
    let setup_ast_allocator = oxc_allocator::Allocator::default();
    let setup_program =
        crate::script::parse_script_setup_program(&setup_ast_allocator, &script_setup_content);

    // 1. Croquis parser: rich analysis with ReactivityTracker
    let mut croquis = profile!(
        "atelier.sfc.script_setup.croquis",
        match setup_program.as_ref() {
            Some(program) => crate::script::analyze_script_setup_program_to_summary(
                program,
                &script_setup_content,
            ),
            None => crate::script::analyze_script_setup_to_summary(&script_setup_content),
        }
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
    let source_is_ts = is_ts_lang(script_setup.lang.as_deref());
    profile!(
        "atelier.sfc.script_context.collect_setup_import_types",
        ctx.collect_imported_types_from_path(&script_setup_content, source_filename, source_is_ts)
    );
    if has_script {
        let script = descriptor.script.as_ref().unwrap();
        profile!(
            "atelier.sfc.script_context.collect_normal_import_types",
            ctx.collect_imported_types_from_path(
                &script.content,
                source_filename,
                is_ts_lang(script.lang.as_deref()),
            )
        );
    }
    profile!(
        "atelier.sfc.script_context.analyze",
        match setup_program.as_ref() {
            Some(program) => ctx.analyze_program(program, &script_setup_content),
            None => ctx.analyze(),
        }
    );

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

    let setup_css_module_names = compiled_styles
        .css_modules
        .iter()
        .filter(|module| {
            is_valid_identifier(&module.name) && !ctx.bindings.bindings.contains_key(&module.name)
        })
        .map(|module| module.name.clone())
        .collect::<Vec<_>>();
    for module_name in &setup_css_module_names {
        script_bindings
            .bindings
            .entry(module_name.clone())
            .or_insert(BindingType::SetupConst);
        ctx.bindings
            .bindings
            .entry(module_name.clone())
            .or_insert(BindingType::SetupConst);
    }

    // When v-model demotion rewrites `const` to `let`, the rewritten content
    // replaces the original for the inline compile below and the shared
    // pre-demotion AST is no longer valid for it.
    let mut demoted_setup_content: Option<String> = None;
    if let Some(template) = &descriptor.template {
        let demote_result = profile!(
            "atelier.sfc.script_setup.demote_v_model_reactive_consts",
            demote_v_model_reactive_const_bindings(
                &template.content,
                script_setup.lang.as_deref(),
                &script_setup_content,
                &mut ctx,
                &mut script_bindings,
                &mut croquis,
            )
        );

        if let Some((rewritten, demoted_ids)) = demote_result {
            for binding_name in demoted_ids {
                warnings.push(create_v_model_reactive_const_warning(
                    script_setup,
                    &binding_name,
                ));
            }
            demoted_setup_content = Some(rewritten);
        }
    }

    // Compile template with bindings (if present) to get the render function
    let template_result = if let Some(template) = &descriptor.template {
        if is_vapor {
            Some(profile!(
                "atelier.sfc.template.vapor",
                compile_template_block_vapor(
                    template,
                    vapor_context,
                    Some(&script_bindings),
                    shared_syntax,
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
                    shared_syntax,
                    &codegen_options,
                )
            ))
        }
    } else {
        None
    };
    let source_map = if let Some(Ok(template_output)) = &template_result {
        warnings.extend(template_output.warnings.clone());
        record_template_source_map_fact(template_output, SourceMapComposition::Skipped)
    } else {
        None
    };
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
            if is_vapor || options.template.ssr {
                let render_fn_name = if options.template.ssr {
                    "ssrRender"
                } else {
                    "render"
                };
                let parts = profile!(
                    "atelier.sfc.template.extract_parts_full",
                    template_output.full_parts_for_inline(render_fn_name)
                )?;
                (
                    parts.imports,
                    parts.hoisted,
                    parts.render_fn,
                    parts.render_fn_name,
                    String::default(),
                    String::default(),
                )
            } else {
                let parts = profile!(
                    "atelier.sfc.template.extract_parts",
                    template_output.body_parts_for_inline()
                )?;
                (
                    parts.imports,
                    parts.hoisted,
                    String::default(),
                    parts.render_fn_name,
                    parts.preamble,
                    parts.render_body,
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

    // When demotion rewrote the content, the shared AST is stale for the
    // rewritten text; the inline compiler re-parses in that rare case.
    let (setup_content_for_inline, setup_program_for_inline) = match demoted_setup_content.as_ref()
    {
        Some(content) => (content.as_str(), None),
        None => (script_setup_content.as_str(), setup_program.as_ref()),
    };

    // Compile script setup with compiler-sfc-compatible inline output ordering.
    let script_result = profile!(
        "atelier.sfc.script_setup.inline_compile",
        compile_script_setup_inline_with_context(
            ctx,
            setup_content_for_inline,
            setup_program_for_inline,
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
            &compiled_styles.css_modules,
            &setup_css_module_names,
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

    finalize_output_mode(&mut code, &mut warnings, &options, &codegen_options);
    trim_trailing_newlines(&mut code);
    let map = compose_template_source_map(source_map, &code, descriptor, filename);

    Ok(SfcCompileResult {
        code,
        css,
        map,
        errors,
        warnings,
        bindings: script_result.bindings,
        macro_artifacts,
    })
}
