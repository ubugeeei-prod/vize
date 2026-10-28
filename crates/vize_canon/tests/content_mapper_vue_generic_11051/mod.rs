use super::check_pack;

#[test]
fn content_mapper_vue_generic_preserves_explicit_type_arguments() {
    check_pack(
        "content-mapper-vue-generic-11051",
        "content_mapper_vue_generic_preserves_explicit_type_arguments",
        "dfc8b0e13c20ea7c8d46e555f70ffd7fe3e89a27",
        Some(11051),
        &["reporter-valid", "invalid-value-control"],
    );
}
