use vize_croquis::croquis::{ComponentUsage, PassedProp};

use super::inline_callback_classifier::is_direct_inline_function_prop_value;
use crate::virtual_ts::helpers::shared_types::{COMPONENT_REF_CALLBACK, TEMPLATE_COMPONENT_REF};
use vize_carton::{String, cstr};

pub(crate) fn is_inline_ref_callback_prop(prop: &PassedProp) -> bool {
    !prop.name_is_dynamic
        && prop.name.as_str() == "ref"
        && prop.is_dynamic
        && prop
            .value
            .as_ref()
            .is_some_and(|value| is_direct_inline_function_prop_value(value.as_str()))
}

pub(super) fn append_ref_callback_helper(
    ts: &mut crate::virtual_ts::TextBuf,
    usages: &[(usize, &ComponentUsage)],
) {
    if usages
        .iter()
        .any(|(_, usage)| usage.props.iter().any(is_inline_ref_callback_prop))
    {
        ts.push_str(
            "  type $ComponentRefCallback<_C> = (ref: (_C extends abstract new (...args: any[]) => infer _I ? _I : _C extends (props: any, ctx: any, expose: (exposed: infer _E) => any, ...args: any[]) => any ? NonNullable<_E> : any) | null, refs: Record<string, any>) => void;\n",
        );
    }
}

/// The type of a function `ref` on component usage `idx`. Vue passes it the
/// instance a template ref of the same component holds, the one
/// `$TemplateComponentRef` gives `useTemplateRef`, and `null` on unmount.
/// Embedded documents declare the generic `$ComponentRefCallback<_C>` above.
pub(super) fn component_ref_callback_type(
    component_type_name: &str,
    idx: usize,
    hoisted: bool,
) -> String {
    if !hoisted {
        let callback = COMPONENT_REF_CALLBACK.select(false);
        return cstr!("{callback}<__{component_type_name}_Component_{idx}>");
    }
    let instance = TEMPLATE_COMPONENT_REF.select(true);
    let record = crate::virtual_ts::generated_names::lib("Record");
    cstr!(
        "(ref: {instance}<__{component_type_name}_Component_{idx}> | null, refs: {record}<string, any>) => void"
    )
}
