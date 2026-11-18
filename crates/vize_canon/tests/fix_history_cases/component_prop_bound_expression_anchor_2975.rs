use crate::support::fix_history_runner::check_pack;

#[test]
fn component_prop_mismatch_is_anchored_at_the_bound_expression() {
    check_pack(
        "component-prop-bound-expression-anchor-2975",
        "fix_history_cases::component_prop_bound_expression_anchor_2975::component_prop_mismatch_is_anchored_at_the_bound_expression",
        "fa98cd8e728c94c923429d56e3b8326bd5f22eac",
        Some(2975),
        &[
            "bound-expression-matches-prop",
            "bound-expression-mismatch-anchor",
        ],
    );
}
