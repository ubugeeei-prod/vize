use vize_carton::{profile, String};

use super::super::super::function_mode::dedupe_imports;
use super::super::super::TemplateParts;
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

    // useModel import if defineModel was used
    if has_define_model {
        output.extend_from_slice(b"import { useModel as _useModel } from 'vue'\n");
    }

    // useCssVars import if style has v-bind()
    if has_css_vars {
        output.extend_from_slice(
            b"import { useCssVars as _useCssVars, unref as _unref } from 'vue'\n",
        );
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
    } else if is_ts && !is_async {
        output.extend_from_slice(b"import { defineComponent as _defineComponent } from 'vue'\n");
    }

    // Template imports (Vue helpers)
    if !template.imports.is_empty() {
        output.extend_from_slice(template.imports.as_bytes());
        output.push(b'\n');
    }

    // Template hoisted consts (e.g., const _hoisted_1 = { class: "..." })
    // Must come BEFORE user imports to match Vue's output order
    if !template.hoisted.is_empty() {
        output.push(b'\n');
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

    // User imports (after hoisted consts) - deduplicate to avoid "already declared" errors
    let deduped_imports = profile!(
        "atelier.script_inline.dedupe_imports",
        dedupe_imports(user_imports, is_ts)
    );
    let normal_script_imports = preserved_normal_script
        .map(|script| parse_script_content(script, is_ts).0)
        .unwrap_or_default();
    let mut setup_return_imports = deduped_imports.clone();
    setup_return_imports.extend(normal_script_imports.iter().cloned());
    for import in &deduped_imports {
        output.extend_from_slice(import.as_bytes());
    }

    // Output TypeScript declarations (interfaces, types) after user imports, before export default
    if !ts_declarations.is_empty() {
        output.push(b'\n');
        for decl in ts_declarations {
            output.extend_from_slice(decl.as_bytes());
            output.push(b'\n');
        }
    }

    // Normal script content goes AFTER imports/hoisted, BEFORE component definition
    // This matches Vue's @vue/compiler-sfc output order
    let has_default_export = if let Some(normal_script) = preserved_normal_script {
        output.push(b'\n');
        output.extend_from_slice(normal_script.as_bytes());
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
