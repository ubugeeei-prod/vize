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
    // Imports from the preserved normal `<script>` are merged into the same
    // deduped emit so each user import appears exactly once; their statements
    // are stripped from the appended normal-script body below (#993).
    let normal_script_imports = preserved_normal_script
        .map(|script| parse_script_content(script, is_ts).0)
        .unwrap_or_default();
    let mut all_user_imports = user_imports.to_vec();
    all_user_imports.extend(normal_script_imports.iter().cloned());
    let deduped_imports = profile!(
        "atelier.script_inline.dedupe_imports",
        dedupe_imports(&all_user_imports, is_ts)
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
        output.push(b'\n');
        // Its `import` statements were already emitted via the deduped
        // import block above; keeping them here would duplicate them in
        // the compiled module (#993).
        let stripped = strip_import_statements(normal_script);
        output.extend_from_slice(stripped.trim_end_matches('\n').as_bytes());
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

fn ensure_blank_line(output: &mut vize_carton::Vec<u8>) {
    match output.as_slice() {
        bytes if bytes.ends_with(b"\n\n") => {}
        bytes if bytes.ends_with(b"\n") => output.push(b'\n'),
        _ => output.extend_from_slice(b"\n\n"),
    }
}

/// Remove top-level `import` statements from a preserved normal `<script>`
/// body. They are emitted through the deduped import block instead, so
/// leaving them in place would duplicate them in the compiled output (#993).
/// Mirrors the import detection in `parser::parse_script_content`, including
/// its template-literal guard.
fn strip_import_statements(content: &str) -> String {
    let mut out = String::default();
    let mut in_import = false;
    let mut in_template_literal = false;

    for line in content.lines() {
        let backtick_count = line
            .chars()
            .fold((0, false), |(count, escaped), c| {
                if escaped {
                    (count, false)
                } else if c == '\\' {
                    (count, true)
                } else if c == '`' {
                    (count + 1, false)
                } else {
                    (count, false)
                }
            })
            .0;
        let was_in_template_literal = in_template_literal;
        if backtick_count % 2 == 1 {
            in_template_literal = !in_template_literal;
        }

        if was_in_template_literal {
            out.push_str(line);
            out.push('\n');
            continue;
        }

        let trimmed = line.trim();

        if in_import {
            if trimmed.ends_with(';') || (trimmed.contains(" from ") && !trimmed.ends_with(',')) {
                in_import = false;
            }
            continue;
        }

        if trimmed.starts_with("import ") {
            // Single-line side-effect import (no `from` clause)
            if !trimmed.contains(" from ") && (trimmed.contains('\'') || trimmed.contains('"')) {
                continue;
            }
            if !(trimmed.ends_with(';') || (trimmed.contains(" from ") && !trimmed.ends_with(',')))
            {
                in_import = true;
            }
            continue;
        }

        out.push_str(line);
        out.push('\n');
    }

    out
}
