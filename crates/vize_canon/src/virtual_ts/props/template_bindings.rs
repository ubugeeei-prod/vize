use vize_croquis::macros::PropsDestructuredBindings;
use vize_croquis::{BindingType, Croquis};

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
