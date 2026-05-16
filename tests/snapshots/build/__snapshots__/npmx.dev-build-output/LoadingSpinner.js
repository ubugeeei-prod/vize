import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "w-4 h-4 border-2 border-fg-subtle border-t-fg rounded-full motion-safe:animate-spin" })

export default /*@__PURE__*/_defineComponent({
  __name: 'LoadingSpinner',
  props: {
    text: { type: String, required: false }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      "aria-busy": "true",
      class: "flex items-center gap-3 text-fg-muted font-mono text-sm py-8"
    }, [ _hoisted_1, _createTextVNode("\n    " + _toDisplayString(__props.text ?? _ctx.$t('common.loading')), 1 /* TEXT */) ]))
}
}

})
