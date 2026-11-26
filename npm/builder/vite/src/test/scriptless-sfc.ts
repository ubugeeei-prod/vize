import type { CompiledModule, StyleBlockInfo } from "../types.ts";
import type { GenerateOutputOptions } from "../utils/index.ts";

/**
 * Compiled SFCs without a `<script>` block, as the native compiler emits them,
 * with the options each production and SSR caller passes. The modules are
 * copies, so the pinned outputs only move when the plugin's assembly does.
 */
const filePath = "/app/components/Note.vue";
const root = "/app";

const templateOnlyClient: CompiledModule = {
  code: 'import { toDisplayString as _toDisplayString, openBlock as _openBlock, createElementBlock as _createElementBlock } from "vue"\n\nconst _hoisted_1 = { class: "a" }\n\nexport function render(_ctx, _cache) {\n  return (_openBlock(), _createElementBlock("p", _hoisted_1, "hi " + _toDisplayString(_ctx.msg), 1 /* TEXT */))\n}',
  scopeId: "da4a867b",
  hasScoped: false,
  styles: [],
};

const scopedStyle: StyleBlockInfo = {
  content: ".a{color:red}",
  src: null,
  lang: null,
  scoped: true,
  module: false,
  index: 0,
};

const scopedStyleClient: CompiledModule = {
  code: 'import { openBlock as _openBlock, createElementBlock as _createElementBlock } from "vue"\n\nconst _hoisted_1 = { class: "a" }\n\nexport function render(_ctx, _cache) {\n  return (_openBlock(), _createElementBlock("p", _hoisted_1, "hi"))\n}',
  scopeId: "da4a867b",
  hasScoped: true,
  css: ".a[data-v-da4a867b]{color:red}",
  styles: [scopedStyle],
};

const scopedStyleSsr: CompiledModule = {
  code: 'import { ssrRenderAttrs as _ssrRenderAttrs } from "@vue/server-renderer"\nimport { mergeProps as _mergeProps } from "vue"\n\nfunction ssrRender(_ctx, _push, _parent, _attrs) {\n  _push(`<p${_ssrRenderAttrs(_mergeProps({ class: "a" }, _attrs))} data-v-da4a867b>hi</p>`)\n}\n\nconst _sfc_main = {}\n_sfc_main.ssrRender = ssrRender\nexport default _sfc_main',
  scopeId: "da4a867b",
  hasScoped: true,
  css: ".a[data-v-da4a867b]{color:red}",
  styles: [scopedStyle],
};

/** A render-only SSR module, the shape the plugin shims with `{ ssrRender }`. */
const ssrRenderOnly: CompiledModule = {
  code: 'import { ssrRenderAttrs as _ssrRenderAttrs } from "@vue/server-renderer"\n\nexport function ssrRender(_ctx, _push, _parent, _attrs) {\n  _push(`<p${_ssrRenderAttrs(_attrs)}>hi</p>`)\n}',
  scopeId: "da4a867b",
  hasScoped: false,
  styles: [],
};

const production = { isProduction: true, isDev: false, filePath, root };
/** The SFC module loader compiles SSR requests with `isDev: false`. */
const ssr = { isProduction: false, isDev: false, ssr: true, filePath, root };

export const SCRIPTLESS_OUTPUT_CASES: Record<
  string,
  { compiled: CompiledModule; options: GenerateOutputOptions }
> = {
  "production template-only": { compiled: templateOnlyClient, options: production },
  "production scoped style, extracted CSS": {
    compiled: scopedStyleClient,
    options: { ...production, extractCss: true },
  },
  "production scoped style, inline CSS": {
    compiled: { ...scopedStyleClient, styles: [] },
    options: production,
  },
  "production SSR scoped style": {
    compiled: scopedStyleSsr,
    options: { ...production, ssr: true },
  },
  "dev SSR scoped style": { compiled: scopedStyleSsr, options: ssr },
  "dev SSR render-only shim": { compiled: ssrRenderOnly, options: ssr },
  "production SSR render-only shim": {
    compiled: ssrRenderOnly,
    options: { ...production, ssr: true },
  },
};
