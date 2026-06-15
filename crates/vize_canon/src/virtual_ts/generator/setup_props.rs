use vize_carton::{String, append};
use vize_croquis::Croquis;

use super::generics::{is_ident_byte, references_any_identifier, skip_ascii_ws};
use crate::virtual_ts::props::{
    OptionsApiPropsSource, PropsTypeEmission, generate_props_type, generate_props_variables,
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

pub(super) struct SetupPropsPlan {
    defer: bool,
    module_scope_declares_props: bool,
}

impl SetupPropsPlan {
    pub(super) fn new(summary: &Croquis) -> Self {
        Self {
            defer: define_props_type_requires_setup_scope(summary),
            module_scope_declares_props: summary
                .type_exports
                .iter()
                .any(|te| te.hoisted && te.name.as_str() == "Props"),
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
    ) {
        generate_props_variables(ts, summary, generic_param, self.template_props_type_ref());
    }

    pub(super) fn template_props_type_ref(&self) -> Option<&'static str> {
        self.defer.then_some("__VizeSetupProps")
    }

    pub(super) fn component_props_type_ref(&self) -> &'static str {
        if self.defer && self.module_scope_declares_props {
            "__VizeResolvedProps"
        } else {
            "Props"
        }
    }

    pub(super) fn emit_component_props_field(
        &self,
        mut ts: &mut String,
        has_emits_for_props: bool,
    ) {
        let props_type_ref = self.component_props_type_ref();
        if has_emits_for_props {
            append!(ts, "  $props: {props_type_ref} & __EmitProps<Emits>;\n");
        } else {
            append!(ts, "  $props: {props_type_ref};\n");
        }
    }

    pub(super) fn emit_artifact(&self, ts: &mut String, summary: &Croquis) {
        if self.defer {
            generate_setup_scoped_props_artifact(ts, summary);
        }
    }

    pub(super) fn push_return_field(&self, fields: &mut Vec<&'static str>) {
        if self.defer {
            fields.push("__vize_setup_props");
        }
    }

    pub(super) fn emit_module_export(&self, ts: &mut String) {
        if !self.defer {
            return;
        }
        if self.module_scope_declares_props {
            ts.push_str(
                "type __VizeResolvedProps = Awaited<ReturnType<typeof __setup>>[\"__vize_setup_props\"];\n\n",
            );
        } else {
            ts.push_str(
                "export type Props = Awaited<ReturnType<typeof __setup>>[\"__vize_setup_props\"];\n\n",
            );
        }
    }

    pub(super) fn generic_param<'a>(&self, generic_param: Option<&'a str>) -> Option<&'a str> {
        generic_param.filter(|_| !self.defer)
    }
}
