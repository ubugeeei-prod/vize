#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use davinci_test_support::script_block_project as project;

#[test]
fn authored_slot_types_and_the_public_component_contract_are_distinct() {
    let component = r#"<script setup lang="ts">
type Slots = { header: (props: { count: number }) => unknown };
defineSlots<Slots>();
</script><template><div /></template>"#;
    let consumer = "import type { Slots } from './App.vue';\nexport const slots: Slots = { header: ({ count }) => count.toFixed() };";
    assert_eq!(
        project::check(&[("src/App.vue", component), ("src/consumer.ts", consumer)]),
        [] as [String; 0]
    );
    let invalid = consumer.replace("count.toFixed()", "count.toUpperCase()");
    assert_eq!(
        project::check(&[("src/App.vue", component), ("src/consumer.ts", &invalid)]),
        [
            "src/consumer.ts(2,60): error TS2339: Property 'toUpperCase' does not exist on type 'number'."
        ]
    );
}

#[test]
fn imported_slot_types_are_reexported_without_redeclaring_the_binding() {
    let component = r#"<script setup lang="ts">
import type { SharedSlots as Slots } from './slots';
defineSlots<Slots>();
</script>"#;
    let slots = "export type SharedSlots = { default: (props: { value: string }) => unknown };";
    let consumer = "import type { Slots } from './App.vue';\nexport const slots: Slots = { default: ({ value }) => value.toUpperCase() };";
    assert_eq!(
        project::check(&[
            ("src/App.vue", component),
            ("src/slots.ts", slots),
            ("src/consumer.ts", consumer),
        ]),
        [] as [String; 0]
    );
}

#[test]
fn an_explicit_normal_script_slot_export_keeps_its_authored_meaning() {
    let component =
        "<script lang=\"ts\">export type Slots = { named: string }; export default {};</script>";
    let consumer =
        "import type { Slots } from './App.vue';\nexport const value: Slots = { named: 'value' };";
    assert_eq!(
        project::check(&[("src/App.vue", component), ("src/consumer.ts", consumer)]),
        [] as [String; 0]
    );
}
