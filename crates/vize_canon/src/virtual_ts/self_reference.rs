use vize_carton::String;

use super::helpers::to_safe_identifier;

pub(crate) const SELF_REFERENCE_COMPONENT: &str = "Self";
pub(crate) const SELF_REFERENCE_COMPONENT_REF: &str = "__VizeSelf";

pub(crate) fn is_reserved_self_reference(name: &str, enabled: bool) -> bool {
    enabled && name == SELF_REFERENCE_COMPONENT
}

pub(crate) fn component_ref_name(name: &str, enabled: bool) -> String {
    if is_reserved_self_reference(name, enabled) {
        SELF_REFERENCE_COMPONENT_REF.into()
    } else {
        to_safe_identifier(name)
    }
}
