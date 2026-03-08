import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue"

import MkAnimBg from '@/components/MkAnimBg.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'PageWithAnimBg',
  setup(__props) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", null, [ _createVNode(MkAnimBg, { style: "position: absolute;" }), _createElementVNode("div", {
        class: _normalizeClass(["_pageScrollable", _ctx.$style.body])
      }, [ _renderSlot(_ctx.$slots, "default") ]) ]))
}
}

})
