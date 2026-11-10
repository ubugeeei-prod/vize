//! An unbound slot (`<template #name>`) reads no payload, so it infers no
//! host. The root usage's constructor-instance pair stays: its props type
//! reads the actual constructor result from it.

use super::generate;

fn count(code: &str, needle: &str) -> usize {
    code.matches(needle).count()
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
    assert!(
        code.contains("void function _slot_content_1(__vize_slot_props_1: unknown) {"),
        "{code}"
    );
}

#[test]
fn unbound_root_slot_keeps_only_the_constructor_pair_its_props_type_reads() {
    let code = generate(r#"<el-badge :value="1"><template #content>x</template></el-badge>"#);
    // The pair is declared once and read by the usage's props type ...
    assert_eq!(
        count(&code, "const __vize_slot_host_1_instances = "),
        1,
        "{code}"
    );
    assert!(
        code.lines().any(|line| line.contains("type __el_badge_Props_0 = ")
            && line.contains("typeof __vize_slot_host_1_instances.withArguments")),
        "{code}"
    );
    // ... with exactly the constructions the full host binding writes.
    assert!(
        code.contains("const __vize_slot_host_1_instances = (undefined as unknown as <__Z, __P>(withoutArguments: __Z, withArguments: __P) => { readonly withoutArguments: __Z; readonly withArguments: __P })(new (undefined as unknown as $SlotConstructorWithoutArguments<$SlotConstructor<typeof el_badge>>)(), new (undefined as unknown as $SlotConstructorWithArguments<$SlotConstructor<typeof el_badge>>)({\n    \"value\": 1,\n  }, ...([] as unknown as $SlotConstructorArgs<$SlotConstructor<typeof el_badge>>)));\n"),
        "{code}"
    );
    // No slot selection: nothing reads the payload.
    assert_eq!(count(&code, "const __vize_slot_host_1 = "), 0, "{code}");
    assert_eq!(count(&code, "$SlotHostSelector<typeof el_badge>"), 0, "{code}");
    assert_eq!(count(&code, "type $SlotsFactory<C>"), 1, "{code}");
    assert!(
        code.contains("void function _slot_content_1(__vize_slot_props_1: unknown) {"),
        "{code}"
    );
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
    // `#content` binds `value`, so its host infers the payload.
    assert_eq!(count(&code, "const __vize_slot_host_2 = "), 1, "{code}");
    assert!(
        code.contains("({ value }: $SlotPayload<typeof __vize_slot_host_2, \"content\">)"),
        "{code}"
    );
    assert_eq!(count(&code, "$SlotHostSelector<typeof el_badge>"), 1, "{code}");
}
