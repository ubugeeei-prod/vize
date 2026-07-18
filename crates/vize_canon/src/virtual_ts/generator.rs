mod anchors;
mod component_export;
mod emits;
mod generics;
mod global_components;
mod imports;
mod legacy_vue2;
mod options_api;
mod options_api_props_identifiers;
mod options_api_support;
mod props_anchors;
mod script_module;
mod setup_helpers;
mod setup_props;
mod spans;
mod template_refs;
use self::anchors::emit_setup_binding_anchors;
use self::component_export::emit_default_export_declaration;
use self::emits::{emit_emit_props_helper, emit_emits_type};
use self::generics::{HoistedGenericAliases, generic_injection_point, references_any_identifier};
use self::global_components::GlobalComponentPlan;
use self::imports::{
    collect_imported_names, emit_reference_path_directives, emit_reference_type_directives,
    extract_declared_name,
};
pub use self::legacy_vue2::generate_virtual_ts_with_offsets_legacy_vue2;
use self::legacy_vue2::{exposed_unwrap_helper, instance_helper, instance_suffix};
use self::options_api::{
    find_default_export_targets, find_options_api_props, generate_options_api_bridge,
    generate_options_api_variables,
};
use self::options_api_props_identifiers::PropsConstAssertions;
use self::props_anchors::emit_setup_scope_prop_anchors;
use self::setup_helpers::emit_setup_helpers;
use self::setup_props::SetupPropsPlan;
use self::spans::{
    DEFINE_COMPONENT_REF, merge_overlapping_spans, rewrite_export_default_for_module_scope,
    template_usage,
};
use super::{
    helpers::{
        IMPORT_META_AUGMENTATION, SETUP_SCOPE_HELPER_NAMES, generate_template_context,
        to_safe_identifier,
    },
    props::{
        OptionsApiPropsSource, add_generic_defaults, collect_template_prop_names,
        extract_generic_names, strip_const_modifiers,
    },
    scope::{ScopeGenerationOptions, generate_scope_closures},
    types::{
        DEFAULT_LIB_REFERENCES, VirtualTsGenerationOptions, VirtualTsOptions, VirtualTsOutput,
        VizeMapping, emit_lib_reference_directives,
    },
};
use vize_carton::{FxHashMap, FxHashSet, String, append, cstr, profile};
use vize_croquis::{Croquis, ScopeData, ScopeKind};
/// Generate virtual TypeScript from Vue SFC analysis.
///
/// This ensures compiler macros like defineProps are ONLY valid in setup scope.
pub fn generate_virtual_ts(
    summary: &Croquis,
    script_content: Option<&str>,
    template_ast: Option<&vize_relief::RootNode<'_>>,
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
    template_ast: Option<&vize_relief::RootNode<'_>>,
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
    template_ast: Option<&vize_relief::RootNode<'_>>,
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

pub(crate) fn generate_virtual_ts_with_offsets_and_checks(
    summary: &Croquis,
    script_content: Option<&str>,
    template_ast: Option<&vize_relief::RootNode<'_>>,
    script_offset: u32,
    template_offset: u32,
    options: &VirtualTsOptions,
    generation_options: VirtualTsGenerationOptions<'_>,
) -> VirtualTsOutput {
    let check_options = generation_options.check_options;
    let check_props = check_options.check_props;
    // Configured Vue dialect, used to emit dialect-aware template instance typing
    // (e.g. a Vue 2 `this`/template shape with `$listeners`,
    // `$children`, `$on`, ... that Vue 3's `ComponentPublicInstance` lacks).
    let dialect = generation_options.dialect;
    let legacy_vue2 = generation_options.legacy_vue2
        || matches!(
            dialect,
            vize_carton::config::VueVersion::V2 | vize_carton::config::VueVersion::V2_7
        );
    let _ = generation_options.template_syntax_quirks;
    let options_api = generation_options.options_api || legacy_vue2;
    let hoist_shared_preamble = generation_options.hoist_shared_preamble;
    let mut ts = String::default();
    let mut mappings: Vec<VizeMapping> = Vec::new();
    let preserve_unused_diagnostics = generation_options.preserve_unused_diagnostics;
    let (template_usage_names, has_template_scope) =
        template_usage(summary, template_ast, generation_options);
    let template_referenced_names = preserve_unused_diagnostics.then_some(&template_usage_names);
    let reference_setup_bindings_comment = if preserve_unused_diagnostics {
        "Reference setup bindings used by template generation"
    } else {
        "Reference setup bindings (used in template/CSS v-bind)"
    };
    let lib_references = generation_options
        .lib_references
        .unwrap_or(DEFAULT_LIB_REFERENCES);
    emit_lib_reference_directives(&mut ts, lib_references);
    emit_reference_path_directives(&mut ts, &options.reference_paths);
    let has_script_reference_types = emit_reference_type_directives(&mut ts, script_content);
    ts.push_str("// ============================================\n");
    ts.push_str("// Virtual TypeScript for Vue SFC Type Checking\n");
    ts.push_str("// Generated by vize\n");
    ts.push_str("// ============================================\n\n");

    // Check for generic type parameter from <script setup generic="T">
    let (generic_param, is_async) = summary
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

    if hoist_shared_preamble {
        // ImportMeta augmentation and shared type helpers live once per
        // program in the ambient helpers file (SHARED_PREAMBLE_DTS); the
        // module no longer augments global scope itself.
        ts.push_str("// Shared preamble hoisted to the program-wide __vize_helpers.d.ts\n");
    } else {
        // ImportMeta augmentation (must be at top level, before any code)
        ts.push_str(IMPORT_META_AUGMENTATION);
        ts.push('\n');
    }

    // Module scope: Extract imports, re-exports, and type declarations to module level.
    // Type declarations (interface, type, enum) must be at module level so they
    // are accessible from `export type Props = ...` outside __setup().
    ts.push_str("// ========== Module Scope (imports) ==========\n");
    if !hoist_shared_preamble {
        ts.push_str(legacy_vue2::vue_type_helpers(legacy_vue2, dialect));
        ts.push('\n');
    }

    let has_script_setup = summary
        .scopes
        .iter()
        .any(|scope| matches!(scope.kind, ScopeKind::ScriptSetup));
    let has_plain_script_scope = summary
        .scopes
        .iter()
        .any(|scope| matches!(scope.kind, ScopeKind::NonScriptSetup))
        || (script_content.is_some() && !has_script_setup);
    let named_value_exports = self::script_module::collect_normal_script_named_value_exports(
        script_content,
        has_script_setup,
        has_plain_script_scope,
    );

    // Classify the main `<script>` default export in one parse. A plain
    // `export default { ... }` (Options API shape) gets wrapped with Vue's
    // `defineComponent` so `this` inside computed/methods is typed by Vue's
    // options machinery; a class default export (class-component shape) keeps
    // its decorators on a standalone class declaration plus a
    // `const __default__ =` alias (a bare `const __default__ = class {}`
    // rewrite would move the decorators onto a class expression — TS1206).
    // Script setup virtual TS is never touched: in setup-only SFCs no rewrite
    // target exists, so the lookup is skipped entirely.
    let default_export_targets = if !has_script_setup || has_plain_script_scope {
        profile!(
            "canon.virtual_ts.find_default_export_targets",
            script_content
                .map(find_default_export_targets)
                .unwrap_or_default()
        )
    } else {
        Default::default()
    };
    let default_export_object = default_export_targets.object;
    let default_export_class = default_export_targets.class;
    let default_export_expr = default_export_targets.expr;
    if default_export_object.is_some() {
        ts.push_str(legacy_vue2::define_component_helper(legacy_vue2, dialect));
    }
    // Collect all module-level statement spans from croquis analysis once and
    // keep them sorted. Later script-body emission advances an index through
    // this list, so each source line checks only the overlapping tail instead
    // of rescanning imports/re-exports/type declarations from the start.
    let module_spans: Vec<(u32, u32)> = profile!("canon.virtual_ts.collect_module_spans", {
        let mut module_spans = Vec::new();
        for imp in &summary.import_statements {
            module_spans.push((imp.start, imp.end));
        }
        if let Some(script) = script_content {
            module_spans.extend(self::script_module::collect_line_module_import_spans(
                script,
            ));
        }
        for re in &summary.re_exports {
            module_spans.push((re.start, re.end));
        }
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

    // Re-declare SFC generics on hoisted declarations with safe defaults.
    let generic_injection: Option<(String, Vec<String>)> = generic_param.map(|g| {
        // Type aliases/interfaces cannot retain function-only `const` modifiers (TS1277).
        let defaults = strip_const_modifiers(&add_generic_defaults(g));
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
                    // Re-base the script-absolute default-export offsets onto
                    // this module span so the rewrite can wrap a plain options
                    // object with `defineComponent`, or rewrite any other shape
                    // to a bare `const __default__ = <expr>`, by slicing on AST
                    // byte offsets instead of scanning lines.
                    let span_start = start as usize;
                    let span_end = end as usize;
                    let rebase = |(export_start, inner_start, inner_end): (usize, usize, usize)| {
                        (export_start >= span_start && inner_end <= span_end).then_some((
                            export_start - span_start,
                            inner_start - span_start,
                            inner_end - span_start,
                        ))
                    };
                    let span_relative_object = default_export_object.and_then(rebase);
                    let span_relative_expr = default_export_expr.and_then(rebase);
                    let text = rewrite_export_default_for_module_scope(
                        text,
                        span_relative_object,
                        span_relative_expr,
                    );
                    ts.push_str(&text);
                    ts.push_str("void __default__;\n");
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
    let hoisted_generic_aliases =
        HoistedGenericAliases::collect(summary, script_content, generic_param);
    if let Some(aliases) = &hoisted_generic_aliases {
        aliases.emit_module_aliases(&mut ts);
    }

    let global_components =
        GlobalComponentPlan::new(summary, legacy_vue2, has_script_reference_types);
    let needs_imported_names = !options.auto_import_stubs.is_empty()
        || (global_components.enabled() && !summary.component_usages.is_empty());
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
    global_components.emit(&mut ts, summary, options, &imported_names);
    ts.push('\n');

    // For an Options API component with no `defineProps` macro, derive a real
    // `export type Props` from its runtime `props:` option so cross-file prop
    // checking is no longer a `{}` no-op. Macro-driven props (script setup) take
    // precedence and are emitted by `generate_props_type` itself.
    let options_api_props: Option<OptionsApiPropsSource> =
        if options_api && summary.macros.props().is_empty() {
            script_content.and_then(find_options_api_props)
        } else {
            None
        };
    let setup_props_plan = SetupPropsPlan::new(summary, options_api_props.as_ref());
    profile!("canon.virtual_ts.generate_props_type", {
        setup_props_plan.generate_props_type(
            &mut ts,
            summary,
            generic_param,
            options_api_props.as_ref(),
        )
    });

    // Setup scope: function that contains setup helpers and script content
    ts.push_str("// ========== Setup Scope ==========\n");
    let async_prefix = if is_async { "async " } else { "" };
    let generic_params = generic_param.map(|g| cstr!("<{g}>")).unwrap_or_default();
    append!(ts, "{async_prefix}function __setup{generic_params}() {{\n",);
    if let Some(aliases) = &hoisted_generic_aliases {
        aliases.emit_setup_aliases(&mut ts);
    }

    // Setup helpers (only valid inside setup scope)
    emit_setup_helpers(
        &mut ts,
        script_content,
        generic_param,
        hoist_shared_preamble,
    );
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
            let named_value_export_starts =
                self::script_module::collect_named_value_export_starts(script);
            let mut props_const_assertions = PropsConstAssertions::new(script, options_api);
            // Script-absolute offset right after the wrapped options object.
            let mut pending_wrap_close: Option<usize> = None;
            // Deferred class-component alias: `(class_end, name)`.
            let mut pending_class_alias: Option<(usize, &str)> = None;
            let mut emitted_default_alias = false;

            let uses_import_meta = self::script_module::emit_import_meta_polyfill(&mut ts, script);

            for raw_line in script.split('\n') {
                // Strip trailing \r for output (normalize CRLF to LF)
                let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
                // raw_line.len() includes \r if present; +1 for the \n from split
                let raw_byte_len = raw_line.len() + 1;

                // Skip lines that overlap with module-level spans (imports, re-exports, type decls)
                let line_start = src_byte_offset;
                let line_end = line_start + raw_line.len(); // use raw length for span check
                let source_token_start = line_start + line.len() - line.trim_start().len();
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

                // Close the `defineComponent(` wrap right after the wrapped
                // options object's closing brace.
                if let Some(close_offset) = pending_wrap_close
                    && close_offset > line_start
                    && close_offset <= line_end
                    && close_offset - line_start <= line.len()
                    && line.is_char_boundary(close_offset - line_start)
                {
                    let column = close_offset - line_start;
                    #[allow(clippy::disallowed_types)]
                    {
                        output_line = std::borrow::Cow::Owned(
                            cstr!("{}){}", &line[..column], &line[column..]).into(),
                        );
                    }
                    pending_wrap_close = None;
                }

                props_const_assertions.splice_output_line(&mut output_line, line_start);

                // Strip `export` from non-import lines inside setup scope
                let trimmed_line = output_line.trim_start();
                if let Some(default_expr) = trimmed_line
                    .strip_prefix("export default")
                    .filter(|rest| options_api_support::follows_default_keyword(rest))
                {
                    emitted_default_alias = true;
                    let leading_ws = &output_line[..output_line.len() - trimmed_line.len()];
                    // A class default export (the class-component shape) stays
                    // a real class declaration so `@Component()` decorators
                    // remain valid (a bare `const __default__ = class {}`
                    // rewrite moves them onto a class expression — TS1206). The
                    // `const __default__ = <Name>` alias is deferred until the
                    // class body closes.
                    let class_default =
                        if has_script_setup || matches!(output_line, std::borrow::Cow::Owned(_)) {
                            None
                        } else {
                            default_export_class.filter(
                                |&(export_start, class_start, class_end, _, _)| {
                                    export_start >= line_start
                                        && class_start >= line_start
                                        && class_start < line_end
                                        && class_end > line_start
                                        && class_start - line_start <= line.len()
                                        && line.is_char_boundary(class_start - line_start)
                                },
                            )
                        };
                    // Wrap a plain object-literal default export (Options
                    // API) with `defineComponent` so `this` in
                    // computed/methods gets Vue's instance typing. Applies
                    // only to the plain <script> block; any other shape keeps
                    // the bare `const __default__ =` rewrite.
                    let wrap_object = if class_default.is_some()
                        || has_script_setup
                        || matches!(output_line, std::borrow::Cow::Owned(_))
                    {
                        None
                    } else {
                        default_export_object.filter(|&(export_start, object_start, _)| {
                            export_start >= line_start
                                && object_start >= line_start
                                && object_start < line_end
                                && object_start - line_start <= line.len()
                                && line[object_start - line_start..].starts_with('{')
                        })
                    };
                    #[allow(clippy::disallowed_types)]
                    if let Some((class_start, class_end, name_start, name_end)) =
                        class_default.map(|(_, cs, ce, ns, ne)| (cs, ce, ns, ne))
                    {
                        // Drop the `export default ` keyword, keep the class
                        // (and any same-line trailing decorator) verbatim.
                        let class_column = class_start - line_start;
                        let name = &script[name_start..name_end];
                        output_line = std::borrow::Cow::Owned(
                            cstr!("{leading_ws}{}", &line[class_column..]).into(),
                        );
                        pending_class_alias = Some((class_end, name));
                    } else if let Some((object_column, object_end)) =
                        wrap_object.and_then(|(_, object_start, object_end)| {
                            let object_column = object_start - line_start;
                            (line.len() - default_expr.len() <= object_column)
                                .then_some((object_column, object_end))
                        })
                    {
                        let keyword_end = line.len() - default_expr.len();
                        if object_end <= line_end && object_end - line_start <= line.len() {
                            // Single-line `export default { ... }`.
                            let close_column = object_end - line_start;
                            output_line = std::borrow::Cow::Owned(
                                cstr!(
                                    "{leading_ws}const __default__ ={}{DEFINE_COMPONENT_REF}({}){}",
                                    &line[keyword_end..object_column],
                                    &line[object_column..close_column],
                                    &line[close_column..],
                                )
                                .into(),
                            );
                        } else {
                            pending_wrap_close = Some(object_end);
                            output_line = std::borrow::Cow::Owned(
                                cstr!(
                                    "{leading_ws}const __default__ ={}{DEFINE_COMPONENT_REF}({}",
                                    &line[keyword_end..object_column],
                                    &line[object_column..],
                                )
                                .into(),
                            );
                        }
                    } else {
                        output_line = std::borrow::Cow::Owned(
                            cstr!("{leading_ws}const __default__ ={}", default_expr).into(),
                        );
                    }
                } else if named_value_export_starts.contains(&(source_token_start as u32))
                    && trimmed_line.starts_with("export ")
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
                // Once the class body has closed, append the deferred
                // `const __default__ = <Name>` alias for the class component.
                if let Some((class_end, name)) = pending_class_alias
                    && class_end <= line_end
                {
                    append!(ts, "  const __default__ = {name};\n");
                    pending_class_alias = None;
                }
                let _ = gen_line_start; // suppress unused warning
                src_byte_offset += raw_byte_len;
            }
            if let Some((_, name)) = pending_class_alias.take() {
                // Defensive: the line carrying the class body's closing brace
                // was never seen; still emit the alias so `__default__` (and the
                // template bindings that read `typeof __default__`) resolve.
                append!(ts, "  const __default__ = {name};\n");
            }
            if pending_wrap_close.take().is_some() {
                // Defensive: if the object close was never emitted, close the `defineComponent(`
                // so the generated module stays parseable.
                ts.push_str("  )\n");
            }
            if emitted_default_alias {
                ts.push_str("  void __default__;\n");
            }
            let script_gen_end = ts.len();
            append!(
                ts,
                "  // @vize-map: {script_gen_start}:{script_gen_end} -> 0:{}\n\n",
                script.len()
            );

            if options_api {
                profile!(
                    "canon.virtual_ts.generate_options_api_bridge",
                    generate_options_api_bridge(&mut ts, summary, script)
                );
            }
        });
    }

    setup_props_plan.emit_artifact(&mut ts, summary);
    // Template scope (nested inside setup)
    if has_template_scope && check_options.check_template_bindings {
        profile!("canon.virtual_ts.emit_template_scope", {
            ts.push_str("  // ========== Template Scope (inherits from setup) ==========\n");

            let template_ref_unwraps = template_refs::TemplateRefUnwraps::collect(
                summary,
                options_api,
                Some(&template_usage_names),
                script_content,
            );
            template_ref_unwraps.emit_type_captures(&mut ts);

            // Semicolon prevents ASI issues when user script doesn't end with `;`
            // (e.g., `console.log(x)\n(function...)` would be parsed as a call)
            ts.push_str("  ;(function __template() {\n");

            // Shadow ref bindings with unwrapped types.
            // `var` allows reassignment (Vue templates can assign to refs).
            template_ref_unwraps.emit_template_variables(
                &mut ts,
                legacy_vue2,
                dialect,
                generic_param.is_some(),
            );

            // Vue template context (available in template expressions)
            let template_context = profile!(
                "canon.virtual_ts.generate_template_context",
                generate_template_context(options, dialect, legacy_vue2)
            );
            ts.push_str(&template_context);
            ts.push('\n');

            profile!("canon.virtual_ts.generate_props_variables", {
                setup_props_plan.generate_props_variables(
                    &mut ts,
                    summary,
                    generic_param,
                    check_props && !legacy_vue2,
                )
            });
            if options_api {
                profile!(
                    "canon.virtual_ts.generate_options_api_variables",
                    generate_options_api_variables(&mut ts, summary, options, script_content)
                );
            }
            let template_prop_names = profile!(
                "canon.virtual_ts.collect_template_prop_names",
                collect_template_prop_names(summary)
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
                            template_source: template_ast.map(|root| root.source.as_str()),
                            check_unresolved_global_components: has_script_reference_types,
                            legacy_vue2,
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
                        || global_components.keeps_unresolved_binding(name)
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
            profile!(
                "canon.virtual_ts.emit_setup_binding_anchors",
                emit_setup_binding_anchors(
                    &mut ts,
                    summary,
                    script_content,
                    template_referenced_names,
                    reference_setup_bindings_comment,
                )
            );

            ts.push_str("  })();\n");
        });
    }

    if has_template_scope && !check_options.check_template_bindings && preserve_unused_diagnostics {
        profile!(
            "canon.virtual_ts.emit_no_check_template_binding_anchors",
            emit_setup_binding_anchors(
                &mut ts,
                summary,
                script_content,
                template_referenced_names,
                reference_setup_bindings_comment,
            )
        );
    }

    emit_setup_scope_prop_anchors(
        &mut ts,
        summary,
        script_content,
        template_referenced_names,
        preserve_unused_diagnostics,
    );

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
    let mut setup_return_fields: Vec<String> = Vec::new();
    self::script_module::push_setup_return_fields(&named_value_exports, &mut setup_return_fields);
    let mut setup_artifact_return_fields = Vec::new();
    setup_props_plan.push_return_field(&mut setup_artifact_return_fields);
    setup_return_fields.extend(setup_artifact_return_fields.into_iter().map(String::from));
    if let Some(expose) = summary.macros.define_expose()
        && expose.type_args.is_none()
        && let Some(runtime_args) = expose.runtime_args.as_ref()
    {
        append!(ts, "\n  const __vize_exposed = ({runtime_args});\n");
        setup_return_fields.push("__vize_exposed".into());
    }
    if let Some(runtime_args) = define_emits_runtime_args {
        append!(
            ts,
            "\n  const __vize_emit_options = ({runtime_args});\n  const __vize_emits = defineEmits(__vize_emit_options);\n"
        );
        setup_return_fields.push("__vize_emit_options".into());
        setup_return_fields.push("__vize_emits".into());
    }
    setup_props_plan.emit_options_api_artifact(&mut ts, options_api_props.as_ref());
    if !setup_return_fields.is_empty() {
        append!(ts, "\n  return {{ {} }};\n", setup_return_fields.join(", "));
    }

    // Close setup function
    ts.push_str("}\n\n");

    // Invoke setup to keep diagnostics inside the generated setup body.
    ts.push_str("// Invoke setup to verify types\n");
    self::script_module::emit_setup_invocation_and_exports(&mut ts, &named_value_exports);

    setup_props_plan.emit_module_export(&mut ts, options_api_props.as_ref());

    let emits_info = emit_emits_type(
        &mut ts,
        summary,
        generic_param,
        define_emits_runtime_args.is_some(),
    );

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

    emit_emit_props_helper(&mut ts, &emits_info, hoist_shared_preamble);

    // Default export
    ts.push_str("// ========== Default Export ==========\n");
    ts.push_str(instance_helper(legacy_vue2, dialect));
    if has_exposed_type {
        ts.push_str(exposed_unwrap_helper(legacy_vue2, dialect));
    }
    let generic_component_params = setup_props_plan.generic_component_params(generic_param);
    ts.push_str("type __VizeComponentInstance = {\n");
    setup_props_plan.emit_component_props_field(
        &mut ts,
        emits_info.has_emits_for_props,
        generic_component_params
            .as_ref()
            .map(|(_, names)| names.as_str()),
    );
    ts.push_str("  $emit: __EmitFn<Emits>;\n");
    ts.push_str("  $slots: Slots;\n");
    ts.push_str(instance_suffix(legacy_vue2, dialect, has_exposed_type));
    ts.push_str(
        "type __VizeComponentConstructor = new (...args: any[]) => __VizeComponentInstance;\n",
    );
    if let Some((generic_decl, generic_names)) = &generic_component_params {
        let emit_props_field = if emits_info.has_emits_for_props {
            " & __EmitProps<Emits>"
        } else {
            ""
        };
        append!(
            ts,
            "type __VizeGenericComponentConstructor = new <{generic_decl}>(...args: any[]) => {{\n  $props: __VizeComponentProps<Props<{generic_names}>>{emit_props_field};\n  $emit: __EmitFn<Emits>;\n  $slots: Slots;\n"
        );
        ts.push_str(instance_suffix(legacy_vue2, dialect, has_exposed_type));
    }
    ts.push_str("type __VizeVueComponentOptions = {\n");
    ts.push_str("  name?: string;\n");
    ts.push_str("  __name?: string;\n");
    ts.push_str("  __file?: string;\n");
    ts.push_str("  __vccOpts?: any;\n");
    ts.push_str("  props?: any;\n");
    ts.push_str("  emits?: any;\n");
    ts.push_str("  slots?: any;\n");
    ts.push_str("  setup?: any;\n");
    ts.push_str("  render?: Function;\n");
    ts.push_str("  components?: any;\n");
    ts.push_str("  directives?: any;\n");
    ts.push_str("  inheritAttrs?: boolean;\n");
    ts.push_str("  compatConfig?: any;\n");
    ts.push_str("  call?: (this: unknown, ...args: unknown[]) => never;\n");
    ts.push_str("  __isFragment?: never;\n");
    ts.push_str("  __isTeleport?: never;\n");
    ts.push_str("  __isSuspense?: never;\n");
    ts.push_str("  __defaults?: any;\n");
    ts.push_str("  __vapor?: boolean;\n");
    ts.push_str("  __multiRoot?: boolean;\n");
    ts.push_str("  __isKeepAlive?: boolean;\n");
    ts.push_str("  __isBuiltIn?: boolean;\n");
    ts.push_str("};\n");
    emit_default_export_declaration(
        &mut ts,
        &emits_info,
        generic_component_params
            .as_ref()
            .map(|(decl, names)| (decl.as_str(), names.as_str())),
    );
    ts.push_str("export default __vize_component__;\n");

    VirtualTsOutput { code: ts, mappings }
}
