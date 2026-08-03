use super::super::helpers::is_reserved_identifier;
use super::template_bindings::should_skip_template_prop_binding;
use vize_carton::{FxHashSet, String};
use vize_croquis::builtins::{is_event_local, is_js_global, is_render_local, is_vue_builtin};
use vize_croquis::drawer::{extract_identifiers_oxc, is_keyword};
use vize_croquis::{Croquis, ScopeData};

fn can_emit_keyed_template_prop_binding(prop_name: &str) -> bool {
    let mut chars = prop_name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_' || first == '$')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
        && !prop_name.starts_with('$')
        && !is_reserved_identifier(prop_name)
}

pub(super) fn collect_keyed_template_prop_names(
    summary: &Croquis,
    emitted_names: &FxHashSet<String>,
) -> Vec<String> {
    let mut names = FxHashSet::default();
    for undef in &summary.undefined_refs {
        let name = undef.name.as_str();
        if emitted_names.contains(name)
            || should_skip_template_prop_binding(summary, name)
            || !can_emit_keyed_template_prop_binding(name)
        {
            continue;
        }
        names.insert(name.into());
    }

    // A v-for source is emitted as the loop initializer rather than as a
    // standalone template expression. Include its root references in the
    // imported/opaque defineProps fallback so template-only props such as
    // `messages` still receive a local binding. Scope locals and language/
    // template globals must remain untouched (for example a nested `group`
    // alias or `Math` in the source expression).
    for scope in summary.scopes.iter() {
        let ScopeData::VFor(data) = scope.data() else {
            continue;
        };
        for ident in extract_identifiers_oxc(data.source.as_str()) {
            let name = ident.as_str();
            if emitted_names.contains(name)
                || should_skip_template_prop_binding(summary, name)
                || !can_emit_keyed_template_prop_binding(name)
                || summary
                    .scopes
                    .iter()
                    .any(|candidate| candidate.has_binding(name))
                || is_js_global(name)
                || is_render_local(name)
                || is_event_local(name)
                || is_vue_builtin(name)
                || is_keyword(name)
            {
                continue;
            }
            names.insert(name.into());
        }
    }
    let mut names: Vec<String> = names.into_iter().collect();
    names.sort_unstable();
    names
}
