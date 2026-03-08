import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, withCtx as _withCtx } from "vue"

import MkClickerGame from '@/components/MkClickerGame.vue'
import { definePage } from '@/page.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'clicker',
  setup(__props) {

definePage(() => ({
	title: '🍪👈',
	icon: 'ti ti-cookie',
}));

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, null, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 800px;"
        }, [
          _createVNode(MkClickerGame)
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
