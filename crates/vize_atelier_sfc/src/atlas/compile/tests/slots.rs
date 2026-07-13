use super::*;

#[test]
fn direct_component_v_slot_is_lexical_and_not_a_runtime_directive() {
    let source_text = r#"<script setup>
const outside = 'outside'
</script>
<template>
  <Popover v-slot="{ item }">{{ item.label }} {{ outside }}</Popover>
</template>"#;
    let mut compilation = Compilation::new();
    register_compile_test_providers(&mut compilation);
    let source = compilation
        .add_source("DirectComponentSlot.vue", source_text)
        .unwrap();
    let mut settings = SfcCompileSettings::default();
    settings.insert(
        source,
        SfcCompileRequest::new(SfcCompileOptions::default(), TemplateSyntaxMode::Standard),
    );
    settings.install(&mut compilation).unwrap();

    let compiled = compilation.query::<SfcCompileProduct>(source).unwrap();
    let code = &compiled.value().code;
    assert!(
        code.contains("default: _withCtx(({ item }) => ["),
        "component v-slot must become a default slot:\n{code}"
    );
    assert!(code.contains("_toDisplayString(item.label)"), "{code}");
    assert!(code.contains("_toDisplayString($setup.outside)"), "{code}");
    assert!(!code.contains("_ctx.item"), "{code}");
    assert!(!code.contains("_resolveDirective(\"slot\")"), "{code}");
}

#[test]
fn suspense_fallback_reaches_dom_ssr_and_ssr_vnode_fallback() {
    let source_text = r#"<script setup>
import Outer from './Outer.vue'
import AsyncView from './AsyncView.vue'
</script>
<template>
  <Outer>
    <Suspense>
      <AsyncView />
      <template #fallback>loading</template>
    </Suspense>
  </Outer>
</template>"#;
    let mut compilation = Compilation::new();
    register_compile_test_providers(&mut compilation);
    let dom_source = compilation
        .add_source("SuspenseDom.vue", source_text)
        .unwrap();
    let ssr_source = compilation
        .add_source("SuspenseSsr.vue", source_text)
        .unwrap();
    let mut ssr_options = SfcCompileOptions::default();
    ssr_options.template.ssr = true;
    let mut settings = SfcCompileSettings::default();
    settings.insert(
        dom_source,
        SfcCompileRequest::new(SfcCompileOptions::default(), TemplateSyntaxMode::Standard),
    );
    settings.insert(
        ssr_source,
        SfcCompileRequest::new(ssr_options, TemplateSyntaxMode::Standard),
    );
    settings.install(&mut compilation).unwrap();

    let dom = compilation.query::<SfcCompileProduct>(dom_source).unwrap();
    assert!(
        dom.value()
            .code
            .contains("fallback: _withCtx(() => [\"loading\"])"),
        "DOM fallback slot was dropped:\n{}",
        dom.value().code
    );

    let ssr = compilation.query::<SfcCompileProduct>(ssr_source).unwrap();
    let code = &ssr.value().code;
    assert!(code.contains("_ssrRenderSuspense(_push, {"), "{code}");
    assert!(
        code.contains("\"fallback\": () => {") && code.contains("_push(\"loading\")"),
        "SSR push fallback slot was dropped:\n{code}"
    );
    assert!(
        code.contains("\"fallback\": _withCtx(() => [_createTextVNode(\"loading\")])"),
        "SSR VNode fallback slot was dropped:\n{code}"
    );
}

#[test]
fn structural_named_slots_use_create_slots_in_every_rendu_backend_path() {
    let source_text = r#"<script setup>
import Outer from './Outer.vue'
import Inner from './Inner.vue'
const ok = true
const slots = { leading: true, trailing: true }
</script>
<template>
  <Outer>
    <Inner>
      <template v-if="ok" #header="{ title }"><h1>{{ title }}</h1></template>
      <template v-else #header><h1>fallback header</h1></template>
      <template v-for="(_, name) in slots" #[name]="slotData">
        <slot :name="name" v-bind="slotData" />
      </template>
      <template #footer>static footer</template>
    </Inner>
  </Outer>
</template>"#;
    let mut compilation = Compilation::new();
    register_compile_test_providers(&mut compilation);
    let dom_source = compilation
        .add_source("DynamicSlotsDom.vue", source_text)
        .unwrap();
    let ssr_source = compilation
        .add_source("DynamicSlotsSsr.vue", source_text)
        .unwrap();
    let vapor_source = compilation
        .add_source("DynamicSlotsVapor.vue", source_text)
        .unwrap();
    let mut ssr_options = SfcCompileOptions::default();
    ssr_options.template.ssr = true;
    let mut vapor_options = SfcCompileOptions::default();
    vapor_options.vapor = true;
    let mut settings = SfcCompileSettings::default();
    settings.insert(
        dom_source,
        SfcCompileRequest::new(SfcCompileOptions::default(), TemplateSyntaxMode::Standard),
    );
    settings.insert(
        ssr_source,
        SfcCompileRequest::new(ssr_options, TemplateSyntaxMode::Standard),
    );
    settings.insert(
        vapor_source,
        SfcCompileRequest::new(vapor_options, TemplateSyntaxMode::Standard),
    );
    settings.install(&mut compilation).unwrap();

    let dom = compilation.query::<SfcCompileProduct>(dom_source).unwrap();
    let dom_code = &dom.value().code;
    assert!(dom_code.contains("_createSlots("), "{dom_code}");
    assert!(
        dom_code.contains("($setup.ok) ? { name: \"header\"")
            && dom_code.contains("key: \"0\"")
            && dom_code.contains("key: \"1\""),
        "conditional slots must remain keyed descriptors:\n{dom_code}"
    );
    assert!(
        dom_code.contains("_renderList($setup.slots, (_, name) => { return { name: name"),
        "iterated dynamic slot must remain a renderList descriptor:\n{dom_code}"
    );
    assert!(dom_code.contains("footer: _withCtx("), "{dom_code}");
    for leaked in ["_ctx.title", "_ctx.name", "_ctx.slotData"] {
        assert!(!dom_code.contains(leaked), "leaked {leaked}:\n{dom_code}");
    }

    let ssr = compilation.query::<SfcCompileProduct>(ssr_source).unwrap();
    let ssr_code = &ssr.value().code;
    assert!(
        ssr_code.matches("_createSlots(").count() >= 2,
        "SSR push and VNode fallback must share dynamic slot planning:\n{ssr_code}"
    );
    assert!(
        ssr_code.contains("_renderList($setup.slots, (_, name) => { return { name: name"),
        "SSR iterated slot descriptor was lost:\n{ssr_code}"
    );
    assert!(
        ssr_code.contains("($setup.ok) ? { name: \"header\"")
            && ssr_code.contains("key: \"0\"")
            && ssr_code.contains("key: \"1\""),
        "SSR conditional slot descriptors were lost:\n{ssr_code}"
    );
    for leaked in ["_ctx.title", "_ctx.name", "_ctx.slotData"] {
        assert!(!ssr_code.contains(leaked), "leaked {leaked}:\n{ssr_code}");
    }

    let vapor = compilation
        .query::<SfcCompileProduct>(vapor_source)
        .unwrap();
    let vapor_code = &vapor.value().code;
    assert!(
        vapor_code.contains("$: [() => (($setup.ok) ? { name: \"header\"")
            && vapor_code.contains(": { name: \"header\""),
        "conditional Vapor slots must remain reactive descriptors:\n{vapor_code}"
    );
    assert!(
        vapor_code.contains("() => (_createForSlots($setup.slots, (_, name) => ({ name: name"),
        "iterated Vapor slots must use createForSlots:\n{vapor_code}"
    );
    assert!(
        vapor_code.contains("fn: _withVaporCtx((slotData) =>"),
        "forwarded Vapor slots must retain their slot owner:\n{vapor_code}"
    );
    assert!(vapor_code.contains("\"footer\":"), "{vapor_code}");
    for leaked in ["_ctx.title", "_ctx.name", "_ctx.slotData"] {
        assert!(
            !vapor_code.contains(leaked),
            "leaked {leaked}:\n{vapor_code}"
        );
    }
}
