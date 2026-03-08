import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue"

import { provide } from 'vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'split',
  props: {
    minWidth: { type: Number, required: false, default: 210 }
  },
  setup(__props: any) {

const props = __props
provide('splited', true);
const minWidth = props.minWidth + 'px';

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ _renderSlot(_ctx.$slots, "default") ]))
}
}

})
