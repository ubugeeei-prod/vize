//! Authored mappings for generated Options API template bindings.

use super::{is_safe_value_identifier, unresolved_extends_template_names};
use crate::virtual_ts::{VirtualTsOptions, VizeMapping, VizeSemanticLink, VizeSemanticLinkKind};
use vize_carton::{FxHashSet, String, append};
use vize_croquis::{BindingType, Croquis};

// Emit declarations for Options API template bindings (`data`/`computed`/
// `methods`/`inject`/`setup`/`props`, plus legacy globals) when the caller
// enables Options API / legacy checking.
pub(in crate::virtual_ts::generator) fn generate_options_api_variables(
    mut ts: &mut String,
    summary: &Croquis,
    options: &VirtualTsOptions,
    script: Option<&str>,
    mappings: &mut Vec<VizeMapping>,
    offset: &dyn Fn(usize) -> usize,
) -> Vec<VizeSemanticLink> {
    let mut links = Vec::new();
    // The Options API bridge only runs for non-`<script setup>` components.
    // `<script setup>` already exposes its bindings (refs, props, setup
    // returns) in template scope via the normal generator, and a
    // `defineProps<Props>()` whose argument is a type reference (not an inline
    // `TSTypeLiteral`) still registers destructured names as
    // `BindingType::Props` without populating `summary.macros.props()`, which
    // would otherwise let those names slip through the filter below and
    // produce spurious `__VizeOptionsBinding` declarations.
    if summary.bindings.is_script_setup {
        return Vec::new();
    }

    let macro_prop_names: FxHashSet<&str> = summary
        .macros
        .props()
        .iter()
        .map(|prop| prop.name.as_str())
        .collect();
    let configured_globals: FxHashSet<&str> = options
        .template_globals
        .iter()
        .map(|global| global.name.as_str())
        .collect();
    let mut names: Vec<(&str, bool)> = summary
        .bindings
        .bindings
        .iter()
        .filter_map(|(name, binding_type)| {
            let name = name.as_str();
            match binding_type {
                BindingType::Data => Some((name, true)),
                BindingType::Options | BindingType::VueGlobal => Some((name, false)),
                BindingType::Props if !macro_prop_names.contains(name) => Some((name, false)),
                _ => None,
            }
        })
        .filter(|(name, _)| !configured_globals.contains(name))
        .filter(|(name, _)| is_safe_value_identifier(name))
        .collect();
    names.sort_unstable_by_key(|(name, _)| *name);
    names.dedup_by(|left, right| left.0 == right.0);
    let inherited_unknown_names =
        unresolved_extends_template_names(summary, &configured_globals, script);

    if names.is_empty() && inherited_unknown_names.is_empty() {
        return Vec::new();
    }

    ts.push_str("  // Options API template bindings\n");
    ts.push_str(
        "  type __VizeOptionsInstance<T> = T extends abstract new (...args: any) => infer I ? I : any;\n",
    );
    ts.push_str(
        "  type __VizeOptionsBinding<T, K extends string> = K extends keyof __VizeOptionsInstance<T> ? __VizeOptionsInstance<T>[K] : any;\n",
    );
    for (name, mutable) in &names {
        let generated_start = ts.len()
            + if *mutable {
                "  var ".len()
            } else {
                "  const ".len()
            };
        append!(
            ts,
            "  {} {name}: __VizeOptionsBinding<typeof __default__, \"{name}\"> = undefined as any;\n",
            if *mutable { "var" } else { "const" }
        );
        let Some(&(start, end)) = summary.binding_spans.get(*name) else {
            continue;
        };
        if script.and_then(|script| script.get(start as usize..end as usize)) != Some(*name) {
            continue;
        }
        let original = offset(start as usize)..offset(end as usize);
        let generated = generated_start..generated_start + name.len();
        if let Some(source_range) =
            super::super::script_module::mapped_binding_range(mappings, &original)
        {
            links.push(VizeSemanticLink {
                source_range,
                target_range: generated.clone(),
                kind: VizeSemanticLinkKind::VueOptionsApiBinding,
            });
        }
        mappings.push(VizeMapping {
            gen_range: generated,
            src_range: original,
            sub_spans: Vec::new(),
        });
    }
    if !inherited_unknown_names.is_empty() {
        ts.push_str("  // Unresolved imported Options API extends bindings\n");
        for name in &inherited_unknown_names {
            append!(ts, "  const {name}: any = undefined as any;\n");
        }
    }
    ts.push_str("  ");
    for (name, _) in &names {
        append!(ts, "void {name};");
    }
    for name in &inherited_unknown_names {
        append!(ts, "void {name};");
    }
    ts.push('\n');
    links
}
