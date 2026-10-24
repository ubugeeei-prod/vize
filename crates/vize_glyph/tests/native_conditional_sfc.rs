//! Whole original SFC laws for explicitly selected native conditional values.

mod native_conditional_sfc {
    mod custody;
    mod layout;
    mod policy;
    mod preservation;
    mod refusals;
    mod routing;
    use crate::support::native_conditional::support;

    use support::{assert_original, assert_output, options};
}

mod support;
