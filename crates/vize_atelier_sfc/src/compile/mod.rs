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

use crate::compile_script::{compile_script_setup_inline_with_context, TemplateParts};
use crate::compile_template::{
    compile_template_block, compile_template_block_vapor, extract_template_parts,
    extract_template_parts_full,
};
use crate::rewrite_default::rewrite_default;
use crate::script::ScriptCompileContext;
use crate::types::{BindingType, SfcCompileOptions, SfcCompileResult, SfcDescriptor, SfcError};

use self::bindings::{croquis_to_legacy_bindings, register_normal_script_bindings};
use self::helpers::{extract_component_name, generate_scope_id};
use self::normal_script::extract_normal_script_content;
use self::styles::compile_styles;

// Re-export ScriptCompileResult for public API
pub use crate::compile_script::ScriptCompileResult;
use vize_carton::{profile, String, ToCompactString};

/// Compile an SFC descriptor into JavaScript and CSS
pub fn compile_sfc(
    descriptor: &SfcDescriptor,
    options: SfcCompileOptions,
) -> Result<SfcCompileResult, SfcError> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut code = String::default();
    let mut css = None;

    let filename = options.script.id.as_deref().unwrap_or("anonymous.vue");

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

    // Vapor components currently render on the client. For SSR we fall back to
    // the standard VDOM compiler and let the client hydrate with Vapor output.
    let is_vapor = !options.template.ssr
        && (options.vapor
            || descriptor
                .script_setup
                .as_ref()
                .map(|s| s.attrs.contains_key("vapor"))
                .unwrap_or(false)
            || descriptor
                .script
                .as_ref()
                .map(|s| s.attrs.contains_key("vapor"))
                .unwrap_or(false));

    // source_has_ts: whether source uses TypeScript (detected from lang="ts")
    // Used for: parsing source as TS, preserving TS declarations, resolving type references
    let source_has_ts = descriptor
        .script_setup
        .as_ref()
        .and_then(|s| s.lang.as_ref())
        .is_some_and(|l| l == "ts" || l == "tsx")
        || descriptor
            .script
            .as_ref()
            .and_then(|s| s.lang.as_ref())
            .is_some_and(|l| l == "ts" || l == "tsx");
    // is_ts controls output format:
    // - true: output TypeScript (add `: any` annotations, defineComponent wrapper)
    // - false: output JavaScript (no type annotations)
    // Auto-detected from source lang, or set by explicit options.
    // When true, TypeScript is preserved in output (downstream tools like Vite strip it via .ts suffix).
    let is_ts = options.script.is_ts || options.template.is_ts || source_has_ts;

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
                compile_template_block_vapor(template, &scope_id, has_scoped, None)
            )
        } else {
            // Enable hoisting for template-only SFCs (hoisted consts go at module level)
            let mut template_opts = options.template.clone();
            let mut dom_opts = template_opts.compiler_options.take().unwrap_or_default();
            dom_opts.hoist_static = true;
            template_opts.compiler_options = Some(dom_opts);
            // Don't pass scope IDs to template compiler - scoped CSS is handled by
            // runtime __scopeId and CSS transformation, not by adding attributes
            // to template elements during compilation.
            profile!(
                "atelier.sfc.template.compile",
                compile_template_block(
                    template,
                    &template_opts,
                    &scope_id,
                    options.template.ssr && has_scoped,
                    is_ts,
                    None,
                    None,
                )
            )
        };

        match template_result {
            Ok(template_code) => {
                code = template_code;
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
            Err(e) => errors.push(e),
        }

        // Compile styles
        let all_css = profile!(
            "atelier.sfc.styles",
            compile_styles(&descriptor.styles, &scope_id, &options.style, &mut warnings)
        );
        if !all_css.is_empty() {
            css = Some(all_css);
        }

        return Ok(SfcCompileResult {
            code,
            css,
            map: None,
            errors,
            warnings,
            bindings: None,
        });
    }

    // Case 2: Script (non-setup) + Template - rewrite default and compile template
    if has_script && !has_script_setup {
        let script = descriptor.script.as_ref().unwrap();

        // Check if source script is TypeScript
        let source_is_ts = script
            .lang
            .as_ref()
            .is_some_and(|l| l == "ts" || l == "tsx");

        // Rewrite `export default` to `const _sfc_main = ...`
        // Parse as TypeScript if source is TypeScript
        let (rewritten_script, _has_default) = profile!(
            "atelier.sfc.normal_script.rewrite_default",
            rewrite_default(&script.content, "_sfc_main", source_is_ts)
        );

        // Transpile TypeScript to JavaScript if needed
        let final_script = if source_is_ts && !is_ts {
            profile!(
                "atelier.sfc.normal_script.ts_to_js",
                crate::compile_script::typescript::transform_typescript_to_js(&rewritten_script)
            )
        } else {
            rewritten_script
        };

        // Compile template if present
        if has_template {
            let template = descriptor.template.as_ref().unwrap();
            let template_result = if is_vapor {
                profile!(
                    "atelier.sfc.template.vapor",
                    compile_template_block_vapor(template, &scope_id, has_scoped, None)
                )
            } else {
                let mut template_opts = options.template.clone();
                let mut dom_opts = template_opts.compiler_options.take().unwrap_or_default();
                dom_opts.hoist_static = true;
                template_opts.compiler_options = Some(dom_opts);

                // Don't pass scope IDs to template compiler - scoped CSS is handled by
                // runtime __scopeId and CSS transformation.
                profile!(
                    "atelier.sfc.template.compile",
                    compile_template_block(
                        template,
                        &template_opts,
                        &scope_id,
                        options.template.ssr && has_scoped,
                        is_ts,
                        None, // No bindings for normal scripts
                        None, // No Croquis for normal scripts
                    )
                )
            };

            match template_result {
                Ok(template_code) => {
                    // Build output matching Vue's compiler-sfc:
                    // 1. Full template output (imports + hoisted + export function render(...))
                    // 2. Rewritten script
                    // 3. _sfc_main.render = render / _sfc_main.ssrRender = ssrRender
                    // 4. export default _sfc_main
                    code.push_str(&template_code);
                    code.push_str(&final_script);
                    code.push('\n');

                    // Export the component with render attached
                    if is_vapor {
                        code.push_str("_sfc_main.__vapor = true\n");
                    }
                    if options.template.ssr {
                        code.push_str("_sfc_main.ssrRender = ssrRender\n");
                    } else {
                        code.push_str("_sfc_main.render = render\n");
                    }
                    code.push_str("export default _sfc_main\n");
                }
                Err(e) => {
                    errors.push(e);
                    // Fall back to just the script
                    code = script.content.to_compact_string();
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

        return Ok(SfcCompileResult {
            code,
            css,
            map: None,
            errors,
            warnings,
            bindings: None,
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

    // 1. Croquis parser: rich analysis with ReactivityTracker
    let croquis = profile!(
        "atelier.sfc.script_setup.croquis",
        crate::script::analyze_script_setup_to_summary(&script_setup.content)
    );
    let mut script_bindings = croquis_to_legacy_bindings(&croquis.bindings);

    // 2. ScriptCompileContext: needed for macro span info and TypeScript type resolution
    //    (Croquis doesn't resolve type references like `defineProps<Props>()`)
    let mut ctx = profile!(
        "atelier.sfc.script_context.new",
        ScriptCompileContext::new(&script_setup.content)
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
        ctx.collect_imported_types_from_path(&script_setup.content, filename)
    );
    if has_script {
        let script = descriptor.script.as_ref().unwrap();
        profile!(
            "atelier.sfc.script_context.collect_normal_import_types",
            ctx.collect_imported_types_from_path(&script.content, filename)
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
    // When both <script> and <script setup> exist, all imports and exported
    // variables from the normal script are accessible in the template.
    // This enables proper component resolution (e.g., `import { Form as PForm }`)
    // and identifier prefix resolution (avoiding incorrect `_ctx.` prefix).
    if has_script {
        let script = descriptor.script.as_ref().unwrap();
        profile!(
            "atelier.sfc.normal_script.register_bindings",
            register_normal_script_bindings(&script.content, &mut script_bindings)
        );
    }

    // Compile template with bindings (if present) to get the render function
    let template_result = if let Some(template) = &descriptor.template {
        if is_vapor {
            Some(profile!(
                "atelier.sfc.template.vapor",
                compile_template_block_vapor(
                    template,
                    &scope_id,
                    has_scoped,
                    Some(&script_bindings)
                )
            ))
        } else {
            // Don't pass scope IDs to template compiler - scoped CSS is handled by
            // runtime __scopeId and CSS transformation.
            Some(profile!(
                "atelier.sfc.template.compile",
                compile_template_block(
                    template,
                    &options.template,
                    &scope_id,
                    options.template.ssr && has_scoped,
                    is_ts,
                    Some(&script_bindings), // Pass bindings for proper ref handling
                    Some(croquis),          // Pass Croquis for enhanced transforms
                )
            ))
        }
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
        Some(Ok(template_code)) => {
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
    // Detect if the source script setup uses TypeScript
    let source_is_ts = script_setup
        .lang
        .as_ref()
        .is_some_and(|l| l == "ts" || l == "tsx");

    let script_result = profile!(
        "atelier.sfc.script_setup.inline_compile",
        compile_script_setup_inline_with_context(
            ctx,
            &script_setup.content,
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
        )
    )?;

    // The inline mode compile_script_setup_inline generates a complete output
    // including imports, hoisted vars, and `export default { ... }` with inline render
    code.push_str(&script_result.code);

    // Compile styles
    let all_css = profile!(
        "atelier.sfc.styles",
        compile_styles(&descriptor.styles, &scope_id, &options.style, &mut warnings)
    );
    if !all_css.is_empty() {
        css = Some(all_css);
    }

    Ok(SfcCompileResult {
        code,
        css,
        map: None,
        errors,
        warnings,
        bindings: script_result.bindings,
    })
}
