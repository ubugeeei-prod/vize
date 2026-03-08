import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString } from "vue"

const _hoisted_1 = { flex: "~ gap-2", "items-center": "true" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { "i-ri-arrow-down-circle-line": "true", absolute: "true", "text-8em": "true", "bottom--10": "true", "inset-ie--10": "true", "text-primary": "true", "dark:text-white": "true", op10: "true", class: "-z-1" })

export function render(_ctx, _cache) {
  return (_ctx.useNuxtApp().$pwa?.needRefresh)
    ? (_openBlock(), _createElementBlock("div", {
      key: 0,
      "m-2": "",
      p5: "",
      bg: "primary-fade",
      relative: "",
      "rounded-lg": "",
      "of-hidden": "",
      flex: "~ col gap-3"
    }, [
      _createElementVNode("h2", _hoisted_1, _toDisplayString(_ctx.$t('pwa.title')), 1 /* TEXT */),
      _createElementVNode("div", { flex: "~ gap-1" }, [
        _createElementVNode("button", {
          type: "button",
          "btn-solid": "",
          "px-4": "",
          "py-1": "",
          "text-center": "",
          "text-sm": "",
          onClick: $event => (_ctx.useNuxtApp().$pwa?.updateServiceWorker())
        }, _toDisplayString(_ctx.$t('pwa.update')), 9 /* TEXT, PROPS */, ["onClick"]),
        _createElementVNode("button", {
          type: "button",
          "btn-text": "",
          "filter-saturate-0": "",
          "px-4": "",
          "py-1": "",
          "text-center": "",
          "text-sm": "",
          onClick: $event => (_ctx.useNuxtApp().$pwa?.close())
        }, _toDisplayString(_ctx.$t('pwa.dismiss')), 9 /* TEXT, PROPS */, ["onClick"])
      ]),
      _hoisted_2
    ]))
    : _createCommentVNode("v-if", true)
}
