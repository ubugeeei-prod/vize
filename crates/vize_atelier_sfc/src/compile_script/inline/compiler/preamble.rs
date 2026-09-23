use vize_carton::{String, profile};

use crate::module_map::Runs;

use super::super::super::TemplateParts;
use super::super::super::function_mode::imports::dedupe_imports_with_origins;
use super::super::super::import_utils::import_block_has_local_from;
use super::parser::parse_script_content;
use super::trace::{Tracer, traced_sections};

/// Per import: its text's provenance and its statement's `.vue` start.
type ImportOrigin = (Runs, Option<usize>);

pub(super) struct PreambleState {
    pub(super) setup_return_imports: Vec<String>,
    pub(super) has_default_export: bool,
}

#[expect(clippy::too_many_arguments, reason = "independent compile inputs")]
pub(super) fn emit_preamble(
    output: &mut vize_carton::Vec<u8>,
    template: &TemplateParts<'_>,
    user_imports: &[String],
    ts_declarations: &[String],
    preserved_normal_script: Option<&String>,
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
    tracer: &mut Tracer<'_>,
) -> PreambleState {
    // mergeDefaults import comes first if needed
    if needs_merge_defaults {
        output.extend_from_slice(b"import { mergeDefaults as _mergeDefaults } from 'vue'\n");
    }

    // useSlots import if defineSlots was used
    if has_define_slots {
        output.extend_from_slice(b"import { useSlots as _useSlots } from 'vue'\n");
    }

    // useModel import if defineModel was used; mergeModels is added on the same
    // import line when defineModel coexists with defineProps/defineEmits.
    if has_define_model {
        if needs_merge_models {
            output.extend_from_slice(
                b"import { useModel as _useModel, mergeModels as _mergeModels } from 'vue'\n",
            );
        } else {
            output.extend_from_slice(b"import { useModel as _useModel } from 'vue'\n");
        }
    }

    // useCssVars import if style has v-bind()
    if has_css_vars {
        let include_define_component = is_ts && !is_async && !is_vapor;
        if import_block_has_local_from(template.imports, "vue", "_unref") {
            if include_define_component {
                output.extend_from_slice(
                    b"import { useCssVars as _useCssVars, defineComponent as _defineComponent } from 'vue'\n",
                );
            } else {
                output.extend_from_slice(b"import { useCssVars as _useCssVars } from 'vue'\n");
            }
        } else if include_define_component {
            output.extend_from_slice(
                b"import { useCssVars as _useCssVars, unref as _unref, defineComponent as _defineComponent } from 'vue'\n",
            );
        } else {
            output.extend_from_slice(
                b"import { useCssVars as _useCssVars, unref as _unref } from 'vue'\n",
            );
        }
    }

    if has_css_modules {
        output.extend_from_slice(b"import { useCssModule as _useCssModule } from 'vue'\n");
    }

    // Component helper import (skip if already emitted with withAsyncContext)
    if is_vapor && !is_async {
        if needs_vapor_setup_context {
            output.extend_from_slice(
                b"import { defineVaporComponent as _defineVaporComponent, getCurrentInstance as _getCurrentInstance, proxyRefs as _proxyRefs } from 'vue'\n",
            );
        } else {
            output.extend_from_slice(
                b"import { defineVaporComponent as _defineVaporComponent } from 'vue'\n",
            );
        }
    } else if is_ts && !is_async && (!has_css_vars || is_vapor) {
        output.extend_from_slice(b"import { defineComponent as _defineComponent } from 'vue'\n");
    }

    // Template imports (Vue helpers)
    if !template.imports.is_empty() {
        output.extend_from_slice(template.imports.as_bytes());
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

    // User imports (after hoisted consts) - deduplicate to avoid "already declared" errors.
    //
    // Imports from BOTH the normal `<script>` block and `<script setup>` are merged and
    // deduplicated here. `dedupe_imports` keys on `source::local`, so an import that appears
    // in both blocks (or the same side-effect import) is emitted exactly once. The normal
    // script's own `import` statements are stripped from its preserved body below, so this is
    // the single emission point for every user import. See #993 (side-effect imports running
    // twice when an SFC has both `<script>` and `<script setup>`).
    let (normal_script_imports, mut origins) = preserved_normal_script
        .map(|script| normal_script_imports(script, is_ts, tracer))
        .unwrap_or_default();
    origins.resize(normal_script_imports.len(), (Runs::default(), None));
    let mut combined_imports: Vec<String> = normal_script_imports;
    combined_imports.extend_from_slice(user_imports);
    if let Some(trace) = tracer.trace() {
        origins.append(&mut trace.imports);
    }
    let deduped_imports = profile!(
        "atelier.script_inline.dedupe_imports",
        dedupe_imports_with_origins(&combined_imports, is_ts)
    );
    let setup_return_imports: Vec<String> = deduped_imports
        .iter()
        .map(|import| import.text.clone())
        .collect();
    if !deduped_imports.is_empty() && !template.hoisted.is_empty() {
        ensure_blank_line(output);
    }
    let traced = origins.len() == combined_imports.len();
    for import in &deduped_imports {
        if let Some((runs, statement)) = origins.get(import.index).filter(|_| traced)
            && let Some(authored) = combined_imports.get(import.index)
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
    if !deduped_imports.is_empty()
        && ts_declarations.is_empty()
        && preserved_normal_script.is_none()
    {
        ensure_blank_line(output);
    }

    // Output TypeScript declarations (interfaces, types) after user imports, before export default
    if !ts_declarations.is_empty() {
        output.push(b'\n');
        let declaration_runs = tracer
            .trace()
            .map(|trace| std::mem::take(&mut trace.ts_declarations))
            .unwrap_or_default();
        for (index, decl) in ts_declarations.iter().enumerate() {
            tracer.copy(output.len(), declaration_runs.get(index));
            output.extend_from_slice(decl.as_bytes());
            output.push(b'\n');
        }
        output.push(b'\n');
    }

    // Normal script content goes AFTER imports/hoisted, BEFORE component definition
    // This matches Vue's @vue/compiler-sfc output order
    let has_default_export = if let Some(normal_script) = preserved_normal_script {
        // Strip the block's own `import` statements: they were already merged into the
        // deduplicated import emission above. Leaving them here would emit each import twice,
        // re-running any top-level side effects in the imported module (#993).
        let (body, body_runs) = strip_import_statements(normal_script);
        output.push(b'\n');
        let normal = tracer.trace().and_then(|trace| trace.normal.clone());
        tracer.copy(
            output.len(),
            normal.map(|normal| body_runs.compose(&normal)).as_ref(),
        );
        output.extend_from_slice(body.as_bytes());
        output.push(b'\n');
        normal_script.contains("const __default__")
    } else {
        false
    };

    PreambleState {
        setup_return_imports,
        has_default_export,
    }
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

fn ensure_blank_line(output: &mut vize_carton::Vec<u8>) {
    match output.as_slice() {
        bytes if bytes.ends_with(b"\n\n") => {}
        bytes if bytes.ends_with(b"\n") => output.push(b'\n'),
        _ => output.extend_from_slice(b"\n\n"),
    }
}
