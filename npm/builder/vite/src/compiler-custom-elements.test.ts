/**
 * `compiler.customElements` tag patterns must compile matched renderer tags as
 * elements while explicit script setup imports still resolve as components.
 */
import assert from "node:assert/strict";

import { compileFile } from "./compiler.ts";

const customElementPatternSource = `<script setup lang="ts">
import { TresCanvas } from "@tresjs/core";
const visible = true;
</script>

<template>
  <TresCanvas>
    <TresMesh v-if="visible">
      <TresSpotLight />
    </TresMesh>
  </TresCanvas>
</template>`;

const customElementPatternCompiled = compileFile(
  "/src/TresCustomElements.vue",
  new Map(),
  {
    sourceMap: false,
    ssr: false,
    vapor: false,
    customRenderer: true,
    customElements: ["Tres*"],
  },
  customElementPatternSource,
);

assert.equal(
  customElementPatternCompiled.code,
  `import { createElementVNode as _createElementVNode, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, withCtx as _withCtx } from "vue";
const _hoisted_1 = /*#__PURE__*/ _createElementVNode("TresSpotLight");
function _sfc_render(_ctx, _cache, $props, $setup, $data, $options) {
  return _openBlock(), _createBlock($setup["TresCanvas"], null, {
    default: _withCtx(() => [$setup.visible ? (_openBlock(), _createElementBlock("TresMesh", { key: 0 }, [_hoisted_1])) : _createCommentVNode("v-if", true)]),
    _: 1
  });
}
import { TresCanvas } from "@tresjs/core";
const visible = true;
export default {
  __name: "TresCustomElements",
  render: _sfc_render,
  setup(__props, { expose: __expose }) {
    __expose();
    const __returned__ = {
      visible,
      TresCanvas
    };
    Object.defineProperty(__returned__, "__isScriptSetup", {
      enumerable: false,
      value: true
    });
    return __returned__;
  }
};`,
  "explicit setup imports and custom-element pattern lowering should match exactly",
);

// Control: without `customElements`, `customRenderer` alone keeps the same tags
// as components resolved at runtime.
const customRendererOnlyCompiled = compileFile(
  "/src/TresCustomElements.vue",
  new Map(),
  {
    sourceMap: false,
    ssr: false,
    vapor: false,
    customRenderer: true,
  },
  customElementPatternSource,
);

assert.match(
  customRendererOnlyCompiled.code,
  /_resolveComponent\("TresMesh"\)/,
  "Without customElements patterns, renderer tags stay runtime-resolved components",
);
