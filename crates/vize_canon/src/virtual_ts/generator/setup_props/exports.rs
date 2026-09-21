use super::{OptionsApiPropsSource, SetupPropsPlan, append_default_props};
use vize_carton::{String, append, cstr};

impl SetupPropsPlan {
    pub(in crate::virtual_ts::generator) fn emit_module_export(
        &self,
        ts: &mut String,
        options_api_props: Option<&OptionsApiPropsSource>,
        generic_param: Option<&str>,
        forwarded_roots: &super::super::fallthrough::ForwardedRoots,
    ) {
        if self.has_inferred_model_defaults {
            crate::virtual_ts::model_types::emit_defaults_type(ts, generic_param);
        }
        if self.defer {
            let (declaration, arguments) = super::super::generics::generic_injection(generic_param)
                .map(|(declaration, names)| {
                    (cstr!("<{declaration}>"), cstr!("<{}>", names.join(", ")))
                })
                .unwrap_or_default();
            let (export, name) = if self.module_scope_declares_props {
                ("", "__VizeResolvedProps")
            } else {
                ("export ", "Props")
            };
            let setup = cstr!("Awaited<ReturnType<typeof __setup{arguments}>>");
            append!(
                *ts,
                "{export}type {name}{declaration} = {setup}[\"__vize_setup_props\"]"
            );
            forwarded_roots.push_props_tail(ts, setup.as_str());
            ts.push_str(";\n\n");
        } else if self.defer_options_api_props {
            let Some(source) =
                options_api_props.filter(|source| source.deferred_object_source().is_some())
            else {
                return;
            };
            if self.module_scope_declares_props {
                ts.push_str(
                    "type __VizeResolvedProps = __VizeOptionsPropShape<Awaited<ReturnType<typeof __setup>>[\"__vize_options_props\"]>",
                );
            } else {
                ts.push_str(
                    "export type Props = __VizeOptionsPropShape<Awaited<ReturnType<typeof __setup>>[\"__vize_options_props\"]>",
                );
            }
            append_default_props(ts, source);
            ts.push_str(";\n\n");
        }
    }
}
