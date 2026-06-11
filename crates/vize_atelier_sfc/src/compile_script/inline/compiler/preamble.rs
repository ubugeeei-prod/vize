use vize_carton::{String, profile};

use super::super::super::TemplateParts;
use super::super::super::function_mode::dedupe_imports;
use super::super::super::import_utils::import_block_has_local_from;
use super::parser::parse_script_content;

pub(super) struct PreambleState {
    pub(super) setup_return_imports: Vec<String>,
    pub(super) has_default_export: bool,
}

#[allow(clippy::too_many_arguments)]
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
    let normal_script_imports = preserved_normal_script
        .map(|script| parse_script_content(script, is_ts, None).0)
        .unwrap_or_default();
    let mut combined_imports: Vec<String> = normal_script_imports;
    combined_imports.extend_from_slice(user_imports);
    let deduped_imports = profile!(
        "atelier.script_inline.dedupe_imports",
        dedupe_imports(&combined_imports, is_ts)
    );
    let setup_return_imports = deduped_imports.clone();
    if !deduped_imports.is_empty() && !template.hoisted.is_empty() {
        ensure_blank_line(output);
    }
    for import in &deduped_imports {
        output.extend_from_slice(import.as_bytes());
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
        for decl in ts_declarations {
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
        let body = strip_import_statements(normal_script);
        output.push(b'\n');
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
fn strip_import_statements(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
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

        out.push_str(line);
        out.push('\n');
    }

    out.trim_matches('\n').into()
}

fn ensure_blank_line(output: &mut vize_carton::Vec<u8>) {
    match output.as_slice() {
        bytes if bytes.ends_with(b"\n\n") => {}
        bytes if bytes.ends_with(b"\n") => output.push(b'\n'),
        _ => output.extend_from_slice(b"\n\n"),
    }
}
