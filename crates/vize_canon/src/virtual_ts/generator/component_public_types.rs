//! Public slot and expose aliases, captured from the setup and template scopes.

use super::{emits::inner_type_of, generics::module_alias_generic_suffix};
use vize_carton::{String, append, cstr};
use vize_croquis::Croquis;

/// Emit the private slot contract and preserve the public `Slots` export. When the slots type from
/// `defineSlots` references an SFC generic parameter, the alias re-declares
/// the parameters (with safe defaults) so declaration emit resolves them
/// (#3065).
/// Returns whether the alias re-declared the SFC's type parameters, so the
/// generic component constructor can instantiate it instead of falling back to
/// the declared defaults (#3354).
pub(super) fn emit_slots_type(
    ts: &mut String,
    summary: &Croquis,
    generic_injection: Option<&(String, Vec<String>)>,
    export_slots: bool,
    inferred_slots: bool,
) -> bool {
    let slots_type_args = summary
        .macros
        .define_slots()
        .and_then(|m| m.type_args.as_ref());
    let is_generic = if let Some(type_args) = slots_type_args {
        let inner_type = inner_type_of(type_args);
        let suffix = module_alias_generic_suffix(generic_injection, inner_type);
        append!(*ts, "type __VizeSlots{suffix} = {inner_type};\n");
        !suffix.is_empty()
    } else if inferred_slots {
        let (declaration, arguments) = generic_injection
            .map(|(declaration, names)| (cstr!("<{declaration}>"), cstr!("<{}>", names.join(", "))))
            .unwrap_or_default();
        append!(
            *ts,
            "type __VizeSlots{declaration} = Awaited<ReturnType<typeof __setup{arguments}>>['__vize_template_slots'];\n"
        );
        generic_injection.is_some()
    } else {
        ts.push_str("type __VizeSlots = {};\n");
        false
    };
    if export_slots {
        ts.push_str("export type { __VizeSlots as Slots };\n");
    }
    is_generic
}

/// Emit the module-scope `export type Exposed` alias (for `InstanceType` and
/// `useTemplateRef`); returns whether the component exposes anything. A typed
/// `defineExpose` referencing an SFC generic parameter re-declares the
/// parameters just like `Slots` (#3065).
///
/// The second flag reports whether the alias re-declared those parameters, so
/// the generic component constructor can instantiate it (#3354).
pub(super) fn emit_exposed_type(
    ts: &mut String,
    summary: &Croquis,
    generic_injection: Option<&(String, Vec<String>)>,
) -> (bool, bool) {
    let Some(expose) = summary.macros.define_expose() else {
        return (false, false);
    };
    if let Some(ref type_args) = expose.type_args {
        let inner_type = inner_type_of(type_args);
        let suffix = module_alias_generic_suffix(generic_injection, inner_type);
        append!(*ts, "export type Exposed{suffix} = {inner_type};\n");
        (true, !suffix.is_empty())
    } else if expose.runtime_args.is_some() {
        // Runtime args are returned from __setup() to keep them in scope.
        // Use Awaited<ReturnType<...>> to handle both sync and async setup.
        if let Some((declaration, names)) = generic_injection {
            let names = names.join(", ");
            append!(
                *ts,
                "export type Exposed<{declaration}> = Awaited<ReturnType<typeof __setup<{names}>>>[\"__vize_exposed\"];\n"
            );
            return (true, true);
        }
        ts.push_str(
            "export type Exposed = Awaited<ReturnType<typeof __setup>>[\"__vize_exposed\"];\n",
        );
        (true, false)
    } else {
        (false, false)
    }
}

pub(super) fn infers_slots(
    summary: &Croquis,
    root: Option<&vize_relief::RootNode<'_>>,
    options: crate::virtual_ts::types::VirtualTsCheckOptions,
) -> bool {
    options.check_props
        && options.check_template_bindings
        && crate::virtual_ts::scope::has_inferred_slots(summary, root)
}

pub(super) fn push_template_return(
    fields: &mut Vec<String>,
    inferred_slots: bool,
    has_root_el: bool,
) {
    if has_root_el {
        fields.push("__vize_root_el".into());
    }
    if inferred_slots {
        fields.push("__vize_template_slots: __vize_template".into());
    }
}
