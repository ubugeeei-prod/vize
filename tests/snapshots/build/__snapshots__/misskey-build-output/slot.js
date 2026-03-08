import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'slot',
  setup(__props) {

function focus() {
	// TODO
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.label),
        onClick: focus
      }, [ _renderSlot(_ctx.$slots, "label") ]), _createElementVNode("div", null, [ _renderSlot(_ctx.$slots, "default") ]), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.caption)
      }, [ _renderSlot(_ctx.$slots, "caption") ]) ]))
}
}

})
