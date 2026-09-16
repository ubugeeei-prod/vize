use vize_canon::{SfcTypeCheckOptions, type_check_sfc, type_check_sfc_with_legacy_vue2};

const COMMENTS: &str = "<script setup>const label = 'hello'</script>\n<template><button\n// caption\n:aria-label=\"label\">{{ label }}</button></template>";
const SLOTS: &str = "<script setup lang=\"ts\">import Child from './Child.vue'</script><template><Child><input /></Child></template>";

#[test]
fn in_tag_comments_are_opt_in_for_single_file_typechecking() {
    let mut options = SfcTypeCheckOptions::new("Example.vue").with_virtual_ts();
    let off = type_check_sfc(COMMENTS, &options);
    options.experimental_in_tag_comments = true;
    let on = type_check_sfc(COMMENTS, &options);
    assert_ne!(
        serde_json::to_value(&off.diagnostics).unwrap(),
        serde_json::to_value(&on.diagnostics).unwrap()
    );
    assert_eq!(on.diagnostics.len(), 0);
    assert!(on.virtual_ts.is_some());
    options.experimental_in_tag_comments = false;
    assert_eq!(
        serde_json::to_value(type_check_sfc(COMMENTS, &options).diagnostics).unwrap(),
        serde_json::to_value(off.diagnostics).unwrap()
    );
}

#[test]
fn strict_slot_contracts_are_opt_in_in_modern_and_legacy_entrypoints() {
    for check in [type_check_sfc, type_check_sfc_with_legacy_vue2] {
        let mut options = SfcTypeCheckOptions::new("Example.vue").with_virtual_ts();
        let off = check(SLOTS, &options).virtual_ts.unwrap();
        options.experimental_strict_slot_children = true;
        let on = check(SLOTS, &options).virtual_ts.unwrap();
        let contracts = |code: &str| -> Vec<std::string::String> {
            code.lines()
                .map(str::trim)
                .filter(|line| line.starts_with("const __vize_slot_children_"))
                .map(str::to_owned)
                .collect()
        };
        assert_eq!(contracts(&off), Vec::<std::string::String>::new());
        assert_eq!(
            contracts(&on),
            [
                "const __vize_slot_children_0_default: __VizeSlotChildren<__VizeSlotContract_0, \"default\"> = null as unknown as __VizeProvidedSlotChildren<[HTMLInputElement]>;",
            ]
        );
        options.experimental_strict_slot_children = false;
        assert_eq!(check(SLOTS, &options).virtual_ts.unwrap(), off);
    }
}
