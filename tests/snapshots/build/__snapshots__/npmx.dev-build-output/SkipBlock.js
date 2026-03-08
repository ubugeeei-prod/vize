import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, toDisplayString as _toDisplayString } from "vue"


const _hoisted_1 = { class: "h-2" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("td")
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:chevrons-up-down w-4 h-4" })
const _hoisted_4 = { class: "px-0 sticky inset-is-2 italic opacity-50" }
const _hoisted_5 = { class: "h-2" }

export default /*@__PURE__*/_defineComponent({
  __name: 'SkipBlock',
  props: {
    count: { type: Number, required: true },
    content: { type: String, required: false }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createElementVNode("tr", _hoisted_1), _createElementVNode("tr", { class: "h-10 font-mono bg-bg-muted text-fg-muted" }, [ _hoisted_2, _createElementVNode("td", { class: "opacity-50 select-none text-center" }, [ _hoisted_3 ]), _createElementVNode("td", null, [ _createElementVNode("span", _hoisted_4, _toDisplayString(__props.content || _ctx.$t('compare.lines_hidden', { count: __props.count })), 1 /* TEXT */) ]) ]), _createElementVNode("tr", _hoisted_5) ], 64 /* STABLE_FRAGMENT */))
}
}

})
