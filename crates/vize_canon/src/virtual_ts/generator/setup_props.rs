use vize_carton::{String, append, cstr, profile};
use vize_croquis::Croquis;

use super::generics::{
    generic_fallback_args, is_ident_byte, references_any_identifier, skip_ascii_ws,
};
use crate::virtual_ts::props::{
    OptionsApiPropsSource, PropsTypeEmission, add_generic_defaults, append_default_props,
    extract_generic_names, generate_props_type, generate_props_variables,
    generate_setup_scoped_props_artifact,
};

fn is_identifier_start_byte(b: u8) -> bool {
    b == b'_' || b == b'$' || b.is_ascii_alphabetic()
}

fn collect_typeof_root_identifiers(source: &str) -> Vec<&str> {
    let bytes = source.as_bytes();
    let mut idents = Vec::new();
    let mut from = 0usize;

    while let Some(rel) = source[from..].find("typeof") {
        let at = from + rel;
        let before_ok = at == 0 || !is_ident_byte(bytes[at - 1]);
        let after_keyword = at + "typeof".len();
        let after_ok = after_keyword >= bytes.len() || !is_ident_byte(bytes[after_keyword]);
        if !before_ok || !after_ok {
            from = after_keyword;
            continue;
        }

        let ident_start = skip_ascii_ws(bytes, after_keyword);
        if ident_start >= bytes.len() || !is_identifier_start_byte(bytes[ident_start]) {
            from = after_keyword;
            continue;
        }

        let mut ident_end = ident_start + 1;
        while ident_end < bytes.len() && is_ident_byte(bytes[ident_end]) {
            ident_end += 1;
        }

        let ident = &source[ident_start..ident_end];
        if ident != "import" {
            idents.push(ident);
        }
        from = ident_end;
    }

    idents
}

fn binding_is_import(summary: &Croquis, name: &str) -> bool {
    summary.binding_spans.get(name).is_some_and(|(start, end)| {
        summary
            .import_statements
            .iter()
            .any(|imp| *start >= imp.start && *end <= imp.end)
    })
}

fn is_setup_value_binding(summary: &Croquis, name: &str) -> bool {
    summary.bindings.bindings.contains_key(name) && !binding_is_import(summary, name)
}

pub(super) fn define_props_type_requires_setup_scope(summary: &Croquis) -> bool {
    let Some(type_args) = summary
        .macros
        .define_props()
        .and_then(|m| m.type_args.as_ref())
    else {
        return false;
    };
    let inner_type = type_args
        .strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(type_args.as_str());

    if collect_typeof_root_identifiers(inner_type)
        .into_iter()
        .any(|name| is_setup_value_binding(summary, name))
    {
        return true;
    }

    let non_hoisted_type_names: Vec<String> = summary
        .type_exports
        .iter()
        .filter(|te| !te.hoisted)
        .map(|te| te.name.as_str().into())
        .collect();
    !non_hoisted_type_names.is_empty()
        && references_any_identifier(inner_type, &non_hoisted_type_names)
}

/// Build the setup props plan and emit the module-level props type in one step.
/// Keeps `generator.rs` from re-threading `options_api_props` through a second
/// call site (and from growing past the source-length gate).
pub(super) fn generate_setup_props(
    ts: &mut String,
    summary: &Croquis,
    generic_param: Option<&str>,
    options_api_props: Option<&OptionsApiPropsSource>,
    props_is_public_export: bool,
) -> SetupPropsPlan {
    let plan = SetupPropsPlan::new(summary, options_api_props, props_is_public_export);
    profile!("canon.virtual_ts.generate_props_type", {
        plan.generate_props_type(ts, summary, generic_param, options_api_props);
    });
    plan
}

pub(super) struct SetupPropsPlan {
    defer: bool,
    defer_options_api_props: bool,
    capture_options_api_default: bool,
    module_scope_declares_props: bool,
}

impl SetupPropsPlan {
    pub(super) fn new(
        summary: &Croquis,
        options_api_props: Option<&OptionsApiPropsSource>,
        props_is_public_export: bool,
    ) -> Self {
        // A `Props` declaration only lands at module scope when it is hoisted
        // (emitted directly at module level) or when it is a value-dependent
        // export recaptured through the setup return (`props_is_public_export`).
        // A private, non-hoisted `type Props` stays inside `__setup`, so it must
        // NOT be treated as an existing public alias — otherwise the public
        // `export type Props` consumers need is suppressed.
        let module_scope_declares_props = props_is_public_export
            || summary
                .type_exports
                .iter()
                .any(|te| te.hoisted && te.name.as_str() == "Props");
        Self {
            defer: define_props_type_requires_setup_scope(summary),
            defer_options_api_props: !module_scope_declares_props
                && options_api_props
                    .is_some_and(|source| source.deferred_object_source().is_some()),
            capture_options_api_default: options_api_props
                .is_some_and(OptionsApiPropsSource::captures_default),
            module_scope_declares_props,
        }
    }

    pub(super) fn props_type_emission(&self) -> PropsTypeEmission {
        if self.defer {
            PropsTypeEmission::DeferredToSetup
        } else {
            PropsTypeEmission::Module
        }
    }

    pub(super) fn generate_props_type(
        &self,
        ts: &mut String,
        summary: &Croquis,
        generic_param: Option<&str>,
        options_api_props: Option<&OptionsApiPropsSource>,
    ) {
        generate_props_type(
            ts,
            summary,
            generic_param,
            options_api_props,
            self.props_type_emission(),
        );
    }

    pub(super) fn generate_props_variables(
        &self,
        ts: &mut String,
        summary: &Croquis,
        generic_param: Option<&str>,
        check_props: bool,
    ) {
        generate_props_variables(
            ts,
            summary,
            generic_param,
            self.template_props_type_ref(),
            check_props,
        );
    }

    pub(super) fn template_props_type_ref(&self) -> Option<&'static str> {
        self.defer.then_some("__VizeSetupProps")
    }

    pub(super) fn component_props_type_ref(&self) -> &'static str {
        // Defer to `__VizeResolvedProps` only when a `Props` already lives at
        // module scope; otherwise the public `Props` alias emitted from the
        // setup return is the component prop source.
        if self.defer && self.module_scope_declares_props {
            "__VizeResolvedProps"
        } else {
            "Props"
        }
    }

    pub(super) fn generic_fallback_component_props_type_ref(&self, generic_decl: &str) -> String {
        let props_type_ref = self.component_props_type_ref();
        if props_type_ref != "Props" {
            return props_type_ref.into();
        }

        let args = generic_fallback_args(generic_decl);
        if args.is_empty() {
            props_type_ref.into()
        } else {
            cstr!("{props_type_ref}<{args}>")
        }
    }

    pub(super) fn emit_component_props_field(
        &self,
        mut ts: &mut String,
        has_emits_for_props: bool,
        generic_decl: Option<&str>,
    ) {
        let props_type_ref = generic_decl
            .map(|decl| self.generic_fallback_component_props_type_ref(decl))
            .unwrap_or_else(|| self.component_props_type_ref().into());
        if has_emits_for_props {
            append!(
                ts,
                "  $props: __VizeComponentProps<{props_type_ref}> & __EmitProps<Emits>;\n"
            );
        } else {
            append!(ts, "  $props: __VizeComponentProps<{props_type_ref}>;\n");
        }
    }

    pub(super) fn emit_artifact(&self, ts: &mut String, summary: &Croquis) {
        if self.defer {
            generate_setup_scoped_props_artifact(ts, summary);
        }
    }

    pub(super) fn push_return_field(&self, fields: &mut Vec<&'static str>) {
        if self.capture_options_api_default {
            fields.push("__default__");
        }
        if self.defer {
            fields.push("__vize_setup_props");
        }
        if self.defer_options_api_props {
            fields.push("__vize_options_props");
        }
    }

    pub(super) fn emit_options_api_artifact(
        &self,
        mut ts: &mut String,
        options_api_props: Option<&OptionsApiPropsSource>,
    ) {
        let Some(source) =
            options_api_props.and_then(OptionsApiPropsSource::deferred_object_source)
        else {
            return;
        };
        if self.module_scope_declares_props {
            return;
        }
        let const_assertion = if source.trim_start().starts_with('{') {
            " as const"
        } else {
            ""
        };
        append!(
            ts,
            "\n  const __vize_options_props = ({source}{const_assertion});\n"
        );
    }

    pub(super) fn emit_module_export(
        &self,
        ts: &mut String,
        options_api_props: Option<&OptionsApiPropsSource>,
    ) {
        if self.defer {
            if self.module_scope_declares_props {
                // A `Props` already lives at module scope (hoisted, or restored
                // by the setup type-export plan); emit an internal alias for the
                // component instance to avoid a duplicate public declaration.
                ts.push_str(
                    "type __VizeResolvedProps = Awaited<ReturnType<typeof __setup>>[\"__vize_setup_props\"];\n\n",
                );
            } else {
                // No `Props` exists at module scope (inline type args, or a
                // private setup-scoped `Props`), so restore the public alias.
                ts.push_str(
                    "export type Props = Awaited<ReturnType<typeof __setup>>[\"__vize_setup_props\"];\n\n",
                );
            }
        } else if !self.module_scope_declares_props {
            let Some(source) =
                options_api_props.filter(|source| source.deferred_object_source().is_some())
            else {
                return;
            };
            ts.push_str(
                "export type Props = __VizeOptionsPropShape<Awaited<ReturnType<typeof __setup>>[\"__vize_options_props\"]>",
            );
            append_default_props(ts, source);
            ts.push_str(";\n\n");
        }
    }

    pub(super) fn generic_component_params(
        &self,
        generic_param: Option<&str>,
    ) -> Option<(String, String)> {
        generic_param.filter(|_| !self.defer).map(|generic| {
            (
                add_generic_defaults(generic),
                extract_generic_names(generic),
            )
        })
    }
}
