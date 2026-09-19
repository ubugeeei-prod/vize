#[path = "support/script_block_project.rs"]
mod project;

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
