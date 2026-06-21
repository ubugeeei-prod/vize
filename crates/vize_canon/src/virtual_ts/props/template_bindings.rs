use vize_carton::{FxHashSet, String};
use vize_croquis::macros::PropDefinition;
use vize_croquis::macros::PropsDestructuredBindings;
use vize_croquis::{BindingType, Croquis};

use super::emit_template_prop_binding;

#[inline]
pub(super) fn should_skip_template_prop_binding(summary: &Croquis, prop_name: &str) -> bool {
    is_define_props_destructure_local(summary.macros.props_destructure(), prop_name)
        || summary
            .bindings
            .get(prop_name)
            .is_some_and(|binding_type| !matches!(binding_type, BindingType::Props))
}

fn is_define_props_destructure_local(
    destructure: Option<&PropsDestructuredBindings>,
    prop_name: &str,
) -> bool {
    destructure.is_some_and(|destructure| {
        destructure.get(prop_name).is_some()
            || destructure.rest_id.as_deref() == Some(prop_name)
            || destructure
                .bindings
                .values()
                .any(|binding| binding.local.as_str() == prop_name)
    })
}

pub(super) fn emit_macro_template_prop_bindings(
    ts: &mut String,
    summary: &Croquis,
    props_type_ref: &str,
    props: &[PropDefinition],
    defaulted_prop_names: &FxHashSet<String>,
    emitted_names: &mut FxHashSet<String>,
) {
    for prop in props {
        if should_skip_template_prop_binding(summary, prop.name.as_str()) {
            continue;
        }
        emit_template_prop_binding(
            ts,
            props_type_ref,
            prop.name.as_str(),
            prop.default_value.is_some() || defaulted_prop_names.contains(&prop.name),
        );
        emitted_names.insert(prop.name.as_str().into());
    }
}
