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
            false,
        ));
    };

    if needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        cstr!("}} & __VizeVue2ComponentInstance & __VizeShallowUnwrapRef<Exposed<{args}>>;\n")
    } else {
        cstr!("}} & __VizeComponentPublicBase & __VizeShallowUnwrapRef<Exposed<{args}>>;\n")
    }
}

pub(in crate::virtual_ts::generator) fn instance_suffix(
    legacy_vue2: bool,
    dialect: VueVersion,
    has_exposed_type: bool,
    has_root_el: bool,
) -> &'static str {
    if has_root_el && !needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        return if has_exposed_type {
            "} & Omit<__VizeComponentPublicBase, '$el'> & { $el: Awaited<ReturnType<typeof __setup>>['__vize_root_el'] } & __VizeShallowUnwrapRef<Exposed>;\n"
        } else {
            "} & Omit<__VizeComponentPublicBase, '$el'> & { $el: Awaited<ReturnType<typeof __setup>>['__vize_root_el'] };\n"
        };
    }
    match (
        needs_legacy_vue2_helpers(legacy_vue2, dialect),
        has_exposed_type,
    ) {
        (true, true) => "} & __VizeVue2ComponentInstance & __VizeShallowUnwrapRef<Exposed>;\n",
        (true, false) => "} & __VizeVue2ComponentInstance;\n",
        (false, true) => "} & __VizeComponentPublicBase & __VizeShallowUnwrapRef<Exposed>;\n",
        (false, false) => "} & __VizeComponentPublicBase;\n",
    }
}
