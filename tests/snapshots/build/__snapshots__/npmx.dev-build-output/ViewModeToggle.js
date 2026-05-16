import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:rows-2 w-4 h-4", "aria-hidden": "true" })
const _hoisted_2 = { class: "sr-only" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:table w-4 h-4", "aria-hidden": "true" })
const _hoisted_4 = { class: "sr-only" }
import type { ViewMode } from '#shared/types/preferences'

export default /*@__PURE__*/_defineComponent({
  __name: 'ViewModeToggle',
  props: {
    "modelValue": { default: 'cards' },
    "modelModifiers": {},
  },
  emits: ["update:modelValue"],
  setup(__props) {

const viewMode = _useModel(__props, "modelValue")

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: "inline-flex rounded-md border border-border p-0.5 bg-bg-muted",
      role: "group",
      "aria-label": _ctx.$t('filters.view_mode.label')
    }, [ _createElementVNode("button", {
        type: "button",
        class: _normalizeClass(["inline-flex items-center px-2.5 py-1.5 text-sm font-medium rounded-sm border transition-colors duration-200 focus-visible:ring-2 focus-visible:ring-fg focus-visible:ring-offset-1", 
          viewMode.value === 'cards'
            ? 'bg-bg-subtle text-fg border-fg-subtle'
            : 'text-fg-muted hover:text-fg border-transparent'
        ]),
        "aria-pressed": viewMode.value === 'cards',
        "aria-label": _ctx.$t('filters.view_mode.cards'),
        onClick: _cache[0] || (_cache[0] = ($event: any) => (viewMode.value = 'cards'))
      }, [ _hoisted_1, _createElementVNode("span", _hoisted_2, _toDisplayString(_ctx.$t('filters.view_mode.cards')), 1 /* TEXT */) ], 10 /* CLASS, PROPS */, ["aria-pressed", "aria-label"]), _createElementVNode("button", {
        type: "button",
        class: _normalizeClass(["inline-flex items-center px-2.5 py-1.5 text-sm font-medium rounded-sm border transition-colors duration-200 focus-visible:ring-2 focus-visible:ring-fg focus-visible:ring-offset-1", 
          viewMode.value === 'table'
            ? 'bg-bg-subtle  text-fg border-fg-subtle'
            : 'text-fg-muted hover:text-fg border-transparent'
        ]),
        "aria-pressed": viewMode.value === 'table',
        "aria-label": _ctx.$t('filters.view_mode.table'),
        onClick: _cache[1] || (_cache[1] = ($event: any) => (viewMode.value = 'table'))
      }, [ _hoisted_3, _createElementVNode("span", _hoisted_4, _toDisplayString(_ctx.$t('filters.view_mode.table')), 1 /* TEXT */) ], 10 /* CLASS, PROPS */, ["aria-pressed", "aria-label"]) ], 8 /* PROPS */, ["aria-label"]))
}
}

})
