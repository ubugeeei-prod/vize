//! Public instance helpers for modern and legacy Vue dialects.

use super::{LEGACY_COMPONENT_INSTANCE_HELPER, needs_legacy_vue2_helpers};
use vize_carton::{config::VueVersion, cstr};

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
    legacy_vue2: bool,
    dialect: VueVersion,
    has_exposed_type: bool,
    exposed_generic_args: Option<&str>,
) -> vize_carton::String {
    let Some(args) = exposed_generic_args.filter(|_| has_exposed_type) else {
        return vize_carton::String::from(instance_suffix(
            legacy_vue2,
            dialect,
            has_exposed_type,
            (false, false),
        ));
    };

    if needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        cstr!("}} & $$HogeVue2ComponentInstance & $$HogeShallowUnwrapRef<Exposed<{args}>>;\n")
    } else {
        cstr!("}} & $$HogeComponentPublicBase & $$HogeShallowUnwrapRef<Exposed<{args}>>;\n")
    }
}

/// `(has_root_el, has_refs)`: the instance members the setup scope infers
/// (`inferComponentDollarEl`, `inferComponentDollarRefs`) replace the ones
/// `ComponentPublicInstance` declares.
pub(in crate::virtual_ts::generator) fn instance_suffix(
    legacy_vue2: bool,
    dialect: VueVersion,
    has_exposed_type: bool,
    (has_root_el, has_refs): (bool, bool),
) -> &'static str {
    if (has_root_el || has_refs) && !needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        return match (has_root_el, has_refs, has_exposed_type) {
            (true, false, true) => {
                "} & Omit<$$HogeComponentPublicBase, '$el'> & { $el: Awaited<ReturnType<typeof __setup>>['__vize_root_el'] } & $$HogeShallowUnwrapRef<Exposed>;\n"
            }
            (true, false, false) => {
                "} & Omit<$$HogeComponentPublicBase, '$el'> & { $el: Awaited<ReturnType<typeof __setup>>['__vize_root_el'] };\n"
            }
            (false, _, true) => {
                "} & Omit<$$HogeComponentPublicBase, '$refs'> & { $refs: Awaited<ReturnType<typeof __setup>>['__vize_refs'] } & $$HogeShallowUnwrapRef<Exposed>;\n"
            }
            (false, _, false) => {
                "} & Omit<$$HogeComponentPublicBase, '$refs'> & { $refs: Awaited<ReturnType<typeof __setup>>['__vize_refs'] };\n"
            }
            (true, true, true) => {
                "} & Omit<$$HogeComponentPublicBase, '$el' | '$refs'> & { $el: Awaited<ReturnType<typeof __setup>>['__vize_root_el']; $refs: Awaited<ReturnType<typeof __setup>>['__vize_refs'] } & $$HogeShallowUnwrapRef<Exposed>;\n"
            }
            (true, true, false) => {
                "} & Omit<$$HogeComponentPublicBase, '$el' | '$refs'> & { $el: Awaited<ReturnType<typeof __setup>>['__vize_root_el']; $refs: Awaited<ReturnType<typeof __setup>>['__vize_refs'] };\n"
            }
        };
    }
    match (
        needs_legacy_vue2_helpers(legacy_vue2, dialect),
        has_exposed_type,
    ) {
        (true, true) => "} & $$HogeVue2ComponentInstance & $$HogeShallowUnwrapRef<Exposed>;\n",
        (true, false) => "} & $$HogeVue2ComponentInstance;\n",
        (false, true) => "} & $$HogeComponentPublicBase & $$HogeShallowUnwrapRef<Exposed>;\n",
        (false, false) => "} & $$HogeComponentPublicBase;\n",
    }
}
