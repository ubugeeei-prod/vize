use crate::support::fix_history_runner::check_pack;

pub(crate) fn contextual_prop_anchors() {
    check_pack(
        "contextual-prop-anchors-10661",
        "contextual_prop_anchors_preserve_authored_diagnostics",
        "da6d833de40c8a084022d0229beaeec25a92eb41",
        Some(10661),
        &[
            "original-from",
            "all-keyword-props",
            "authored-contextual-locals",
            "authored-prop-shadow-imports",
            "reactive-prop-destructure",
            "const-enum-import",
            "unicode-identities",
            "authored-ts2552-retained",
            "non-colliding-keyword-props",
            "authored-undefined-shadow",
        ],
    );
}
