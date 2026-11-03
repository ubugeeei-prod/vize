use vize_carton::{String, profile};

use crate::compile_script::erasure::Erasure;
use crate::compile_script::runtime_imports::RuntimeModule;
use crate::module_map::Runs;

use super::super::super::TemplateParts;
use super::super::super::function_mode::imports::{DedupedImport, dedupe_imports_with_origins};
use super::super::super::import_utils::import_block_has_local_from;
use super::parser::parse_script_content;
use super::trace::{Tracer, traced_sections};

/// Per import: its text's provenance and its statement's `.vue` start.
type ImportOrigin = (Runs, Option<usize>);

/// The module's user imports (both blocks, deduplicated) and the normal
/// `<script>` body they were cut from.
pub(super) struct AuthoredHead {
    combined: Vec<String>,
    imports: Vec<DedupedImport>,
    origins: Vec<ImportOrigin>,
    /// The normal `<script>` without its imports, and its `.vue` provenance.
    pub(super) normal_body: Option<(String, Option<Runs>)>,
    pub(super) has_default_export: bool,
}

impl AuthoredHead {
    /// User imports from BOTH the normal `<script>` and `<script setup>`,
    /// merged and deduplicated: `dedupe_imports` keys on `source::local`, so
    /// an import in both blocks (or a repeated side-effect import) is emitted
    /// once. The normal script's own `import` statements are stripped from
    /// its body, so this is the single emission point for every user import
    /// (#993: side-effect imports ran twice).
    pub(super) fn collect(
        user_imports: &[String],
        preserved_normal_script: Option<&String>,
        is_ts: bool,
        tracer: &mut Tracer<'_>,
    ) -> Self {
        let (normal_script_imports, mut origins) = preserved_normal_script
            .map(|script| normal_script_imports(script, is_ts, tracer))
            .unwrap_or_default();
        origins.resize(normal_script_imports.len(), (Runs::default(), None));
        let mut combined: Vec<String> = normal_script_imports;
        combined.extend_from_slice(user_imports);
        if let Some(trace) = tracer.trace() {
            origins.append(&mut trace.imports);
        }
        let imports = profile!(
            "atelier.script_inline.dedupe_imports",
            dedupe_imports_with_origins(&combined, is_ts)
        );
        let normal = tracer.trace().and_then(|trace| trace.normal.clone());
        let normal_body = preserved_normal_script.map(|script| {
            let (body, body_runs) = strip_import_statements(script);
            (body, normal.map(|normal| body_runs.compose(&normal)))
        });
        Self {
            combined,
            imports,
            origins,
            normal_body,
            has_default_export: preserved_normal_script
                .is_some_and(|script| script.contains("const __default__")),
        }
    }

    pub(super) fn import_texts(&self) -> Vec<String> {
        self.imports
            .iter()
            .map(|import| import.text.clone())
            .collect()
    }

    /// Drop the specifiers `erasure` saw no value use of. `locals` holds
    /// each import's value bindings, parallel to [`Self::import_texts`].
    pub(super) fn elide(&mut self, erasure: &Erasure, locals: &[Vec<String>]) {
        let mut index = 0;
        self.imports.retain_mut(|import| {
            let used = locals
                .get(index)
                .is_some_and(|names| names.iter().all(|name| erasure.uses(name)));
            index += 1;
            if used {
                return true;
            }
            match erasure.elide(&import.text) {
                Some(text) if text == import.text => true,
                Some(text) => {
                    import.text = text;
                    import.verbatim = false;
                    true
                }
                None => false,
            }
        });
    }
}

#[expect(clippy::too_many_arguments, reason = "independent compile inputs")]
pub(super) fn emit_preamble(
    output: &mut Vec<u8>,
    template: &TemplateParts<'_>,
    head: &AuthoredHead,
    ts_declarations: &[String],
    declaration_runs: &[Runs],
    needs_merge_defaults: bool,
    has_define_model: bool,
    needs_merge_models: bool,
    has_define_slots: bool,
    has_css_vars: bool,
    has_css_modules: bool,
    needs_vapor_setup_context: bool,
    vapor_render_alias: Option<&str>,
    is_vapor: bool,
    is_ts: bool,
    is_async: bool,
    runtime: &RuntimeModule<'_>,
    tracer: &mut Tracer<'_>,
) -> usize {
    // mergeDefaults import comes first if needed
    if needs_merge_defaults {
        runtime.push_import(output, "mergeDefaults as _mergeDefaults");
    }

    // useSlots import if defineSlots was used
    if has_define_slots {
        runtime.push_import(output, "useSlots as _useSlots");
    }

    // useModel import if defineModel was used; mergeModels is added on the same
    // import line when defineModel coexists with defineProps/defineEmits.
    if has_define_model {
        if needs_merge_models {
            runtime.push_import(output, "useModel as _useModel, mergeModels as _mergeModels");
        } else {
            runtime.push_import(output, "useModel as _useModel");
        }
    }

    // CSS variables need the helper for the component's effective renderer.
    if has_css_vars {
        let include_define_component = is_ts && !is_async && !is_vapor;
        if is_vapor {
            if import_block_has_local_from(template.imports, runtime.name, "_unref") {
                runtime.push_import(output, "useVaporCssVars as _useVaporCssVars");
            } else {
                runtime.push_import(
                    output,
                    "useVaporCssVars as _useVaporCssVars, unref as _unref",
                );
            }
        } else if import_block_has_local_from(template.imports, runtime.name, "_unref") {
            if include_define_component {
                runtime.push_import(
                    output,
                    "useCssVars as _useCssVars, defineComponent as _defineComponent",
                );
            } else {
                runtime.push_import(output, "useCssVars as _useCssVars");
            }
        } else if include_define_component {
            runtime.push_import(
                output,
                "useCssVars as _useCssVars, unref as _unref, defineComponent as _defineComponent",
            );
        } else {
            runtime.push_import(output, "useCssVars as _useCssVars, unref as _unref");
        }
    }

    if has_css_modules {
        runtime.push_import(output, "useCssModule as _useCssModule");
    }

    // Component helper import (skip if already emitted with withAsyncContext)
    if is_vapor && !is_async {
        if needs_vapor_setup_context {
            runtime.push_import(output, "defineVaporComponent as _defineVaporComponent, getCurrentInstance as _getCurrentInstance, proxyRefs as _proxyRefs");
        } else {
            runtime.push_import(output, "defineVaporComponent as _defineVaporComponent");
        }
    } else if is_ts && !is_async && (!has_css_vars || is_vapor) {
        runtime.push_import(output, "defineComponent as _defineComponent");
    }

    // Template imports (Vue helpers)
    if !template.imports.is_empty() {
        output.extend_from_slice(template.imports.as_bytes());
    }
    // Every import before this offset is the compiler's own.
    let generated_imports_end = output.len();
    if !template.imports.is_empty() {
        ensure_blank_line(output);
    }

    // Template hoisted consts (e.g., const _hoisted_1 = { class: "..." })
    // Must come BEFORE user imports to match Vue's output order
    if !template.hoisted.is_empty() {
        ensure_blank_line(output);
        output.extend_from_slice(template.hoisted.as_bytes());
    }

    if !template.render_fn.is_empty() {
        output.push(b'\n');
        output.extend_from_slice(template.render_fn.as_bytes());
        if let Some(alias) = vapor_render_alias {
            output.extend_from_slice(b"const ");
            output.extend_from_slice(alias.as_bytes());
            output.extend_from_slice(b" = render\n");
        }
    }

    // User imports (after hoisted consts).
    if !head.imports.is_empty() && !template.hoisted.is_empty() {
        ensure_blank_line(output);
    }
    let traced = head.origins.len() == head.combined.len();
    for import in &head.imports {
        if let Some((runs, statement)) = head.origins.get(import.index).filter(|_| traced)
            && let Some(authored) = head.combined.get(import.index)
        {
            let authored = authored.as_str();
            if import.verbatim {
                let lead = authored.len() - authored.trim_start().len();
                tracer.copy(output.len(), Some(&runs.slice(lead, authored.trim().len())));
            } else {
                tracer.point(output.len(), *statement);
            }
        }
        output.extend_from_slice(import.text.as_bytes());
    }
    if !head.imports.is_empty() && ts_declarations.is_empty() && head.normal_body.is_none() {
        ensure_blank_line(output);
    }

    // Output TypeScript declarations (interfaces, types) after user imports, before export default
    if !ts_declarations.is_empty() {
        output.push(b'\n');
        for (index, decl) in ts_declarations.iter().enumerate() {
            tracer.copy(output.len(), declaration_runs.get(index));
            output.extend_from_slice(decl.as_bytes());
            output.push(b'\n');
        }
        output.push(b'\n');
    }

    // Normal script content goes AFTER imports/hoisted, BEFORE component definition
    // This matches Vue's @vue/compiler-sfc output order. Its own imports were
    // merged into the import emission above (#993).
    if let Some((body, runs)) = &head.normal_body {
        output.push(b'\n');
        tracer.copy(output.len(), runs.as_ref());
        output.extend_from_slice(body.as_bytes());
        output.push(b'\n');
    }
    generated_imports_end
}

/// Remove top-level `import` statements (single-line, multi-line, and side-effect forms)
/// from a preserved `<script>` body, keeping every other line verbatim. Import detection
/// mirrors the logic in [`parse_script_content`]; surrounding blank lines left behind by a
/// removed leading/trailing import are trimmed so the emitted body keeps the same spacing
/// the verbatim block had (the caller frames the body with its own newlines).
/// The kept lines' provenance in `content` is returned beside the body.
fn strip_import_statements(content: &str) -> (String, Runs) {
    let mut out = String::with_capacity(content.len());
    let mut runs = Runs::default();
    let mut in_import = false;
    for line in content.lines() {
        let trimmed = line.trim();

        if in_import {
            // Continuation lines of a multi-line import: an import ends on a line that
            // terminates with `;` or contains the `from` clause without a trailing comma.
            if trimmed.ends_with(';') || (trimmed.contains(" from ") && !trimmed.ends_with(',')) {
                in_import = false;
            }
            continue;
        }

        if trimmed.starts_with("import ") {
            // Side-effect import (no `from`, single-line): `import './reset.css'`.
            if !trimmed.contains(" from ") && (trimmed.contains('\'') || trimmed.contains('"')) {
                continue;
            }
            // Single-line named/default import completes on this line.
            if trimmed.ends_with(';') || (trimmed.contains(" from ") && !trimmed.ends_with(',')) {
                continue;
            }
            // Otherwise the import spans multiple lines; skip until it closes.
            in_import = true;
            continue;
        }

        runs.copy(
            out.len(),
            line.as_ptr() as usize - content.as_ptr() as usize,
            line.len(),
        );
        out.push_str(line);
        out.push('\n');
    }

    let lead = out.len() - out.trim_start_matches('\n').len();
    let body: String = out.trim_matches('\n').into();
    let body_runs = runs.slice(lead, body.len());
    (body, body_runs)
}

/// The normal `<script>`'s imports, with their provenance in `.vue` offsets
/// when the compile traces one (empty otherwise).
fn normal_script_imports(
    script: &str,
    is_ts: bool,
    tracer: &mut Tracer<'_>,
) -> (Vec<String>, Vec<ImportOrigin>) {
    let normal = tracer.trace().and_then(|trace| trace.normal.clone());
    if let Some(normal) = normal
        && let Some(((imports, _, _), sections)) = traced_sections(script, is_ts, None, false)
    {
        let origins = sections
            .imports
            .into_iter()
            .map(|(runs, statement)| (runs.compose(&normal), normal.lookup(statement)))
            .collect();
        return (imports, origins);
    }
    (parse_script_content(script, is_ts, None).0, Vec::new())
}

fn ensure_blank_line(output: &mut Vec<u8>) {
    match output.as_slice() {
        bytes if bytes.ends_with(b"\n\n") => {}
        bytes if bytes.ends_with(b"\n") => output.push(b'\n'),
        _ => output.extend_from_slice(b"\n\n"),
    }
}
