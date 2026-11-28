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
    sfc_virtual_ts(source.as_str())
}

fn sfc_virtual_ts(source: &str) -> vize_carton::String {
    let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
    let result = type_check_sfc(source, &options);
    vize_carton::String::from(
        result
            .virtual_ts
            .expect("type_check_sfc must produce virtual TypeScript")
            .as_str(),
    )
}

/// Every generated line that declares, specializes or checks a template ref.
fn ref_lines(ts: &str) -> Vec<&str> {
    ts.lines()
        .map(str::trim)
        .filter(|line| line.contains("TemplateRef") || line.contains("InferRef"))
        .collect()
}

/// The registry type the setup scope declares, if any.
fn registry(ts: &str) -> Option<&str> {
    ts.lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("type $TemplateRefs = "))
}

const USE_TEMPLATE_REF: &str = r#"type $UseTemplateRef = { <_K extends keyof $TemplateRefs>(_key: _K): Readonly<import('vue').ShallowRef<$TemplateRefs[_K] | null>>; <_T = unknown>(_key: string): Readonly<import('vue').ShallowRef<_T | null>>; };"#;

#[test]
fn a_map_child_ref_holds_an_array_and_other_refs_one_element() {
    let ts = virtual_ts(r#"<div ref="box">{items.map((item) => <li ref="row">{item}</li>)}</div>"#);
    // A render helper of another module can register a ref too, so an
    // unlisted key reads the open index signature instead of an error.
    assert_eq!(
        ref_lines(&ts),
        [
            "import { defineComponent, useTemplateRef } from 'vue'",
            "void useTemplateRef;",
            r#"type $TemplateRefs = { "box": $DomElement<"div">; "row": $DomElement<"li">[]; [key: string]: unknown; };"#,
            USE_TEMPLATE_REF,
            "const __vizeInferRef = (undefined as unknown as $UseTemplateRef);",
            r#"void (null! as $TemplateRefs["row"]); void (null! as $TemplateRefs["missing"]); const __default__ = defineComponent({"#,
            "const target = __vizeInferRef('row')",
            "const missing = __vizeInferRef('missing')",
        ]
    );
}

#[test]
fn control_flow_arms_and_function_callbacks_lower_to_lists() {
    let ts = virtual_ts(
        r#"<ul>{items.length > 0 && items.map(function (item) { return <li ref="row">{item}</li> })}{items.length ? items.map((item) => <i ref="mark">{item}</i>) : null}</ul>"#,
    );
    assert_eq!(
        registry(&ts),
        Some(
            r#"{ "row": $DomElement<"li">[]; "mark": $DomElement<"i">[]; [key: string]: unknown; };"#
        )
    );
}

/// Only a `.map()` JSX child lowers to `v-for`; elsewhere the ref is set once
/// per element and keeps the last.
#[test]
fn a_map_outside_a_jsx_child_keeps_one_element() {
    let ts = virtual_ts(
        r#"{ const rows = items.map((item) => <li ref="row">{item}</li>); return <ul>{rows}{items?.map((item) => <i ref="mark">{item}</i>)}</ul> }"#,
    );
    assert_eq!(
        registry(&ts),
        Some(r#"{ "row": $DomElement<"li">; "mark": $DomElement<"i">; [key: string]: unknown; };"#)
    );
}

#[test]
fn svg_refs_resolve_through_the_svg_tag_map() {
    let ts = virtual_ts(
        r#"<svg ref="icon"><circle ref="dot" /><foreignObject><a ref="link" /></foreignObject></svg>"#,
    );
    assert_eq!(
        registry(&ts),
        Some(
            r#"{ "icon": $DomElement<"svg", true>; "dot": $DomElement<"circle", true>; "link": $DomElement<"a">; [key: string]: unknown; };"#
        )
    );
}

/// A name a component also registers is left out, never typed as a partial
/// union, and a script without typed refs keeps Vue's own signature.
#[test]
fn a_name_a_component_registers_stays_untyped() {
    let ts = virtual_ts(r#"<div><Child ref="row" /><li ref="row" /><p ref="text" /></div>"#);
    assert_eq!(
        registry(&ts),
        Some(r#"{ "text": $DomElement<"p">; [key: string]: unknown; };"#)
    );
    let ts = virtual_ts(r#"<Child ref="row" />"#);
    assert_eq!(
        ref_lines(&ts),
        [
            "import { defineComponent, useTemplateRef } from 'vue'",
            "void useTemplateRef;",
            "const target = useTemplateRef('row')",
            "const missing = useTemplateRef('missing')",
        ]
    );
}

/// A template renders instead of the script's JSX, so it alone registers refs
/// and a key it never registers is checked.
#[test]
fn a_template_keeps_its_own_registry() {
    let ts = sfc_virtual_ts(TEMPLATE_SFC);
    assert_eq!(
        ref_lines(&ts),
        [
            "import { useTemplateRef } from 'vue'",
            "void useTemplateRef;",
            r#"type $TemplateRefs = { "text": $DomElement<"p">; };"#,
            USE_TEMPLATE_REF,
            "const __vizeInferRef = (undefined as unknown as $UseTemplateRef);",
            r#"void (null! as $TemplateRefs["row"]); const target = __vizeInferRef('row')"#,
            "void Render; void items; void target; void (null as unknown as keyof typeof useTemplateRef);",
        ]
    );
}

const TEMPLATE_SFC: &str = r#"<script setup lang="tsx">
import { useTemplateRef } from 'vue'
const items = [1, 2]
const target = useTemplateRef('row')
const Render = () => <ul>{items.map((item) => <li ref="row">{item}</li>)}</ul>
</script>

<template>
  <Render />
  <p ref="text" />
</template>"#;
