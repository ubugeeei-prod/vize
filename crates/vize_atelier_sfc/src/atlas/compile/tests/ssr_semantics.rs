use super::*;

fn compile_ssr(name: &str, source_text: &str, scope_id: Option<&str>) -> vize_carton::String {
    let mut compilation = Compilation::new();
    register_compile_test_providers(&mut compilation);
    let source = compilation.add_source(name, source_text).unwrap();
    let mut options = SfcCompileOptions::default();
    options.template.ssr = true;
    options.scope_id = scope_id.map(Into::into);
    install_sfc_compile_request(
        &mut compilation,
        source,
        SfcCompileRequest::new(options, TemplateSyntaxMode::Standard),
    )
    .unwrap();
    compilation
        .query::<SfcCompileProduct>(source)
        .unwrap()
        .value()
        .code
        .clone()
}

#[test]
fn graph_ssr_forwards_root_attrs_through_elements_components_and_if_branches() {
    let conditional = compile_ssr(
        "ConditionalRoot.vue",
        r#"<script setup>const ok = true</script>
<template><main v-if="ok" class="own" /><section v-else /></template>
<style scoped>main, section { color: red }</style>"#,
        Some("rootattrs"),
    );
    assert_eq!(
        conditional.matches("_ssrRenderAttrs(_mergeProps(").count(),
        2,
        "each possible single root must receive fallthrough attrs:\n{conditional}"
    );
    assert!(
        conditional.contains("_mergeProps({\"class\": \"own\"}, _attrs)"),
        "authored and fallthrough classes must be normalized together:\n{conditional}"
    );
    assert_eq!(
        conditional.matches("_push(\" data-v-rootattrs\")").count(),
        2,
        "each branch must retain the component's own scope:\n{conditional}"
    );

    let component = compile_ssr(
        "ComponentRoot.vue",
        r#"<script setup>import Child from './Child.vue'</script>
<template><Child /></template>"#,
        None,
    );
    assert!(
        component.contains("_ssrRenderComponent($setup.Child, _mergeProps({}, _attrs)"),
        "component roots must forward attrs to their child root:\n{component}"
    );
}

#[test]
fn graph_ssr_suspense_uses_dynamic_slot_descriptors_in_push_and_vnode_paths() {
    let code = compile_ssr(
        "DynamicSuspense.vue",
        r#"<script setup>
import Outer from './Outer.vue'
import AsyncView from './AsyncView.vue'
const pending = true
const extraSlots = ['fallback']
</script>
<template>
  <Outer>
    <Suspense>
      <AsyncView />
      <template v-if="pending" #fallback>loading</template>
      <template v-else #fallback>retry</template>
      <template v-for="name in extraSlots" #[name]>dynamic fallback</template>
    </Suspense>
  </Outer>
</template>"#,
        None,
    );
    assert!(
        code.contains("_ssrRenderSuspense(_push, _createSlots("),
        "SSR push Suspense must consume the shared dynamic slot plan:\n{code}"
    );
    assert!(
        code.matches("_createSlots(").count() >= 2,
        "SSR push and VNode fallback must both retain dynamic slots:\n{code}"
    );
    assert!(
        code.contains("($setup.pending) ? { name: \"fallback\"")
            && code.contains("key: \"0\"")
            && code.contains("key: \"1\""),
        "conditional Suspense fallbacks must stay keyed descriptors:\n{code}"
    );
    assert!(
        code.contains("_renderList($setup.extraSlots, (name) => { return { name: name"),
        "iterated dynamic Suspense slots must stay descriptors:\n{code}"
    );
}

#[test]
fn graph_ssr_transition_group_preserves_dynamic_and_fragment_wrappers() {
    let dynamic = compile_ssr(
        "DynamicTransitionGroup.vue",
        r#"<script setup>const tag = 'ul'</script>
<template><TransitionGroup :tag="tag" class="items"><li /></TransitionGroup></template>
<style scoped>.items { display: grid }</style>"#,
        Some("transitiongroup"),
    );
    assert_eq!(
        dynamic.matches("_push(String($setup.tag))").count(),
        2,
        "dynamic TransitionGroup must emit matching open and close tags:\n{dynamic}"
    );
    assert!(
        dynamic.contains("_push(\" data-v-transitiongroup\")"),
        "the dynamic wrapper must retain the component scope:\n{dynamic}"
    );

    let fragment = compile_ssr(
        "FragmentTransitionGroup.vue",
        "<template><TransitionGroup><li /><li /></TransitionGroup></template>",
        None,
    );
    assert!(
        fragment.contains("_push(\"<!--[-->\")") && fragment.contains("_push(\"<!--]-->\")"),
        "tagless TransitionGroup must preserve its hydration fragment boundary:\n{fragment}"
    );
}
