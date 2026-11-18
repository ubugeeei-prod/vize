use crate::support::fix_history_runner::check_pack;

#[test]
fn define_expose_bindings_are_part_of_component_instance() {
    check_pack(
        "exposed-component-instance-1144",
        "fix_history_cases::exposed_component_instance_1144::define_expose_bindings_are_part_of_component_instance",
        "c493e893338e64efdd7fe4f9f49acffa0c01b3d1",
        Some(1144),
        &[
            "exposed-members-callable",
            "non-exposed-member",
            "typed-exposed-wrong-argument",
        ],
    );
}
