use vize_carton::{String, append, cstr};
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
