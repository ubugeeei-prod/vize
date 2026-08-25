//! End-to-end SFC regression coverage for lone `v-bind` object props.
//!
//! Reduced from Nuxt UI's `src/runtime/components/Icon.vue`, whose entire
//! template is a `v-if` component carrying nothing but `v-bind="iconProps"`.
//! When that single prop segment is dropped the component server-renders with
//! every prop undefined, which took down the whole Nuxt UI playground with
//! `Missing required prop: "name"` followed by an SSR 500.

use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
use vize_carton::String;

const NUXT_UI_ICON_SFC: &str = r#"<script setup lang="ts">
const props = defineProps<{ name: string }>()
const iconProps = useForwardProps(props)
</script>

<template>
  <Icon v-if="typeof name === 'string'" v-bind="iconProps" />
  <component :is="name" v-else />
</template>
"#;

fn compile_ssr_module(source: &str) -> String {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse SFC");
    let options = SfcCompileOptions {
        template: vize_atelier_sfc::TemplateCompileOptions {
            ssr: true,
            is_ts: true,
            ..Default::default()
        },
        script: vize_atelier_sfc::ScriptCompileOptions {
            is_ts: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, options).expect("compile SFC");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    result.code
}

#[test]
fn nuxt_ui_icon_sfc_forwards_its_setup_bound_spread() {
    assert_eq!(
        compile_ssr_module(NUXT_UI_ICON_SFC),
        r#"import { defineComponent as _defineComponent } from 'vue'
import { ssrRenderVNode as _ssrRenderVNode, ssrRenderComponent as _ssrRenderComponent } from "@vue/server-renderer"
import { createVNode as _createVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, mergeProps as _mergeProps, normalizeProps as _normalizeProps, guardReactiveProps as _guardReactiveProps } from "vue"


function ssrRender(_ctx, _push, _parent, _attrs, $props, $setup, $data, $options) {
  if (typeof $props.name === 'string') {
    _push(_ssrRenderComponent(_resolveComponent("Icon"), _mergeProps(_normalizeProps(_guardReactiveProps($setup.iconProps)), _attrs), null, _parent))
  } else {
    _ssrRenderVNode(_push, _createVNode(_resolveDynamicComponent($props.name), _attrs, null), _parent)
  }
}

export default /*@__PURE__*/_defineComponent({
  __name: 'anonymous',
  props: {
    name: { type: String, required: true }
  },
  ssrRender: ssrRender,
  setup(__props: any) {

const props = __props
const iconProps = useForwardProps(props)

const __returned__ = { props, iconProps }
Object.defineProperty(__returned__, '__isScriptSetup', { enumerable: false, value: true })
return __returned__
}

})"#
    );
}
