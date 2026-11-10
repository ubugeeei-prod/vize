use vize_carton::{String, append, cstr};
use vize_croquis::Croquis;

use super::append_model_props_type_literal;
use crate::virtual_ts::generic_parameters::GenericParameters;

/// Props type emission settings for `defineProps<T>()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PropsTypeEmission {
    deferred_to_setup: bool,
    prefer_identifier_props: bool,
}

impl PropsTypeEmission {
    pub(crate) const fn module(prefer_identifier_props: bool) -> Self {
        Self {
            deferred_to_setup: false,
            prefer_identifier_props,
        }
    }

    pub(crate) const fn deferred_to_setup(prefer_identifier_props: bool) -> Self {
        Self {
            deferred_to_setup: true,
            prefer_identifier_props,
        }
    }

    pub(crate) const fn is_deferred_to_setup(self) -> bool {
        self.deferred_to_setup
    }

    pub(crate) const fn prefer_identifier_props(self) -> bool {
        self.prefer_identifier_props
    }
}

/// Emit the setup-local props type artifact used when the `defineProps<T>()`
/// type argument can only resolve inside `__setup()`.
pub(crate) fn generate_setup_scoped_props_artifact(
    ts: &mut crate::virtual_ts::TextBuf,
    summary: &Croquis,
    script: Option<&str>,
    generic_parameters: Option<&GenericParameters>,
) {
    let Some(type_args) = summary
        .macros
        .define_props()
        .and_then(|m| m.type_args.as_ref())
    else {
        return;
    };
    let inner_type = type_args
        .strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(type_args.as_str());
    let models = summary.macros.models();

    ts.push_str("\n  // Setup-scoped props type artifact\n");
    if models.is_empty() {
        append!(*ts, "  type $SetupProps = {inner_type};\n");
    } else {
        append!(*ts, "  type $SetupProps = {inner_type} & ");
        append_model_props_type_literal(ts, summary, script, models, generic_parameters);
        ts.push_str(";\n");
    }
    ts.push_str("  const __vize_setup_props = undefined as unknown as $SetupProps;\n");
}

pub(super) fn props_type_ref(
    generic_param: Option<&GenericParameters>,
    props_type_ref_override: Option<&str>,
) -> String {
    props_type_ref_override
        .map(String::from)
        .unwrap_or_else(|| {
            let props = super::names::declared("Props");
            generic_param.map_or(props.clone(), |g| cstr!("{props}<{}>", g.names))
        })
}

pub(super) fn unused_generic_comment(
    generic_param: Option<&GenericParameters>,
    props_source: &str,
) -> &'static str {
    let Some(generic) = generic_param else {
        return "";
    };
    let names = &generic.names;
    let type_references = crate::virtual_ts::type_dependencies::free_names(props_source);
    let uses_every_generic = type_references.is_some_and(|type_references| {
        names
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .all(|name| {
                type_references
                    .iter()
                    .any(|type_reference| type_reference.as_str() == name)
            })
    });
    if uses_every_generic {
        ""
    } else {
        "// @ts-ignore TS6196: SFC generic may be used by emits or slots.\n"
    }
}

#[cfg(test)]
mod tests {
    fn unused_generic_comment(generic: Option<&str>, props: &str) -> &'static str {
        let plan = generic.map(crate::virtual_ts::generic_parameters::GenericParameters::parse);
        super::unused_generic_comment(plan.as_ref(), props)
    }

    #[test]
    fn suppresses_only_props_aliases_that_omit_sfc_generics() {
        assert_eq!(
            unused_generic_comment(Some("T"), "ImportedProps"),
            "// @ts-ignore TS6196: SFC generic may be used by emits or slots.\n"
        );
        assert_eq!(unused_generic_comment(Some("T"), "LocalProps<T>"), "");
        assert_eq!(
            unused_generic_comment(Some("T, U"), "LocalProps<T>"),
            "// @ts-ignore TS6196: SFC generic may be used by emits or slots.\n"
        );
        assert_eq!(unused_generic_comment(Some("T, U"), "LocalProps<T, U>"), "");
        assert_eq!(
            unused_generic_comment(Some("T"), r#"{ T: string; kind: "T" }"#),
            "// @ts-ignore TS6196: SFC generic may be used by emits or slots.\n"
        );
        assert_eq!(unused_generic_comment(Some("T"), "{ value: T }"), "");
        assert_eq!(
            unused_generic_comment(Some("T"), "SomeTTProps"),
            "// @ts-ignore TS6196: SFC generic may be used by emits or slots.\n"
        );
    }
}
