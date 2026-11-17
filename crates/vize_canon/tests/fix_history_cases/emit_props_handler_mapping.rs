use crate::support::fix_history_runner::check_pack;

#[test]
fn declared_emits_map_to_checked_handler_props() {
    check_pack(
        "emit-props-handler-mapping",
        "fix_history_cases::emit_props_handler_mapping::declared_emits_map_to_checked_handler_props",
        "d9535048975662e1fd68bf05db50b3ffd64a19f4",
        None,
        &[
            "emit-only-child",
            "emit-listener-matching-payload",
            "emit-listener-wrong-payload",
        ],
    );
}
