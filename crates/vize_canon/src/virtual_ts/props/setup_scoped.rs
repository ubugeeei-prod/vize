use vize_carton::{FxHashSet, String, append, cstr};
use vize_croquis::Croquis;

use super::{append_model_props_type_literal, extract_generic_names};

/// Props type emission mode for `defineProps<T>()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PropsTypeEmission {
    /// Emit `export type Props = ...` in module scope before `__setup()`.
    Module,
    /// Keep the concrete props type inside `__setup()` and export it through
    /// `ReturnType<typeof __setup>`. This is needed when `T` references a
    /// setup-scope value via `typeof`.
    DeferredToSetup,
}

/// Emit the setup-local props type artifact used when the `defineProps<T>()`
/// type argument can only resolve inside `__setup()`.
pub(crate) fn generate_setup_scoped_props_artifact(ts: &mut String, summary: &Croquis) {
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
        append!(*ts, "  type __VizeSetupProps = {inner_type};\n");
    } else {
        append!(*ts, "  type __VizeSetupProps = {inner_type} & ");
        append_model_props_type_literal(ts, models);
        ts.push_str(";\n");
    }
    ts.push_str("  const __vize_setup_props = undefined as unknown as __VizeSetupProps;\n");
}

pub(super) fn props_type_ref(
    generic_param: Option<&str>,
    props_type_ref_override: Option<&str>,
) -> String {
    props_type_ref_override
        .map(String::from)
        .unwrap_or_else(|| {
            generic_param
                .map(|g| {
                    let names = extract_generic_names(g);
                    cstr!("Props<{names}>")
                })
                .unwrap_or_else(|| "Props".into())
        })
}

pub(super) fn unused_generic_comment(
    generic_param: Option<&str>,
    type_references: Option<&FxHashSet<String>>,
) -> &'static str {
    let Some(generic) = generic_param else {
        return "";
    };
    let names = extract_generic_names(generic);
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
    use super::unused_generic_comment;
    use vize_carton::FxHashSet;

    fn references(names: &[&str]) -> FxHashSet<vize_carton::String> {
        names.iter().map(|name| (*name).into()).collect()
    }

    #[test]
    fn suppresses_only_props_aliases_that_omit_sfc_generics() {
        assert_eq!(
            unused_generic_comment(Some("T"), Some(&references(&["ImportedProps"]))),
            "// @ts-ignore TS6196: SFC generic may be used by emits or slots.\n"
        );
        assert_eq!(
            unused_generic_comment(Some("T"), Some(&references(&["LocalProps", "T"]))),
            ""
        );
        assert_eq!(
            unused_generic_comment(Some("T, U"), Some(&references(&["LocalProps", "T"]))),
            "// @ts-ignore TS6196: SFC generic may be used by emits or slots.\n"
        );
        assert_eq!(
            unused_generic_comment(Some("T, U"), Some(&references(&["LocalProps", "T", "U"])),),
            ""
        );
        assert_eq!(
            unused_generic_comment(Some("T"), Some(&references(&[]))),
            "// @ts-ignore TS6196: SFC generic may be used by emits or slots.\n"
        );
        assert_eq!(
            unused_generic_comment(Some("T"), Some(&references(&["T"]))),
            ""
        );
        assert_eq!(
            unused_generic_comment(Some("T"), Some(&references(&["SomeTTProps"]))),
            "// @ts-ignore TS6196: SFC generic may be used by emits or slots.\n"
        );
    }
}
