use super::*;

const BUILTINS_SOURCE: &str = r#"<script setup>
import { BaseTransition, Suspense, Teleport, KeepAlive, Transition, TransitionGroup } from 'vue'
const target = 'body'
const view = 'section'
</script>
<template>
  <Suspense><template #default><div /></template><template #fallback><i /></template></Suspense>
  <Teleport :to="target"><span /></Teleport>
  <KeepAlive><Child /></KeepAlive>
  <Transition><div /></Transition>
  <TransitionGroup tag="ul"><li /></TransitionGroup>
  <Box><BaseTransition><em /></BaseTransition></Box>
  <component :is="view" class="dynamic" />
  <component />
</template>"#;

fn compile_builtins(target: SfcRenderTarget) -> vize_carton::String {
    let mut compilation = Compilation::new();
    register_compile_test_providers(&mut compilation);
    let source = compilation
        .add_source("Builtins.vue", BUILTINS_SOURCE)
        .unwrap();
    let mut options = SfcCompileOptions::default();
    options.template.ssr = target == SfcRenderTarget::Ssr;
    options.vapor = target == SfcRenderTarget::Vapor;
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
fn dom_uses_typed_builtin_identity_and_consumes_dynamic_is() {
    let code = compile_builtins(SfcRenderTarget::Dom);
    for helper in [
        "_h(_Suspense",
        "_h(_Teleport",
        "_h(_KeepAlive",
        "_h(_Transition",
        "_h(_TransitionGroup",
        "_h(_BaseTransition",
    ] {
        assert!(code.contains(helper), "missing {helper}:\n{code}");
    }
    assert!(
        code.contains("_h(_resolveDynamicComponent($setup.view), {class: \"dynamic\"}"),
        "{code}"
    );
    assert!(
        code.contains("_h(_resolveComponent(\"component\")"),
        "{code}"
    );
    assert!(!code.contains("\"is\":"), "{code}");
}

#[test]
fn ssr_dispatches_builtins_by_kind_even_after_setup_binding_resolution() {
    let code = compile_builtins(SfcRenderTarget::Ssr);
    assert!(code.contains("_ssrRenderSuspense(_push"), "{code}");
    assert!(code.contains("_ssrRenderTeleport(_push"), "{code}");
    assert!(code.contains("_push(\"<ul\")"), "{code}");
    assert!(code.contains("_createVNode(_BaseTransition"), "{code}");
    assert!(
        code.contains("_ssrRenderComponent(_resolveDynamicComponent($setup.view)"),
        "{code}"
    );
    assert!(
        code.contains("_ssrRenderComponent(_resolveComponent(\"component\")"),
        "{code}"
    );
    assert!(!code.contains("\"is\":"), "{code}");
}

#[test]
fn vapor_selects_builtin_and_dynamic_component_helpers_from_the_same_kind() {
    let code = compile_builtins(SfcRenderTarget::Vapor);
    for helper in [
        "_createComponent(_Suspense",
        "_createComponent(_VaporTeleport",
        "_createComponent(_VaporKeepAlive",
        "_createComponent(_VaporTransition",
        "_createComponent(_VaporTransitionGroup",
    ] {
        assert!(code.contains(helper), "missing {helper}:\n{code}");
    }
    assert!(
        code.matches("_createComponent(_VaporTransition").count() >= 2,
        "BaseTransition and Transition must both retain Vapor transition semantics:\n{code}"
    );
    assert!(
        code.contains("_createDynamicComponent(() => ($setup.view)"),
        "{code}"
    );
    assert!(
        code.contains("_createComponentWithFallback(_resolveComponent(\"component\")"),
        "{code}"
    );
    assert!(!code.contains("\"is\":"), "{code}");
}
