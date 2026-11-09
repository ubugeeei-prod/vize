use super::{OptionsApiPropsSource, SetupPropsPlan, append_default_props};
use crate::virtual_ts::generated_names as names;
use vize_carton::{append, cstr};

impl SetupPropsPlan {
    pub(in crate::virtual_ts::generator) fn emit_module_export(
        &self,
        ts: &mut crate::virtual_ts::TextBuf,
        options_api_props: Option<&OptionsApiPropsSource>,
        generic_param: Option<&crate::virtual_ts::generic_parameters::GenericParameters>,
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
            let head = if self.module_scope_declares_props {
                names::local_type("$ResolvedProps")
            } else {
                names::export_type("Props")
            };
            let setup = names::setup_return(&arguments);
            append!(*ts, "{head}{declaration} = {setup}[\"__vize_setup_props\"]");
            forwarded_roots.push_props_tail(ts, &setup);
            ts.push_str(";\n");
            if !self.module_scope_declares_props {
                names::reexport(ts, "Props");
            }
            ts.push('\n');
        } else if self.defer_options_api_props {
            let Some(source) =
                options_api_props.filter(|source| source.deferred_object_source().is_some())
            else {
                return;
            };
            let head = if self.module_scope_declares_props {
                names::local_type("$ResolvedProps")
            } else {
                names::export_type("Props")
            };
            append!(
                *ts,
                "{head} = $OptionsPropShape<Awaited<ReturnType<typeof __setup>>[\"__vize_options_props\"]>"
            );
            append_default_props(ts, source);
            ts.push_str(";\n");
            if !self.module_scope_declares_props {
                names::reexport(ts, "Props");
            }
            ts.push('\n');
        }
    }
}
