use crate::support::fix_history_runner::check_pack;

#[test]
fn component_prop_type_errors_anchor_at_attribute_name() {
    check_pack(
        "prop-type-error-attribute-name-anchor-3205",
        "fix_history_cases::prop_type_error_attribute_name_anchor_3205::component_prop_type_errors_anchor_at_attribute_name",
        "55227fa84922109af547beb1f6dc7a00d7c2dbba",
        Some(3205),
        &[
            "cross-file-literal-mismatch-anchors-at-name",
            "check-server-multibyte-matching-prop",
            "check-server-multibyte-mismatch-anchors-at-name",
            "prefixed-and-modified-names-match",
            "prefixed-and-modified-names-mismatch-anchor-at-names",
            "utf8-static-label-matching-prop",
            "utf8-static-label-mismatch-anchors-at-name",
        ],
    );
}
