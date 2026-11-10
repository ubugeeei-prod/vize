//! An unbound slot (`<template #name>`) reads no payload, so it infers no
//! host. The root usage's constructor-instance pair stays: its props type
//! reads the actual constructor result from it.

use super::generate;

fn count(code: &str, needle: &str) -> usize {
    code.matches(needle).count()
}

const UNREAD_CONTENT_SLOT: &str = "void function _slot_content_1(__vize_slot_props_1: unknown) {";

fn assert_no_slot_helpers(code: &str) {
    for helper in [
        "__vize_slot_host_",
        "$SlotHostSelector<",
        "$SlotsFactory<",
        "$SlotPayload<",
    ] {
        assert_eq!(count(code, helper), 0, "unexpected {helper}:\n{code}");
    }
}

#[test]
fn unbound_slot_without_props_infers_no_host() {
    let code = generate(r#"<el-badge><template #content>x</template></el-badge>"#);
    assert_eq!(count(&code, "__vize_slot_host_"), 0, "{code}");
    assert_eq!(count(&code, "$SlotHostSelector<"), 0, "{code}");
    assert_eq!(count(&code, "$SlotsFactory<typeof el_badge>"), 0, "{code}");
    // Nothing reads the slot helpers, so none is declared.
    assert_eq!(count(&code, "type $SlotsFactory<C>"), 0, "{code}");
    assert_eq!(count(&code, "type $SlotPayload<"), 0, "{code}");
    assert_eq!(count(&code, UNREAD_CONTENT_SLOT), 1, "{code}");
}

#[test]
fn guarded_unbound_slot_with_checked_props_emits_no_slot_helpers() {
    let code = generate(
        r#"<div><el-badge v-if="enabled" :value="1"><template #content>x</template></el-badge></div>"#,
    );

    assert_eq!(
        count(&code, "const __vize_prop_check_0_value:"),
        1,
        "{code}"
    );
    assert_no_slot_helpers(&code);
}

#[test]
fn repeated_unbound_slot_with_checked_props_emits_no_slot_helpers() {
    let code = generate(
        r#"<div><el-badge v-for="item in items" :value="item"><template #content>x</template></el-badge></div>"#,
    );

    assert_eq!(
        count(&code, "const __vize_prop_check_0_value:"),
        1,
        "{code}"
    );
    assert_no_slot_helpers(&code);
}

#[test]
fn unbound_root_slot_keeps_only_the_constructor_pair_its_props_type_reads() {
    let code = generate(r#"<el-badge :value="1"><template #content>x</template></el-badge>"#);
    // The pair is written once, with exactly the constructions the full host
    // binding writes for it ...
    assert_eq!(
        count(
            &code,
            "  const __vize_slot_host_1_instances = (undefined as unknown as <__Z, __P>(withoutArguments: __Z, withArguments: __P) => { readonly withoutArguments: __Z; readonly withArguments: __P })(new (undefined as unknown as $SlotConstructorWithoutArguments<$SlotConstructor<typeof el_badge>>)(), new (undefined as unknown as $SlotConstructorWithArguments<$SlotConstructor<typeof el_badge>>)({\n    \"value\": 1,\n  }, ...([] as unknown as $SlotConstructorArgs<$SlotConstructor<typeof el_badge>>)));\n",
        ),
        1,
        "{code}"
    );
    // ... and the usage's props type is its only reader.
    let props_alias = code
        .lines()
        .find(|line| line.trim_start().starts_with("type __VProps0 = "))
        .unwrap_or_else(|| panic!("props alias:\n{code}"));
    let alias_reads = count(props_alias, "typeof __vize_slot_host_1_instances.");
    assert_ne!(alias_reads, 0, "{code}");
    assert_eq!(
        count(&code, "typeof __vize_slot_host_1_instances."),
        alias_reads,
        "{code}"
    );
    // No slot selection: nothing reads the payload.
    assert_eq!(count(&code, "const __vize_slot_host_1 = "), 0, "{code}");
    assert_eq!(
        count(&code, "$SlotHostSelector<typeof el_badge>"),
        0,
        "{code}"
    );
    assert_eq!(count(&code, "type $SlotsFactory<C>"), 1, "{code}");
    assert_eq!(count(&code, UNREAD_CONTENT_SLOT), 1, "{code}");
}

#[test]
fn only_the_bound_slot_of_a_usage_infers_its_host() {
    let code = generate(
        r#"<el-badge :value="1"><template #icon>i</template><template #content="{ value }">{{ value }}</template></el-badge>"#,
    );
    // `#icon` is the usage's root slot: it keeps the pair the props type
    // reads, and nothing else.
    assert_eq!(
        count(&code, "const __vize_slot_host_1_instances = "),
        1,
        "{code}"
    );
    assert_eq!(count(&code, "const __vize_slot_host_1 = "), 0, "{code}");
    assert_eq!(
        count(
            &code,
            "void function _slot_icon_1(__vize_slot_props_1: unknown) {"
        ),
        1,
        "{code}"
    );
    // `#content` binds `value`, so its host infers the payload.
    assert_eq!(count(&code, "const __vize_slot_host_2 = "), 1, "{code}");
    assert_eq!(
        count(
            &code,
            "  void function _slot_content_2({ value }: $SlotPayload<typeof __vize_slot_host_2, \"content\", $RegisteredSlotProvider<import(\"vue\").GlobalComponents, \"ElBadge\", \"elBadge\", \"el-badge\">>) {\n"
        ),
        1,
        "{code}"
    );
    assert_eq!(
        count(&code, "$SlotHostSelector<typeof el_badge>"),
        1,
        "{code}"
    );
}
