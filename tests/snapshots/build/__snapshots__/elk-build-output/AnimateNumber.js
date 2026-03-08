import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'AnimateNumber',
  props: {
    increased: { type: Boolean, required: false }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      "of-hidden": "",
      h: "1.25rem"
    }, [ _createElementVNode("div", {
        flex: "~ col",
        "transition-transform": "",
        "duration-300": "",
        class: _normalizeClass(__props.increased ? 'translate-y--1/2' : 'translate-y-0')
      }, [ _renderSlot(_ctx.$slots, "default"), _renderSlot(_ctx.$slots, "next") ], 2 /* CLASS */) ]))
}
}

})
