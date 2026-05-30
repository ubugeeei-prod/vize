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
    has_define_slots: bool,
    has_css_vars: bool,
    needs_vapor_setup_context: bool,
    vapor_render_alias: Option<&str>,
    is_vapor: bool,
    is_ts: bool,
    is_async: bool,
) -> PreambleState {
    // @vue/compiler-sfc groups the script-level Vue helper imports
    // (`mergeDefaults`, `useSlots`, `useModel`, `useCssVars`, `defineComponent`, …)
    // into a single `import ... from 'vue'` statement. Collect the active helpers
    // and emit one combined import so multi-helper output matches upstream — e.g.
    // typed `defineModel<T>()` pulls in both `useModel` and `defineComponent`.
    let mut script_helpers: Vec<&str> = Vec::new();

    // mergeDefaults comes first if needed
    if needs_merge_defaults {
        script_helpers.push("mergeDefaults as _mergeDefaults");
    }

    // useSlots if defineSlots was used
    if has_define_slots {
        script_helpers.push("useSlots as _useSlots");
    }

    // useModel if defineModel was used
    if has_define_model {
        script_helpers.push("useModel as _useModel");
    }

    // useCssVars if style has v-bind() (pulling in unref unless already imported)
    if has_css_vars {
        script_helpers.push("useCssVars as _useCssVars");
        if !import_block_has_local_from(template.imports, "vue", "_unref") {
            script_helpers.push("unref as _unref");
        }
    }

    // Component helper (skip defineComponent if it is emitted with withAsyncContext)
    if is_vapor && !is_async {
        if needs_vapor_setup_context {
            script_helpers.push("defineVaporComponent as _defineVaporComponent");
            script_helpers.push("getCurrentInstance as _getCurrentInstance");
            script_helpers.push("proxyRefs as _proxyRefs");
        } else {
            script_helpers.push("defineVaporComponent as _defineVaporComponent");
        }
    } else if is_ts && !is_async {
        script_helpers.push("defineComponent as _defineComponent");
    }

    if !script_helpers.is_empty() {
        output.extend_from_slice(b"import { ");
        output.extend_from_slice(script_helpers.join(", ").as_bytes());
        output.extend_from_slice(b" } from 'vue'\n");
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
