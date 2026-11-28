use crate::sfc_typecheck::{SfcTypeCheckOptions, type_check_sfc};

fn virtual_ts(render: &str) -> vize_carton::String {
    let source = vize_carton::cstr!(
        r#"<script lang="tsx">
import {{ defineComponent, useTemplateRef }} from 'vue'

export default defineComponent({{
  setup() {{
    const items = [1, 2]
    const target = useTemplateRef('row')
    const missing = useTemplateRef('missing')
    return () => {render}
  }},
}})
</script>"#
    );
    let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
    let result = type_check_sfc(source.as_str(), &options);
    vize_carton::String::from(
        result
            .virtual_ts
            .expect("type_check_sfc must produce virtual TypeScript")
            .as_str(),
    )
}

#[test]
fn a_map_child_ref_holds_an_array_and_other_refs_one_element() {
    let ts = virtual_ts(r#"<div ref="box">{items.map((item) => <li ref="row">{item}</li>)}</div>"#);
    assert!(
        ts.contains(
            r#"type $TemplateRefs = { "box": $DomElement<"div">; "row": $DomElement<"li">[]; [key: string]: unknown; };"#
        ),
        "{ts}"
    );
    assert!(ts.contains("const target = __vizeInferRef('row')"), "{ts}");
    assert!(
        ts.contains("const missing = __vizeInferRef('missing')"),
        "{ts}"
    );
    // A render helper of another module can register a ref too, so an
    // unlisted key reads the open index signature instead of an error.
    assert!(
        ts.contains(r#"void (null! as $TemplateRefs["missing"]); "#),
        "{ts}"
    );
}

#[test]
fn control_flow_arms_and_function_callbacks_lower_to_lists() {
    let ts = virtual_ts(
        r#"<ul>{items.length > 0 && items.map(function (item) { return <li ref="row">{item}</li> })}{items.length ? items.map((item) => <i ref="mark">{item}</i>) : null}</ul>"#,
    );
    assert!(
        ts.contains(
            r#"type $TemplateRefs = { "row": $DomElement<"li">[]; "mark": $DomElement<"i">[]; [key: string]: unknown; };"#
        ),
        "{ts}"
    );
}

/// Only a `.map()` JSX child lowers to `v-for`; elsewhere the ref is set once
/// per element and keeps the last.
#[test]
fn a_map_outside_a_jsx_child_keeps_one_element() {
    let ts = virtual_ts(
        r#"{ const rows = items.map((item) => <li ref="row">{item}</li>); return <ul>{rows}{items?.map((item) => <i ref="mark">{item}</i>)}</ul> }"#,
    );
    assert!(
        ts.contains(
            r#"type $TemplateRefs = { "row": $DomElement<"li">; "mark": $DomElement<"i">; [key: string]: unknown; };"#
        ),
        "{ts}"
    );
}

#[test]
fn svg_refs_resolve_through_the_svg_tag_map() {
    let ts = virtual_ts(
        r#"<svg ref="icon"><circle ref="dot" /><foreignObject><a ref="link" /></foreignObject></svg>"#,
    );
    assert!(
        ts.contains(r#"type $TemplateRefs = { "icon": $DomElement<"svg", true>; "dot": $DomElement<"circle", true>; "link": $DomElement<"a">; [key: string]: unknown; };"#),
        "{ts}"
    );
}

/// A name a component also registers is left out, never typed as a partial
/// union, and a script without typed refs keeps Vue's own signature.
#[test]
fn a_name_a_component_registers_stays_untyped() {
    let ts = virtual_ts(r#"<div><Child ref="row" /><li ref="row" /><p ref="text" /></div>"#);
    assert!(
        ts.contains(
            r#"type $TemplateRefs = { "text": $DomElement<"p">; [key: string]: unknown; };"#
        ),
        "{ts}"
    );
    let ts = virtual_ts(r#"<Child ref="row" />"#);
    assert!(!ts.contains("$TemplateRefs"), "{ts}");
    assert!(ts.contains("const target = useTemplateRef('row')"), "{ts}");
}

/// A template renders instead of the script's JSX, so it alone registers refs.
#[test]
fn a_template_keeps_its_own_registry() {
    let source = r#"<script setup lang="tsx">
import { useTemplateRef } from 'vue'
const items = [1, 2]
const target = useTemplateRef('row')
const Render = () => <ul>{items.map((item) => <li ref="row">{item}</li>)}</ul>
</script>

<template>
  <Render />
  <p ref="text" />
</template>"#;
    let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
    let ts = type_check_sfc(source, &options)
        .virtual_ts
        .expect("type_check_sfc must produce virtual TypeScript");
    assert!(
        ts.contains(r#"type $TemplateRefs = { "text": $DomElement<"p">; };"#),
        "{ts}"
    );
    assert!(
        ts.contains(r#"void (null! as $TemplateRefs["row"]); "#),
        "{ts}"
    );
}
