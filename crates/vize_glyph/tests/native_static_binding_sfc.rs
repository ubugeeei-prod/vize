//! Independently authored whole original SFC laws for explicit static bindings.

mod native_static_binding_sfc {
    use crate::support::native_conditional::fingerprint;
    use crate::support::native_conditional::support as prior;

    mod budgets;
    mod custody;
    mod failure_support;
    mod geometry;
    mod head_refusals;
    mod layout;
    mod policy;
    mod preservation;
    mod profiles;
    mod refusals;
    mod routing;
    mod support;
    mod transfers;

    use prior::assert_original;
    use support::{assert_bindings, assert_output, options, selected};
}

mod support;
