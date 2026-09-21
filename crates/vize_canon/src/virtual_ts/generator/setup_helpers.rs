//! Setup-scope compiler macro helper emission.
//!
//! Generic SFCs need a narrower `defineProps<T>()` boolean-prop model than the
//! shared helper can express safely. The shared conditional boolean-key helper
//! is intentionally left in place for non-generic SFCs, while this module uses
//! the parsed OXC type AST to pass only concrete local boolean keys for generic
//! setup scopes.

use vize_carton::{CompactString, FxHashSet, String, append};
use vize_croquis::Croquis;
use vize_relief::RootNode;

mod boolean_keys;
mod declarations;
mod macro_results;
pub(super) use declarations::SetupHelperPlan;
mod template_ref_registry;

use boolean_keys::{DefinePropsBooleanKeys, collect_define_props_boolean_keys};
use template_ref_registry::template_ref_registry;

pub(super) struct SetupHelperComponentContext<'a> {
    pub(super) helpers: &'a SetupHelperPlan,
    pub(super) summary: &'a Croquis,
    pub(super) options: &'a crate::virtual_ts::types::VirtualTsOptions,
    pub(super) syntactic_type_only_imported_names: &'a FxHashSet<CompactString>,
}

pub(super) fn define_emits_runtime_args(summary: &Croquis) -> Option<&String> {
    let emits = summary.macros.define_emits()?;
    emits
        .type_args
        .is_none()
        .then_some(emits.runtime_args.as_ref())
        .flatten()
}

pub(super) fn emit_return_artifacts(
    ts: &mut String,
    summary: &Croquis,
    fields: &mut Vec<String>,
    preserve_authored_component: bool,
) {
    crate::virtual_ts::model_types::emit_defaults_artifact(ts, summary, fields);
    if preserve_authored_component && !fields.iter().any(|field| field == "__default__") {
        fields.push("__default__".into());
    }
    if let Some(expose) = summary.macros.define_expose()
        && expose.type_args.is_none()
        && let Some(runtime_args) = expose.runtime_args.as_ref()
    {
        append!(*ts, "\n  const __vize_exposed = ({runtime_args});\n");
        fields.push("__vize_exposed".into());
    }
    if let Some(runtime_args) = define_emits_runtime_args(summary) {
        append!(
            *ts,
            "\n  const __vize_emit_options = ({runtime_args});\n  const __vize_emits = defineEmits(__vize_emit_options);\n"
        );
        fields.push("__vize_emit_options".into());
        fields.push("__vize_emits".into());
    }
}

pub(super) fn emit_setup_helpers(
    ts: &mut String,
    component_context: SetupHelperComponentContext<'_>,
    script_content: Option<&str>,
    generic_param: Option<&str>,
    hoist_shared_preamble: bool,
    template_ast: Option<&RootNode<'_>>,
) -> bool {
    // Static `ref="name"` attributes on plain elements, keyed for
    // `useTemplateRef` (#3896): the registry exists only to retype this
    // scope's shim, so it is collected here rather than by the caller.
    let registry = template_ref_registry(
        component_context.summary,
        component_context.options,
        script_content,
        template_ast,
        component_context.syntactic_type_only_imported_names,
    );
    let template_refs = registry.as_ref().map(|registry| registry.body.as_str());
    if let Some(registry) = registry.as_ref() {
        let dom_ref_helper = if registry.includes_dom_element {
            "  type __VizeDomElement<_Tag extends string, _Svg extends boolean = false> = _Svg extends true ? (_Tag extends keyof SVGElementTagNameMap ? SVGElementTagNameMap[_Tag] : Element) : (_Tag extends keyof HTMLElementTagNameMap ? HTMLElementTagNameMap[_Tag] : Element);\n"
        } else {
            ""
        };
        let component_ref_helper = if registry.includes_component {
            "  type __VizeTemplateComponentRef<_C> = _C extends abstract new (...args: any[]) => infer _I ? _I : _C extends (props: any, ctx: any, expose: (exposed: infer _E) => any, ...args: any[]) => any ? NonNullable<_E> : any;\n"
        } else {
            ""
        };
        // `NativeElements` maps tags to their *props* for template checking;
        // a template ref holds the mounted DOM node, so the registry resolves
        // through the DOM tag-name maps instead (#3896).
        //
        // `_Svg` selects the map, and neither branch falls back to the other.
        // The two overlap on `a`, `script`, `style` and `title`, so an
        // HTML-first lookup would pin `<svg><a ref="link" /></svg>` to
        // `HTMLAnchorElement`, whose `href` is a `string` where `SVGAElement`
        // has an `SVGAnimatedString`; symmetrically, an SVG-map fallback in the
        // HTML branch would hand an element the parser placed in the HTML
        // namespace an SVG interface it cannot have (a custom renderer forces
        // HTML even for SVG tag names). A tag missing from its own map stops at
        // `Element`, which is what a custom element resolves to.
        let registry_body = registry.body.as_str();
        append!(
            *ts,
            "{dom_ref_helper}{component_ref_helper}  type __VizeTemplateRefs = {{{registry_body}}};\n  type __VizeUseTemplateRef = {{ <_K extends keyof __VizeTemplateRefs>(_key: _K): Readonly<import('vue').ShallowRef<__VizeTemplateRefs[_K] | null>>; <_T = unknown>(_key: string): Readonly<import('vue').ShallowRef<_T | null>>; }};\n"
        );
    }
    let boolean_keys =
        generic_param.and_then(|_| script_content.and_then(collect_define_props_boolean_keys));
    if let Some(keys) = boolean_keys.as_ref() {
        emit_define_props_boolean_keys_type(ts, keys);
    }
    declarations::emit(
        ts,
        component_context.helpers,
        hoist_shared_preamble,
        boolean_keys.is_some(),
        template_refs.is_some(),
    );
    // Whether the template's ref registry is declared in this scope.
    template_refs.is_some()
}

fn emit_define_props_boolean_keys_type(ts: &mut String, collection: &DefinePropsBooleanKeys) {
    if collection.keys.is_empty() && !collection.has_unresolved_references {
        ts.push_str("  type __VizeDefinePropsBooleanKeys<_T> = never;\n");
        return;
    }

    ts.push_str("  type __VizeDefinePropsBooleanKeys<_T> =\n");
    if collection.has_unresolved_references {
        ts.push_str("    __VizeBooleanKey<_T>\n");
    }
    for (index, key) in collection.keys.iter().enumerate() {
        let separator = if index == 0 && !collection.has_unresolved_references {
            "    "
        } else {
            "  | "
        };
        let mut key_literal = String::default();
        push_ts_string_literal(&mut key_literal, key.as_str());
        append!(
            *ts,
            "{separator}(_T extends {{ {key_literal}?: boolean | undefined }} ? {key_literal} : never)\n"
        );
    }
    ts.push_str("  ;\n");
}

fn push_ts_string_literal(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
}
