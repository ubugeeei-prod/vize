import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, renderSlot as _renderSlot, toDisplayString as _toDisplayString, mergeProps as _mergeProps, normalizeClass as _normalizeClass } from "vue"

import type { HTMLAttributes } from 'vue'

export default /*@__PURE__*/Object.assign({
  inheritAttrs: false,
}, {
  __name: 'CopyToClipboardButton',
  props: {
    copied: { type: Boolean, required: true },
    copyText: { type: String, required: false },
    copiedText: { type: String, required: false },
    ariaLabelCopy: { type: String, required: false },
    ariaLabelCopied: { type: String, required: false },
    buttonAttrs: { type: null, required: false }
  },
  emits: ["click"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const buttonCopyText = computed(() => props.copyText || $t('common.copy'))
const buttonCopiedText = computed(() => props.copiedText || $t('common.copied'))
const buttonAriaLabelCopy = computed(() => props.ariaLabelCopy || $t('common.copy'))
const buttonAriaLabelCopied = computed(() => props.ariaLabelCopied || $t('common.copied'))
function handleClick() {
  emit('click')
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", _mergeProps(_ctx.$attrs, {
      class: "group relative"
    }), [ _renderSlot(_ctx.$slots, "default"), _createElementVNode("button", _mergeProps(__props.buttonAttrs, {
        type: "button",
        onClick: handleClick,
        class: ["absolute z-20 inset-is-0 top-full inline-flex items-center gap-1 px-2 py-1 rounded border text-xs font-mono whitespace-nowrap transition-all duration-150 opacity-0 -translate-y-1 pointer-events-none group-hover:opacity-100 group-hover:translate-y-0 group-hover:pointer-events-auto focus-visible:opacity-100 focus-visible:translate-y-0 focus-visible:pointer-events-auto", [
          _ctx.$style.copyButton,
          __props.copied ? 'text-accent bg-accent/10' : 'text-fg-muted bg-bg border-border',
        ]],
        "aria-label": __props.copied ? buttonAriaLabelCopied.value : buttonAriaLabelCopy.value
      }), [ _createElementVNode("span", {
          class: _normalizeClass(["w-3.5 h-3.5", __props.copied ? 'i-lucide:check' : 'i-lucide:copy']),
          "aria-hidden": "true"
        }, null, 2 /* CLASS */), _createTextVNode("\n      " + _toDisplayString(__props.copied ? buttonCopiedText.value : buttonCopyText.value), 1 /* TEXT */) ], 16 /* FULL_PROPS */, ["aria-label"]) ], 16 /* FULL_PROPS */))
}
}

})
