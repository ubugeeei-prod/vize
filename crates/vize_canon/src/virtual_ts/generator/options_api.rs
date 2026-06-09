//! Options API template-binding emission for the virtual TypeScript generator.

use vize_croquis::{BindingType, Croquis};

use crate::virtual_ts::types::VirtualTsOptions;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;

// Emit `const <name>: any` declarations for Options API template bindings
// (`data`/`computed`/`methods`/`inject`/`setup`/`props`, plus any Nuxt 2 globals
// the legacy path collected). Options API is officially supported in Vue 3, so
// this is part of the standard build and driven by a runtime opt-in — it costs
// nothing unless the caller enables Options API / legacy checking.
pub(super) fn generate_options_api_variables(
    mut ts: &mut String,
    summary: &Croquis,
    options: &VirtualTsOptions,
) {
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
    let mut names: Vec<&str> = summary
        .bindings
        .bindings
        .iter()
        .filter_map(|(name, binding_type)| {
            let name = name.as_str();
            match binding_type {
                BindingType::Data | BindingType::Options | BindingType::VueGlobal => Some(name),
                BindingType::Props if !macro_prop_names.contains(name) => Some(name),
                _ => None,
            }
        })
        .filter(|name| !configured_globals.contains(name))
        .filter(|name| is_safe_value_identifier(name))
        .collect();
    names.sort_unstable();
    names.dedup();

    if names.is_empty() {
        return;
    }

    ts.push_str("  // Options API template bindings\n");
    for name in &names {
        append!(ts, "  const {name}: any = undefined as any;\n");
    }
    ts.push_str("  ");
    for name in &names {
        append!(ts, "void {name};");
    }
    ts.push('\n');
}

fn is_safe_value_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_' || first == '$') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}
