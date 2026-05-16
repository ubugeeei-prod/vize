import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { "i-ri-arrow-right-line": "true", "ml--1": "true", "text-secondary-light": "true" })

export default /*@__PURE__*/_defineComponent({
  __name: 'ContentMentionGroup',
  props: {
    replying: { type: Boolean, required: false }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("p", {
      flex: "~ gap-1 wrap",
      "items-center": "",
      "text-sm": "",
      class: _normalizeClass({ 'zen-none': !__props.replying })
    }, [ _hoisted_1, _renderSlot(_ctx.$slots, "default") ], 2 /* CLASS */))
}
}

})
