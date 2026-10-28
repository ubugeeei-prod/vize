use super::check_pack;

#[test]
fn content_mapper_template_callback_listener_preserves_contextual_parameters() {
    check_pack(
        "content-mapper-template-callback-listener-11023",
        "content_mapper_template_callback_listener_11023::content_mapper_template_callback_listener_preserves_contextual_parameters",
        "407e7cf15219405110efdd0c977bb30cb1db1644",
        Some(11023),
        &["reporter-valid", "authored-invalid-control"],
    );
}
