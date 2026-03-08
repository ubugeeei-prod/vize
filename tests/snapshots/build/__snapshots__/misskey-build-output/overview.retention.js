import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, normalizeClass as _normalizeClass } from "vue"

import MkRetentionHeatmap from '@/components/MkRetentionHeatmap.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'overview.retention',
  setup(__props) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["_panel", _ctx.$style.root])
    }, [ _createVNode(MkRetentionHeatmap) ]))
}
}

})
