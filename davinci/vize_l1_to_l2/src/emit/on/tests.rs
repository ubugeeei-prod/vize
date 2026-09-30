use super::classify_modifiers;

#[test]
fn common_two_modifier_buckets_stay_inline() {
    let classified = classify_modifiers(
        "keyup",
        ["capture", "once", "stop", "prevent", "enter", "escape"],
    );

    assert!(!classified.options.spilled());
    assert!(!classified.event.spilled());
    assert!(!classified.keys.spilled());
}

#[test]
fn authored_modifiers_spill_without_a_length_ceiling() {
    let classified = classify_modifiers(
        "keyup",
        [
            "capture", "once", "passive", "stop", "prevent", "self", "enter", "escape", "space",
        ],
    );

    assert!(classified.options.spilled());
    assert!(classified.event.spilled());
    assert!(classified.keys.spilled());
    assert_eq!(
        classified.options.as_slice(),
        ["capture", "once", "passive"]
    );
    assert_eq!(classified.event.as_slice(), ["stop", "prevent", "self"]);
    assert_eq!(classified.keys.as_slice(), ["enter", "escape", "space"]);
}

#[cfg(target_pointer_width = "64")]
#[test]
fn inline_storage_stack_tradeoff_is_pinned() {
    assert_eq!(core::mem::size_of::<super::Classified<'_>>(), 120);
    assert_eq!(core::mem::size_of::<super::OptionModifiers<'_>>() * 3, 120);
}
