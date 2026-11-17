use crate::support::fix_history_runner::check_pack;

#[test]
fn hyphenated_and_colon_emit_listeners_resolve_camelized_handlers() {
    check_pack(
        "hyphenated-emit-handlers-1998",
        "fix_history_cases::hyphenated_emit_handlers_1998::hyphenated_and_colon_emit_listeners_resolve_camelized_handlers",
        "f47f024b760f7692352612dc8a0cf850c18ae373",
        Some(1998),
        &[
            "hyphenated-emit-listeners",
            "colon-hyphen-listener-wrong-payload",
            "hyphen-listener-wrong-payload",
        ],
    );
}
