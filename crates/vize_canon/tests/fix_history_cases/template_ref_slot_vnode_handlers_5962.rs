use crate::support::fix_history_runner::check_pack;

#[test]
fn slot_listeners_vnode_hooks_and_ref_callbacks_keep_their_contexts() {
    check_pack(
        "template-ref-slot-vnode-handlers-5962",
        "fix_history_cases::template_ref_slot_vnode_handlers_5962::slot_listeners_vnode_hooks_and_ref_callbacks_keep_their_contexts",
        "788000753c5c6bab73faedebc979a18d6db735e2",
        Some(5962),
        &[
            "slot-outlet-listeners-use-declared-slot-props",
            "slot-outlet-listener-payload-mismatch",
            "component-ref-callback-contextual",
            "component-ref-callback-keeps-ts-ignore",
            "component-ref-callback-without-ts-ignore",
            "unresolved-global-component-ref-callback",
        ],
    );
}
