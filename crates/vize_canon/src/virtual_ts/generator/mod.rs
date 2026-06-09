//! Main virtual TypeScript generation entry points.
//!
//! Contains the public `generate_virtual_ts` and `generate_virtual_ts_with_offsets`
//! functions that orchestrate the full virtual TypeScript generation pipeline.

mod generics;
mod imports;
mod options_api;
mod spans;

use vize_croquis::{BindingType, Croquis, ScopeData, ScopeKind};

use self::generics::{generic_injection_point, references_any_identifier};
use self::imports::{
    collect_imported_names, emit_global_component_stubs, emit_reference_type_directives,
    extract_declared_name,
};
use self::options_api::generate_options_api_variables;
use self::spans::{
    collect_template_referenced_names, is_local_setup_binding, merge_overlapping_spans,
    rewrite_export_default_for_module_scope,
};
use super::{
    helpers::{
        IMPORT_META_AUGMENTATION, SETUP_SCOPE_HELPER_NAMES, VUE_SETUP_HELPERS, VUE_TYPE_HELPERS,
        generate_template_context, to_safe_identifier,
    },
    props::{
        add_generic_defaults, collect_template_prop_names, extract_generic_names,
        generate_props_type, generate_props_variables,
    },
    scope::{ScopeGenerationOptions, generate_scope_closures},
    types::{VirtualTsGenerationOptions, VirtualTsOptions, VirtualTsOutput, VizeMapping},
};
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::profile;
use vize_carton::{FxHashMap, FxHashSet, String};

/// Generate virtual TypeScript from Vue SFC analysis.
///
/// The generated TypeScript uses proper scope hierarchy:
/// 1. Module scope: imports only
/// 2. Setup scope (__setup function): compiler macros + script content
/// 3. Template scope (nested in setup): template expressions
///
/// This ensures compiler macros like defineProps are ONLY valid in setup scope.
pub fn generate_virtual_ts(
    summary: &Croquis,
    script_content: Option<&str>,
    template_ast: Option<&vize_relief::ast::RootNode<'_>>,
    template_offset: u32,
) -> VirtualTsOutput {
    generate_virtual_ts_with_offsets(
        summary,
        script_content,
        template_ast,
        0,
        template_offset,
        &VirtualTsOptions::default(),
    )
}

/// Generate virtual TypeScript with explicit script and template offsets.
///
/// `script_offset` is the byte offset of the script content within the SFC file.
/// `template_offset` is the byte offset of the template content within the SFC file.
/// When these are provided, source mappings point to SFC-absolute positions.
/// `options` controls template globals and other generation settings.
pub fn generate_virtual_ts_with_offsets(
    summary: &Croquis,
    script_content: Option<&str>,
    template_ast: Option<&vize_relief::ast::RootNode<'_>>,
    script_offset: u32,
    template_offset: u32,
    options: &VirtualTsOptions,
) -> VirtualTsOutput {
    generate_virtual_ts_with_offsets_and_checks(
        summary,
        script_content,
        template_ast,
        script_offset,
        template_offset,
        options,
        VirtualTsGenerationOptions::default(),
    )
}

/// Generate virtual TypeScript with Vue 3 Options API binding resolution
/// enabled (opt-in, standard build — no `legacy` feature required).
pub fn generate_virtual_ts_with_offsets_options_api(
    summary: &Croquis,
    script_content: Option<&str>,
    template_ast: Option<&vize_relief::ast::RootNode<'_>>,
    script_offset: u32,
    template_offset: u32,
    options: &VirtualTsOptions,
) -> VirtualTsOutput {
    generate_virtual_ts_with_offsets_and_checks(
        summary,
        script_content,
        template_ast,
        script_offset,
        template_offset,
        options,
        VirtualTsGenerationOptions {
            options_api: true,
            ..Default::default()
        },
    )
}

/// Generate virtual TypeScript with Vue 2.7 / Nuxt 2 compatibility enabled.
pub fn generate_virtual_ts_with_offsets_legacy_vue2(
    summary: &Croquis,
    script_content: Option<&str>,
    template_ast: Option<&vize_relief::ast::RootNode<'_>>,
    script_offset: u32,
    template_offset: u32,
    options: &VirtualTsOptions,
) -> VirtualTsOutput {
    generate_virtual_ts_with_offsets_and_checks(
        summary,
        script_content,
        template_ast,
        script_offset,
        template_offset,
        options,
        VirtualTsGenerationOptions {
            legacy_vue2: true,
            ..Default::default()
        },
    )
}

pub(crate) fn generate_virtual_ts_with_offsets_and_checks(
    summary: &Croquis,
    script_content: Option<&str>,
    template_ast: Option<&vize_relief::ast::RootNode<'_>>,
    script_offset: u32,
    template_offset: u32,
    options: &VirtualTsOptions,
    generation_options: VirtualTsGenerationOptions,
) -> VirtualTsOutput {
    let check_options = generation_options.check_options;
    let legacy_vue2 = generation_options.legacy_vue2;
    let options_api = generation_options.options_api || legacy_vue2;
    let mut ts = String::default();
    let mut mappings: Vec<VizeMapping> = Vec::new();
    let preserve_unused_diagnostics = generation_options.preserve_unused_diagnostics;
    let template_referenced_names =
        preserve_unused_diagnostics.then(|| collect_template_referenced_names(summary));
    let reference_setup_bindings_comment = if preserve_unused_diagnostics {
        "Reference setup bindings used by template generation"
    } else {
        "Reference setup bindings (used in template/CSS v-bind)"
    };

    // Header with ES target library references.
    // These ensure import.meta, Promise, Array.includes(), etc. are available.
    ts.push_str("/// <reference lib=\"es2022\" />\n");
    ts.push_str("/// <reference lib=\"dom\" />\n");
    ts.push_str("/// <reference lib=\"dom.iterable\" />\n");
    let has_script_reference_types = emit_reference_type_directives(&mut ts, script_content);
    ts.push_str("// ============================================\n");
    ts.push_str("// Virtual TypeScript for Vue SFC Type Checking\n");
    ts.push_str("// Generated by vize\n");
    ts.push_str("// ============================================\n\n");

    // Check for generic type parameter from <script setup generic="T">
    let (generic_param, mut is_async) = summary
        .scopes
        .iter()
        .find(|s| matches!(s.kind, ScopeKind::ScriptSetup))
        .map(|s| {
            if let ScopeData::ScriptSetup(data) = s.data() {
                (data.generic.as_ref().map(|s| s.as_str()), data.is_async)
            } else {
                (None, false)
            }
        })
        .unwrap_or((None, false));

    // Also detect top-level await in script content (Vue 3 script setup supports this)
    if let Some(script) = script_content
        && script.contains("await ")
        && !is_async
    {
        is_async = true;
    }

    // ImportMeta augmentation (must be at top level, before any code)
    ts.push_str(IMPORT_META_AUGMENTATION);
    ts.push('\n');

    // Module scope: Extract imports, re-exports, and type declarations to module level.
    // Type declarations (interface, type, enum) must be at module level so they
    // are accessible from `export type Props = ...` outside __setup().
    ts.push_str("// ========== Module Scope (imports) ==========\n");
    ts.push_str(VUE_TYPE_HELPERS);
    ts.push('\n');

    // Collect all module-level statement spans from croquis analysis once and
    // keep them sorted. Later script-body emission advances an index through
    // this list, so each source line checks only the overlapping tail instead
    // of rescanning imports/re-exports/type declarations from the start.
    let module_spans: Vec<(u32, u32)> = profile!("canon.virtual_ts.collect_module_spans", {
        let mut module_spans = Vec::new();
        for imp in &summary.import_statements {
            module_spans.push((imp.start, imp.end));
        }
        for re in &summary.re_exports {
            module_spans.push((re.start, re.end));
        }
        let has_script_setup = summary
            .scopes
            .iter()
            .any(|scope| matches!(scope.kind, ScopeKind::ScriptSetup));
        if has_script_setup {
            module_spans.extend(summary.scopes.iter().filter_map(|scope| {
                matches!(scope.kind, ScopeKind::NonScriptSetup)
                    .then_some((scope.span.start, scope.span.end))
            }));
        }
        for te in &summary.type_exports {
            // Non-hoisted types reference setup-scope values via `typeof`
            // and must stay inside `__setup` so TS can resolve them.
            if te.hoisted {
                module_spans.push((te.start, te.end));
            }
        }
        merge_overlapping_spans(module_spans)
    });

    // For a `<script setup generic="...">` SFC, hoisted type declarations are
    // lifted verbatim to module scope, but the generic parameters only live on
    // `__setup<...>()`. A lifted declaration that mentions a generic parameter
    // (e.g. `type Option = { key: T }`) would reference an unbound name there.
    // Re-declare the SFC generics as defaulted parameters on each such
    // declaration (`type Option<T extends string = any> = ...`) so the
    // reference resolves at module scope while bare uses (`Option[]`) still
    // work via the `= any` defaults.
    let generic_injection: Option<(String, Vec<String>)> = generic_param.map(|g| {
        let defaults = add_generic_defaults(g);
        let names = extract_generic_names(g)
            .split(',')
            .map(|n| String::from(n.trim()))
            .filter(|n| !n.is_empty())
            .collect();
        (defaults, names)
    });
    let hoisted_type_spans: FxHashMap<(u32, u32), &str> = if generic_injection.is_some() {
        summary
            .type_exports
            .iter()
            .filter(|te| te.hoisted)
            .map(|te| ((te.start, te.end), te.name.as_str()))
            .collect()
    } else {
        FxHashMap::default()
    };

    if let Some(script) = script_content {
        profile!("canon.virtual_ts.emit_module_statements", {
            // Emit each module-level statement with source mapping
            for &(start, end) in &module_spans {
                let text = &script[start as usize..end as usize];

                // Splice the SFC generic parameters into a hoisted
                // type/interface declaration that references them, so the
                // reference resolves at module scope.
                if let Some((defaults, names)) = &generic_injection
                    && let Some(type_name) = hoisted_type_spans.get(&(start, end))
                    && references_any_identifier(text, names)
                    && let Some(inject_at) = generic_injection_point(text, type_name)
                {
                    let (prefix, suffix) = text.split_at(inject_at);
                    let src_base = script_offset as usize + start as usize;

                    let gen_start = ts.len();
                    ts.push_str(prefix);
                    mappings.push(VizeMapping {
                        gen_range: gen_start..ts.len(),
                        src_range: src_base..(src_base + prefix.len()),
                        sub_spans: Vec::new(),
                    });

                    // Synthetic parameter list; no corresponding source span.
                    append!(ts, "<{defaults}>");
                    // Avoid forming `>=` when the alias has no space before `=`.
                    if suffix.starts_with('=') {
                        ts.push(' ');
                    }

                    let gen_start = ts.len();
                    ts.push_str(suffix);
                    ts.push('\n');
                    mappings.push(VizeMapping {
                        gen_range: gen_start..ts.len(),
                        src_range: (src_base + prefix.len())
                            ..(src_base + prefix.len() + suffix.len()),
                        sub_spans: Vec::new(),
                    });
                    continue;
                }

                let gen_start = ts.len();
                if text.contains("export default") {
                    let text = rewrite_export_default_for_module_scope(text);
                    ts.push_str(&text);
                } else {
                    ts.push_str(text);
                }
                ts.push('\n');
                let gen_end = ts.len();
                mappings.push(VizeMapping {
                    gen_range: gen_start..gen_end,
                    src_range: (script_offset as usize + start as usize)
                        ..(script_offset as usize + end as usize),
                    sub_spans: Vec::new(),
                });
            }

            // Void-reference imported names that match setup-scope helper names.
            // These get shadowed by __setup() declarations, causing TS6133 at module level.
            let shadowed_imports: Vec<&&str> = SETUP_SCOPE_HELPER_NAMES
                .iter()
                .filter(|&&name| summary.bindings.bindings.contains_key(name))
                .collect();
            if !shadowed_imports.is_empty() {
                ts.push_str(
                    "// Prevent TS6133 for imports shadowed by setup-scope compiler macros\n",
                );
                for name in &shadowed_imports {
                    append!(ts, "void {name};\n");
                }
            }
        });
    }

    let needs_imported_names = !options.auto_import_stubs.is_empty()
        || (has_script_reference_types && !summary.component_usages.is_empty());
    let imported_names: FxHashSet<&str> = if needs_imported_names {
        profile!(
            "canon.virtual_ts.extract_imported_names",
            collect_imported_names(summary, script_content)
        )
    } else {
        FxHashSet::default()
    };

    // Auto-import stubs (e.g., Nuxt composables)
    // Only emit stubs for names NOT already declared via imports or bindings.
    // Collect imported names from all module-level import statements to handle
    // cases where plain <script> imports are not in summary.bindings (which
    // only holds <script setup> bindings when both blocks exist).
    if !options.auto_import_stubs.is_empty() {
        profile!("canon.virtual_ts.emit_auto_import_stubs", {
            let mut has_header = false;
            for stub in &options.auto_import_stubs {
                let name = extract_declared_name(stub);
                if let Some(name) = name {
                    // Skip if already imported or declared in script bindings
                    if summary.bindings.bindings.contains_key(name)
                        || imported_names.contains(&name)
                    {
                        continue;
                    }
                }
                if !has_header {
                    ts.push_str("\n// Auto-import stubs (framework-provided globals)\n");
                    has_header = true;
                }
                ts.push_str(stub);
                ts.push('\n');
            }
        });
    }
    emit_global_component_stubs(
        &mut ts,
        summary,
        options,
        &imported_names,
        has_script_reference_types,
    );
    ts.push('\n');

    // Props type (defined at module level so it's available inside __setup)
    profile!(
        "canon.virtual_ts.generate_props_type",
        generate_props_type(&mut ts, summary, generic_param)
    );

    // Setup scope: function that contains setup helpers and script content
    ts.push_str("// ========== Setup Scope ==========\n");
    let async_prefix = if is_async { "async " } else { "" };
    let generic_params = generic_param.map(|g| cstr!("<{g}>")).unwrap_or_default();
    append!(ts, "{async_prefix}function __setup{generic_params}() {{\n",);

    // Setup helpers (only valid inside setup scope)
    ts.push_str(VUE_SETUP_HELPERS);
    ts.push_str("\n\n");

    // User's script content (minus imports)
    if let Some(script) = script_content {
        profile!("canon.virtual_ts.emit_script_body", {
            ts.push_str("  // User setup code\n");
            let script_gen_start = ts.len();
            // Use split('\n') to correctly track byte offsets for CRLF files.
            // Rust's lines() strips \r from CRLF but +1 for \n undercounts,
            // causing src_byte_offset drift that incorrectly skips user code.
            let mut src_byte_offset: usize = 0; // offset within script content
            let mut module_span_index = 0usize;

            // Check if script uses import.meta and add a polyfill variable.
            // This avoids TS1343 when module is not set to es2020+.
            let uses_import_meta = script.contains("import.meta");
            if uses_import_meta {
                ts.push_str("  const __import_meta: any = {};\n");
            }

            for raw_line in script.split('\n') {
                // Strip trailing \r for output (normalize CRLF to LF)
                let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
                // raw_line.len() includes \r if present; +1 for the \n from split
                let raw_byte_len = raw_line.len() + 1;

                // Skip lines that overlap with module-level spans (imports, re-exports, type decls)
                let line_start = src_byte_offset;
                let line_end = line_start + raw_line.len(); // use raw length for span check
                while module_span_index < module_spans.len()
                    && module_spans[module_span_index].1 as usize <= line_start
                {
                    module_span_index += 1;
                }
                let is_module_level = module_spans[module_span_index..]
                    .iter()
                    .take_while(|&&(start, _)| (start as usize) < line_end)
                    .any(|&(start, end)| line_start < end as usize && line_end > start as usize);
                if is_module_level {
                    src_byte_offset += raw_byte_len;
                    continue;
                }
                let gen_line_start = ts.len();
                ts.push_str("  "); // indentation (not in source)
                let gen_content_start = ts.len();

                // Process the line: strip `export` keyword (invalid inside function),
                // replace import.meta with polyfill variable
                let mut output_line = std::borrow::Cow::Borrowed(line);

                // Strip `export` from non-import lines inside setup scope
                let trimmed_line = output_line.trim_start();
                if let Some(default_expr) = trimmed_line
                    .strip_prefix("export default")
                    .filter(|rest| rest.chars().next().is_none_or(char::is_whitespace))
                {
                    let leading_ws = &output_line[..output_line.len() - trimmed_line.len()];
                    #[allow(clippy::disallowed_types)]
                    {
                        output_line = std::borrow::Cow::Owned(
                            cstr!("{leading_ws}const __default__ ={}", default_expr).into(),
                        );
                    }
                } else if trimmed_line.starts_with("export ")
                    && !trimmed_line.starts_with("export type ")
                    && !trimmed_line.starts_with("export interface ")
                {
                    let leading_ws = &output_line[..output_line.len() - trimmed_line.len()];
                    if let Some(rest) = trimmed_line.strip_prefix("export ") {
                        #[allow(clippy::disallowed_types)]
                        {
                            output_line =
                                std::borrow::Cow::Owned(cstr!("{leading_ws}{rest}").into());
                        }
                    }
                }

                // Replace import.meta with polyfill variable to avoid TS1343
                if uses_import_meta && output_line.contains("import.meta") {
                    #[allow(clippy::disallowed_types)]
                    {
                        output_line = std::borrow::Cow::Owned(
                            output_line.replace("import.meta", "__import_meta"),
                        );
                    }
                }

                ts.push_str(&output_line);
                let gen_content_end = ts.len();
                ts.push('\n');
                // Map the line content (excluding the "  " indent prefix)
                if !line.is_empty() {
                    let src_line_start = script_offset as usize + src_byte_offset;
                    let src_line_end = src_line_start + line.len();
                    mappings.push(VizeMapping {
                        gen_range: gen_content_start..gen_content_end,
                        src_range: src_line_start..src_line_end,
                        sub_spans: Vec::new(),
                    });
                }
                let _ = gen_line_start; // suppress unused warning
                src_byte_offset += raw_byte_len;
            }
            let script_gen_end = ts.len();
            append!(
                ts,
                "  // @vize-map: {script_gen_start}:{script_gen_end} -> 0:{}\n\n",
                script.len()
            );
        });
    }

    // Template scope (nested inside setup)
    if template_ast.is_some() {
        profile!("canon.virtual_ts.emit_template_scope", {
            ts.push_str("  // ========== Template Scope (inherits from setup) ==========\n");

            // Collect ref bindings for auto-unwrapping in template
            let mut ref_bindings: Vec<&str> =
                if let Some(template_referenced_names) = template_referenced_names.as_ref() {
                    summary
                        .bindings
                        .bindings
                        .iter()
                        .filter(|(name, _)| template_referenced_names.contains(name.as_str()))
                        .filter(|(name, binding_type)| {
                            summary.reactivity.needs_value_access(name.as_str())
                                || matches!(binding_type, BindingType::SetupMaybeRef)
                                    && is_local_setup_binding(summary, name.as_str())
                        })
                        .map(|(name, _)| name.as_str())
                        .collect()
                } else {
                    summary
                        .bindings
                        .bindings
                        .iter()
                        .filter(|(name, binding_type)| {
                            summary.reactivity.needs_value_access(name.as_str())
                                || matches!(binding_type, BindingType::SetupMaybeRef)
                                    && is_local_setup_binding(summary, name.as_str())
                        })
                        .map(|(name, _)| name.as_str())
                        .collect()
                };
            ref_bindings.sort_unstable();

            // Capture ref types BEFORE template scope to avoid circular references.
            // `typeof count` here refers to the setup-scope Ref<number>.
            if !ref_bindings.is_empty() {
                ts.push_str("  // Ref type captures (before template scope shadows them)\n");
                for name in &ref_bindings {
                    append!(ts, "  type __R_{name} = typeof {name};\n");
                }
            }

            // Semicolon prevents ASI issues when user script doesn't end with `;`
            // (e.g., `console.log(x)\n(function...)` would be parsed as a call)
            ts.push_str("  ;(function __template() {\n");

            // Shadow ref bindings with unwrapped types.
            // `var` allows reassignment (Vue templates can assign to refs).
            if !ref_bindings.is_empty() {
                ts.push_str("    // Auto-unwrap Vue refs in template scope\n");
                ts.push_str(
                    "    type __U<T> = T extends import('vue').Ref<infer V, any> ? V : T;\n",
                );
                for name in &ref_bindings {
                    append!(ts, "    var {name}: __U<__R_{name}> = undefined as any;\n");
                }
            }

            // Vue template context (available in template expressions)
            let template_context = profile!(
                "canon.virtual_ts.generate_template_context",
                generate_template_context(options)
            );
            ts.push_str(&template_context);
            ts.push('\n');

            // Props are available in template as variables
            profile!(
                "canon.virtual_ts.generate_props_variables",
                generate_props_variables(&mut ts, summary, script_content, generic_param)
            );
            if options_api {
                profile!(
                    "canon.virtual_ts.generate_options_api_variables",
                    generate_options_api_variables(&mut ts, summary, options)
                );
            }
            let template_prop_names = profile!(
                "canon.virtual_ts.collect_template_prop_names",
                collect_template_prop_names(summary, script_content)
            );

            // Generate scope closures
            if check_options.any_enabled() {
                profile!(
                    "canon.virtual_ts.generate_scope_closures",
                    generate_scope_closures(
                        &mut ts,
                        &mut mappings,
                        summary,
                        &template_prop_names,
                        template_offset,
                        ScopeGenerationOptions {
                            check_options,
                            virtual_ts_options: options,
                            check_unresolved_global_components: has_script_reference_types,
                        },
                    )
                );
            }

            // Declare unresolved components (auto-imported or built-in) as `any`.
            // Names known to be provided by ambient project declarations stay
            // unshadowed so their actual component prop types are preserved.
            if !summary.used_components.is_empty() {
                let external_template_bindings: FxHashSet<&str> = options
                    .external_template_bindings
                    .iter()
                    .map(|name| name.as_str())
                    .collect();
                let mut has_unresolved = false;
                for component in &summary.used_components {
                    let name = component.as_str();
                    // Skip if already declared via script bindings (import/const)
                    if summary.bindings.bindings.contains_key(name)
                        || external_template_bindings.contains(name)
                        || has_script_reference_types
                    {
                        continue;
                    }
                    if !has_unresolved {
                        ts.push_str(
                            "\n  // Auto-imported/built-in components (not in script bindings)\n",
                        );
                        has_unresolved = true;
                    }
                    let safe = to_safe_identifier(name);
                    append!(ts, "  const {safe}: any = undefined as any;\n");
                }

                ts.push_str("\n  // Mark used components as referenced\n");
                for component in &summary.used_components {
                    let safe = to_safe_identifier(component.as_str());
                    append!(ts, "  void {safe};\n");
                }
            }

            // In projects that opt into unused-local diagnostics, this list is
            // narrowed to template-referenced names so user TS6133 can surface.
            if !summary.bindings.bindings.is_empty() {
                let mut first = true;
                let mut binding_names: Vec<&str> =
                    if let Some(template_referenced_names) = template_referenced_names.as_ref() {
                        summary
                            .bindings
                            .bindings
                            .keys()
                            .map(|name| name.as_str())
                            .filter(|name| template_referenced_names.contains(*name))
                            .collect()
                    } else {
                        summary
                            .bindings
                            .bindings
                            .keys()
                            .map(|name| name.as_str())
                            .collect()
                    };
                binding_names.sort_unstable();
                if !binding_names.is_empty() {
                    append!(ts, "\n  // {reference_setup_bindings_comment}\n  ");
                }
                for name in binding_names {
                    // Skip bindings that are JS keywords or would cause syntax errors
                    if matches!(
                        name,
                        "default"
                            | "class"
                            | "new"
                            | "delete"
                            | "void"
                            | "typeof"
                            | "in"
                            | "instanceof"
                            | "return"
                            | "switch"
                            | "case"
                            | "break"
                            | "continue"
                            | "throw"
                            | "try"
                            | "catch"
                            | "finally"
                            | "if"
                            | "else"
                            | "for"
                            | "while"
                            | "do"
                            | "with"
                            | "var"
                            | "let"
                            | "const"
                            | "function"
                            | "this"
                            | "super"
                            | "import"
                            | "export"
                            | "yield"
                            | "await"
                            | "async"
                            | "static"
                            | "enum"
                            | "implements"
                            | "interface"
                            | "package"
                            | "private"
                            | "protected"
                            | "public"
                    ) {
                        continue;
                    }
                    if !first {
                        ts.push(' ');
                    }
                    append!(ts, "void {name};");
                    first = false;
                }
                if !first {
                    ts.push('\n');
                }
            }

            ts.push_str("  })();\n");
        });
    }

    // Reference props destructure bindings at setup scope level.
    // These variables are declared in user script (e.g., `const { foo } = defineProps<...>()`)
    // but shadowed inside __template() by generate_props_variables, so void them here.
    if let Some(destructure) = summary.macros.props_destructure()
        && !destructure.bindings.is_empty()
    {
        ts.push_str("\n  // Reference destructured props (prevent TS6133)\n  ");
        let mut first = true;
        for binding in destructure.bindings.values() {
            if !first {
                ts.push(' ');
            }
            append!(ts, "void {};", binding.local);
            first = false;
        }
        if let Some(ref rest) = destructure.rest_id {
            if !first {
                ts.push(' ');
            }
            append!(ts, "void {};", rest);
        }
        ts.push('\n');
    }

    let define_emits_runtime_args = summary.macros.define_emits().and_then(|call| {
        if call.type_args.is_none() {
            call.runtime_args.as_ref()
        } else {
            None
        }
    });

    // Return runtime-derived type artifacts from __setup() so their types can be
    // extracted at module level while keeping each runtime expression in setup
    // scope, where script-setup bindings are defined.
    let mut setup_return_fields = Vec::new();
    if let Some(expose) = summary.macros.define_expose()
        && expose.type_args.is_none()
        && let Some(runtime_args) = expose.runtime_args.as_ref()
    {
        append!(ts, "\n  const __vize_exposed = ({runtime_args});\n");
        setup_return_fields.push("__vize_exposed");
    }
    if let Some(runtime_args) = define_emits_runtime_args {
        append!(
            ts,
            "\n  const __vize_emits = defineEmits({runtime_args});\n"
        );
        setup_return_fields.push("__vize_emits");
    }
    if !setup_return_fields.is_empty() {
        append!(ts, "\n  return {{ {} }};\n", setup_return_fields.join(", "));
    }

    // Close setup function
    ts.push_str("}\n\n");

    // Invoke setup to keep diagnostics inside the generated setup body.
    ts.push_str("// Invoke setup to verify types\n");
    ts.push_str("__setup();\n\n");

    // Emits type
    let emits_already_defined = summary
        .type_exports
        .iter()
        .any(|te| te.name.as_str() == "Emits");
    let define_emits_type_args = summary
        .macros
        .define_emits()
        .and_then(|call| call.type_args.as_ref());
    let models = summary.macros.models();
    let has_model_emits = !models.is_empty();
    let has_emits_for_props = emits_already_defined
        || define_emits_type_args.is_some()
        || define_emits_runtime_args.is_some()
        || !summary.macros.emits().is_empty()
        || has_model_emits;
    if !emits_already_defined {
        if let Some(type_args) = define_emits_type_args {
            let inner_type = type_args
                .strip_prefix('<')
                .and_then(|s| s.strip_suffix('>'))
                .unwrap_or(type_args.as_str());
            if has_model_emits {
                append!(ts, "export type Emits = {inner_type} & {{\n");
                for model in models {
                    let name = model.name.as_str();
                    let payload = model.model_type.as_deref().unwrap_or("unknown");
                    append!(ts, "  \"update:{name}\": [value: {payload}];\n");
                }
                ts.push_str("};\n");
            } else {
                append!(ts, "export type Emits = {inner_type};\n");
            }
        } else if define_emits_runtime_args.is_some() {
            ts.push_str(
                "export type Emits = Awaited<ReturnType<typeof __setup>>[\"__vize_emits\"]",
            );
            for model in models {
                let name = model.name.as_str();
                let payload = model.model_type.as_deref().unwrap_or("unknown");
                append!(
                    ts,
                    " & ((event: \"update:{name}\", value: {payload}) => void)"
                );
            }
            ts.push_str(";\n");
        } else if !summary.macros.emits().is_empty() || has_model_emits {
            ts.push_str("export type Emits = {\n");
            let mut emitted_names: FxHashSet<String> = FxHashSet::default();
            for emit in summary.macros.emits() {
                let payload = emit.payload_type.as_deref().unwrap_or("any[]");
                append!(ts, "  \"{}\": {payload};\n", emit.name);
                emitted_names.insert(emit.name.as_str().into());
            }
            for model in models {
                let event_name = cstr!("update:{}", model.name);
                if emitted_names.contains(event_name.as_str()) {
                    continue;
                }
                let payload = model.model_type.as_deref().unwrap_or("unknown");
                append!(ts, "  \"{event_name}\": [value: {payload}];\n");
            }
            ts.push_str("};\n");
        } else {
            ts.push_str("export type Emits = {};\n");
        }
    }

    // Slots type
    let slots_type_args = summary
        .macros
        .define_slots()
        .and_then(|m| m.type_args.as_ref());
    if let Some(type_args) = slots_type_args {
        let inner_type = type_args
            .strip_prefix('<')
            .and_then(|s| s.strip_suffix('>'))
            .unwrap_or(type_args.as_str());
        append!(ts, "export type Slots = {inner_type};\n");
    } else {
        ts.push_str("export type Slots = {};\n");
    }

    // Exposed type (for InstanceType and useTemplateRef)
    let has_exposed_type = summary
        .macros
        .define_expose()
        .is_some_and(|expose| expose.type_args.is_some() || expose.runtime_args.is_some());
    if let Some(expose) = summary.macros.define_expose() {
        if let Some(ref type_args) = expose.type_args {
            let inner_type = type_args
                .strip_prefix('<')
                .and_then(|s| s.strip_suffix('>'))
                .unwrap_or(type_args.as_str());
            append!(ts, "export type Exposed = {inner_type};\n");
        } else if expose.runtime_args.is_some() {
            // Runtime args are returned from __setup() to keep them in scope.
            // Use Awaited<ReturnType<...>> to handle both sync and async setup.
            ts.push_str(
                "export type Exposed = Awaited<ReturnType<typeof __setup>>[\"__vize_exposed\"];\n",
            );
        }
    }
    ts.push('\n');

    if has_emits_for_props {
        ts.push_str("type __VizeOverloadProps<TOverload> = Pick<TOverload, keyof TOverload>;\n");
        ts.push_str("type __VizeOverloadUnionRecursive<TOverload, TPartialOverload = unknown> = TOverload extends (...args: infer TArgs) => infer TReturn ? TPartialOverload extends TOverload ? never : __VizeOverloadUnionRecursive<TPartialOverload & TOverload, TPartialOverload & ((...args: TArgs) => TReturn) & __VizeOverloadProps<TOverload>> | ((...args: TArgs) => TReturn) : never;\n");
        ts.push_str("type __VizeOverloadUnion<TOverload extends (...args: any[]) => any> = Exclude<__VizeOverloadUnionRecursive<(() => never) & TOverload>, TOverload extends () => never ? never : () => never>;\n");
        ts.push_str("type __VizeOverloadParameters<T extends (...args: any[]) => any> = Parameters<__VizeOverloadUnion<T>>;\n");
        ts.push_str("type __VizeIsStringLiteral<T> = T extends string ? string extends T ? false : true : false;\n");
        ts.push_str("type __VizeParametersToFns<T extends any[]> = { [K in T[0]]: __VizeIsStringLiteral<K> extends true ? (...args: T extends [e: infer E, ...args: infer P] ? K extends E ? P : never : never) => any : never };\n");
        ts.push_str("type __EmitOptions<T> = { [K in keyof __EmitShape<T> & string]: (...args: __EmitArgs<__EmitShape<T>, K>) => any } & (__EmitShape<T> extends (...args: any[]) => any ? __VizeParametersToFns<__VizeOverloadParameters<__EmitShape<T>>> : {});\n");
        ts.push_str("type __EmitProps<T> = import('vue').EmitsToProps<__EmitOptions<T>>;\n\n");
    }

    // Default export
    ts.push_str("// ========== Default Export ==========\n");
    ts.push_str("type __VizeComponentInstance = {\n");
    if has_emits_for_props {
        ts.push_str("  $props: Props & __EmitProps<Emits>;\n");
    } else {
        ts.push_str("  $props: Props;\n");
    }
    ts.push_str("  $emit: __EmitFn<Emits>;\n");
    ts.push_str("  $slots: Slots;\n");
    if has_exposed_type {
        ts.push_str("} & Exposed;\n");
    } else {
        ts.push_str("};\n");
    }
    // For a `<script setup generic="...">` component the construct signature's
    // `$props` collapses `Props<T>` to its constraint, so a parent that extracts
    // props via `typeof Comp extends { new (): { $props } }` cannot infer `T`
    // from the passed prop values. Expose a generic functional prop-checker on
    // the default export so the parent can invoke it with the assembled props
    // object and let TypeScript infer `T` from the call (see #775). Non-generic
    // components keep the plain construct signature unchanged.
    if let Some(generic) = generic_param {
        let generic_decl = add_generic_defaults(generic);
        let generic_names = extract_generic_names(generic);
        append!(
            ts,
            "declare const __vize_component__: (new (...args: any[]) => __VizeComponentInstance) & {{ __vizeCheck: <{generic_decl}>(props: Partial<Props<{generic_names}>> & Record<string, unknown>) => void; }};\n",
        );
    } else {
        ts.push_str(
            "declare const __vize_component__: new (...args: any[]) => __VizeComponentInstance;\n",
        );
    }
    ts.push_str("export default __vize_component__;\n");

    VirtualTsOutput { code: ts, mappings }
}
