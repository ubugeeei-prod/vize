//! Public instance helpers for modern and legacy Vue dialects.

use super::{LEGACY_COMPONENT_INSTANCE_HELPER, needs_legacy_vue2_helpers};
use crate::virtual_ts::generated_names as names;
use vize_carton::{append, config::VueVersion};

pub(in crate::virtual_ts::generator) fn instance_helper(
    legacy_vue2: bool,
    dialect: VueVersion,
) -> &'static str {
    if needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        LEGACY_COMPONENT_INSTANCE_HELPER
    } else {
        ""
    }
}

/// [`instance_suffix`] for the generic component constructor, where the SFC's
/// type parameters are in scope and a generic `Exposed` alias must therefore be
/// instantiated rather than left to its declared defaults (#3354).
///
/// `exposed_generic_args` is `None` when the alias takes no parameters, which
/// keeps the output identical to [`instance_suffix`].
pub(in crate::virtual_ts::generator) fn generic_instance_suffix(
    ts: &mut crate::virtual_ts::TextBuf,
    (legacy_vue2, dialect): (bool, VueVersion),
    has_exposed_type: bool,
    exposed_generic_args: Option<&str>,
) {
    let Some(args) = exposed_generic_args.filter(|_| has_exposed_type) else {
        instance_suffix(ts, legacy_vue2, dialect, has_exposed_type, (false, false));
        return;
    };
    let base = if needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        names::helper("$Vue2ComponentInstance", &[])
    } else {
        names::helper("$ComponentPublicBase", &[])
    };
    append!(
        *ts,
        "}} & {base} & {}<{}<{args}>>;\n",
        names::helper("$ShallowUnwrapRef", &[""]),
        names::declared("Exposed")
    );
}

/// `(has_root_el, has_refs)`: the instance members the setup scope infers
/// (`inferComponentDollarEl`, `inferComponentDollarRefs`) replace the ones
/// `ComponentPublicInstance` declares.
fn instance_suffix(
    ts: &mut crate::virtual_ts::TextBuf,
    legacy_vue2: bool,
    dialect: VueVersion,
    has_exposed_type: bool,
    (has_root_el, has_refs): (bool, bool),
) {
    ts.push_str("} & ");
    instance_base(
        ts,
        legacy_vue2,
        dialect,
        has_exposed_type,
        (has_root_el, has_refs),
    );
    ts.push_str(";\n");
}

pub(in crate::virtual_ts::generator) fn instance_base(
    ts: &mut crate::virtual_ts::TextBuf,
    legacy_vue2: bool,
    dialect: VueVersion,
    has_exposed_type: bool,
    (has_root_el, has_refs): (bool, bool),
) {
    if (has_root_el || has_refs) && !needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        let setup = "Awaited<ReturnType<typeof __setup>>";
        ts.push_str("$MappedOmit<$ComponentPublicBase, ");
        match (has_root_el, has_refs) {
            (true, false) => append!(*ts, "'$el'> & {{ $el: {setup}['__vize_root_el'] }}"),
            (false, _) => append!(*ts, "'$refs'> & {{ $refs: {setup}['__vize_refs'] }}"),
            (true, true) => append!(
                *ts,
                "'$el' | '$refs'> & {{ $el: {setup}['__vize_root_el']; $refs: {setup}['__vize_refs'] }}"
            ),
        }
    } else if needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        ts.push_str("$Vue2ComponentInstance");
    } else {
        ts.push_str("$ComponentPublicBase");
    }
    if has_exposed_type {
        append!(*ts, " & $ShallowUnwrapRef<{}>", names::declared("Exposed"));
    }
}
