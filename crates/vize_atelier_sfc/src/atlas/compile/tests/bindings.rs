use super::*;
use crate::{SfcCroquisMode, SfcCroquisRequest, SfcCroquisSettingsInput};

#[test]
fn normal_script_type_props_reach_croquis_and_dom_codegen() {
    let source_text = r#"<template>
  <span :style="{ maxWidth: `${100 / minScale}%` }"></span>
</template>
<script lang="ts">
interface Props {
  readonly minScale?: number;
}
</script>
<script setup lang="ts">
const props = withDefaults(defineProps<Props>(), { minScale: 0 });
void props;
</script>"#;
    let mut compilation = Compilation::new();
    register_compile_test_providers(&mut compilation);
    let source = compilation
        .add_source("ReadonlyInterfaceProps.vue", source_text)
        .unwrap();
    let mut settings = SfcCompileSettings::default();
    settings.insert(
        source,
        SfcCompileRequest::new(SfcCompileOptions::default(), TemplateSyntaxMode::Standard),
    );
    settings.install(&mut compilation).unwrap();

    let document = compilation.query::<CroquisDocumentProduct>(source).unwrap();
    let semantics = document.value().analysis();
    assert!(semantics.bindings.is_prop("minScale"));
    assert!(
        semantics
            .macros
            .props()
            .iter()
            .any(|prop| prop.name == "minScale")
    );

    let compiled = compilation.query::<SfcCompileProduct>(source).unwrap();
    assert!(
        compiled.value().code.contains("$props.minScale"),
        "{}",
        compiled.value().code
    );
    assert!(!compiled.value().code.contains("_ctx.minScale"));
}

#[test]
fn setup_component_bindings_survive_croquis_to_rendu_lowering() {
    let source_text = r#"<script setup>
import ChildCard from './ChildCard.vue'
import * as Cards from './Cards.tsx'
import { Primitive } from '@tresjs/core'
</script>
<template>
  <child-card />
  <Cards.Button />
  <primitive />
</template>"#;
    let mut compilation = Compilation::new();
    register_compile_test_providers(&mut compilation);
    let source = compilation
        .add_source("ComponentBindings.vue", source_text)
        .unwrap();
    let mut settings = SfcCompileSettings::default();
    settings.insert(
        source,
        SfcCompileRequest::new(SfcCompileOptions::default(), TemplateSyntaxMode::Standard),
    );
    settings.install(&mut compilation).unwrap();

    let compiled = compilation.query::<SfcCompileProduct>(source).unwrap();
    let code = &compiled.value().code;
    for binding in [
        "$setup.ChildCard",
        "$setup.Cards.Button",
        "$setup.Primitive",
    ] {
        assert!(code.contains(binding), "missing {binding}:\n{code}");
    }
    assert!(!code.contains("_resolveComponent(\""), "{code}");
}

#[test]
fn normal_script_runtime_bindings_reach_setup_return_and_ssr_render() {
    let source_text = r#"<script lang="ts">
import { type FormFieldState, Form as PForm } from '@primevue/forms'
import { valibotResolver } from '@primevue/forms/resolvers/valibot'

export interface FormProps { schema?: unknown }
</script>
<script setup lang="ts">
const { schema } = defineProps<FormProps>()
const emit = defineEmits<{ submit: [] }>()
</script>
<template>
  <PForm :resolver="schema ? valibotResolver(schema) : undefined" @submit="emit('submit')" />
</template>"#;
    let mut compilation = Compilation::new();
    register_compile_test_providers(&mut compilation);
    let source = compilation
        .add_source("NormalRuntimeBindings.vue", source_text)
        .unwrap();
    let mut options = SfcCompileOptions::default();
    options.template.ssr = true;
    let mut settings = SfcCompileSettings::default();
    settings.insert(
        source,
        SfcCompileRequest::new(options, TemplateSyntaxMode::Standard),
    );
    settings.install(&mut compilation).unwrap();

    let compiled = compilation.query::<SfcCompileProduct>(source).unwrap();
    let code = &compiled.value().code;
    assert!(code.contains("Form as PForm"), "{code}");
    assert!(code.contains("import { valibotResolver }"), "{code}");
    assert!(!code.contains("FormFieldState"), "{code}");
    let returned = code
        .split("const __returned__ = {")
        .nth(1)
        .expect("script setup should expose template bindings");
    for binding in ["PForm", "emit", "valibotResolver"] {
        assert!(returned.contains(binding), "missing {binding}:\n{code}");
    }
    assert!(code.contains("_ssrRenderComponent($setup.PForm"), "{code}");
    assert!(
        code.contains("$setup.valibotResolver($props.schema)"),
        "{code}"
    );
}

#[test]
fn options_api_event_handlers_keep_live_instance_lookup() {
    let source_text = r#"<script>
export default {
  methods: {
    onFocus(event) { this.$emit('focus', event) }
  }
}
</script>
<template><input @focus="onFocus" /></template>"#;
    let mut compilation = Compilation::new();
    register_compile_test_providers(&mut compilation);
    let source = compilation
        .add_source("OptionsEvent.vue", source_text)
        .unwrap();
    let mut settings = SfcCompileSettings::default();
    settings.insert(
        source,
        SfcCompileRequest::new(SfcCompileOptions::default(), TemplateSyntaxMode::Standard),
    );
    settings.install(&mut compilation).unwrap();

    let document = compilation.query::<CroquisDocumentProduct>(source).unwrap();
    assert_eq!(
        document.value().analysis().bindings.get("onFocus"),
        Some(vize_croquis::BindingType::Options)
    );
    let compiled = compilation.query::<SfcCompileProduct>(source).unwrap();
    assert!(
        compiled
            .value()
            .code
            .contains("onFocus\": (...args) => (_ctx.onFocus && _ctx.onFocus(...args))"),
        "{}",
        compiled.value().code
    );
    assert!(!compiled.value().code.contains("$options.onFocus"));
}

#[test]
fn lexical_aliases_shadow_options_api_event_handlers() {
    let source_text = r#"<script>
export default { methods: { onFocus() {} }, data: () => ({ handlers: [] }) }
</script>
<template><button v-for="onFocus in handlers" @click="onFocus" /></template>"#;
    let mut compilation = Compilation::new();
    register_compile_test_providers(&mut compilation);
    let source = compilation
        .add_source("OptionsEventShadow.vue", source_text)
        .unwrap();
    let mut settings = SfcCompileSettings::default();
    settings.insert(
        source,
        SfcCompileRequest::new(SfcCompileOptions::default(), TemplateSyntaxMode::Standard),
    );
    settings.install(&mut compilation).unwrap();

    let compiled = compilation.query::<SfcCompileProduct>(source).unwrap();
    let code = &compiled.value().code;
    assert!(code.contains("(onFocus) => _h(\"button\""), "{code}");
    assert!(code.contains("\"onClick\": onFocus"), "{code}");
    assert!(!code.contains("_ctx.onFocus && _ctx.onFocus"), "{code}");
}

#[test]
fn explicit_compile_request_preserves_virtual_sfc_source_identity() {
    let source_text = r#"<script setup lang="ts">const msg = 'hello'</script>
<template><div>{{ msg }}</div></template>"#;
    let mut compilation = Compilation::new();
    register_compile_test_providers(&mut compilation);
    let source = compilation
        .add_source("/virtual/Card.setup.ts", source_text)
        .unwrap();
    let mut settings = SfcCompileSettings::default();
    settings.insert(
        source,
        SfcCompileRequest::new(SfcCompileOptions::default(), TemplateSyntaxMode::Standard),
    );
    settings.install(&mut compilation).unwrap();

    let compiled = compilation.query::<SfcCompileProduct>(source).unwrap();
    assert!(compiled.value().code.contains("export default _sfc_main"));
    let descriptor = compilation.query::<SfcDescriptorProduct>(source).unwrap();
    assert!(descriptor.value().descriptor().is_some());
}

#[test]
fn dual_script_projects_options_and_setup_bindings_without_full_croquis() {
    let source_text = r#"<script>
export default { data: () => ({ count: 1 }) }
</script>
<script setup>const message = 'ready'</script>
<template><p>{{ count }} {{ message }}</p></template>"#;
    let mut compilation = Compilation::new();
    register_compile_test_providers(&mut compilation);
    let source = compilation
        .add_source("DualScript.vue", source_text)
        .unwrap();
    install_sfc_compile_request(
        &mut compilation,
        source,
        SfcCompileRequest::new(SfcCompileOptions::default(), TemplateSyntaxMode::Standard),
    )
    .unwrap();

    let bindings = compilation
        .query::<SfcTemplateBindingsProduct>(source)
        .unwrap();
    assert_eq!(
        bindings.value().bindings.get("count"),
        Some(&vize_carton::BindingType::Data)
    );
    assert_eq!(
        bindings.value().bindings.get("message"),
        Some(&vize_carton::BindingType::LiteralConst)
    );
    let compiled = compilation.query::<SfcCompileProduct>(source).unwrap();
    assert!(!compiled.plan().contains::<CroquisDocumentProduct>());
    assert!(compiled.value().code.contains("$data.count"));
    assert!(compiled.value().code.contains("$setup.message"));
}

#[test]
fn inferred_binding_mode_refreshes_after_a_persistent_source_changes_shape() {
    let mut compilation = Compilation::new();
    register_compile_test_providers(&mut compilation);
    let source = compilation
        .add_source(
            "Persistent.vue",
            "<script setup>const message = 'ready'</script><template>{{ message }}</template>",
        )
        .unwrap();
    let request =
        SfcCompileRequest::new(SfcCompileOptions::default(), TemplateSyntaxMode::Standard);
    install_sfc_compile_request(&mut compilation, source, request.clone()).unwrap();
    let first = compilation
        .query::<SfcTemplateBindingsProduct>(source)
        .unwrap();
    assert!(!first.value().bindings.contains_key("count"));

    compilation
        .update_source(
            source,
            r#"<script>export default { data: () => ({ count: 1 }) }</script>
<script setup>const message = 'ready'</script><template>{{ count }} {{ message }}</template>"#,
        )
        .unwrap();
    let second = compilation
        .query::<SfcTemplateBindingsProduct>(source)
        .unwrap();
    assert_eq!(
        second.value().bindings.get("count"),
        Some(&vize_carton::BindingType::Data)
    );
}

#[test]
fn explicit_semantic_mode_overrides_compiler_inference() {
    let mut compilation = Compilation::new();
    register_compile_test_providers(&mut compilation);
    let source = compilation
        .add_source(
            "ExplicitMode.vue",
            "<script>export default { data: () => ({ count: 1 }) }</script><template>{{ count }}</template>",
        )
        .unwrap();
    compilation
        .set_source_input::<SfcCroquisSettingsInput>(
            source,
            SfcCroquisRequest {
                mode: SfcCroquisMode::Full,
                ..Default::default()
            },
        )
        .unwrap();
    install_sfc_compile_request(
        &mut compilation,
        source,
        SfcCompileRequest::new(SfcCompileOptions::default(), TemplateSyntaxMode::Standard),
    )
    .unwrap();

    let bindings = compilation
        .query::<SfcTemplateBindingsProduct>(source)
        .unwrap();
    assert!(!bindings.value().bindings.contains_key("count"));
}
