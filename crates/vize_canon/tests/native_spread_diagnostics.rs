#[path = "support/script_block_project.rs"]
mod project;

#[test]
fn native_spreads_require_objects_and_map_errors_to_the_operand() {
    let source = r#"<script setup lang="ts">
const attrs = { id: 'root' };
const nullable: object | null | undefined = attrs;
const invalid = 1;
</script>
<template>
  <div v-bind="attrs" />
  <div v-bind="nullable" />
  <div v-bind="invalid" />
</template>"#;
    assert_eq!(
        project::check(&[("src/App.vue", source)]),
        ["src/App.vue(9,16): error TS2322: Type 'number' is not assignable to type 'object'."]
    );
}

#[test]
fn dynamic_directive_arguments_require_property_keys() {
    let source = "<script setup lang=\"ts\">const name = {};</script>\n<template>\n  <div :[name]=\"123\"></div>\n</template>";
    assert_eq!(
        project::check(&[("src/App.vue", source)]),
        [
            "src/App.vue(3,9): error TS2464: A computed property name must be of type 'string', 'number', 'symbol', or 'any'."
        ]
    );
}

#[test]
fn nullable_dynamic_listener_names_disable_events_without_key_errors() {
    let source = "<script setup lang=\"ts\">let event: string | null = null; const handler = () => {};</script><template><div @[event]=\"handler\" /></template>";
    assert!(project::check(&[("src/App.vue", source)]).is_empty());
    let invalid = source.replace("@[event]", "@[missing]");
    let diagnostics = project::check(&[("src/App.vue", &invalid)]);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert!(diagnostics[0].contains("TS2304: Cannot find name 'missing'"));
}
