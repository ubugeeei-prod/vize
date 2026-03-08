import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, resolveComponent as _resolveComponent, renderSlot as _renderSlot, withCtx as _withCtx } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'Announce',
  props: {
    text: { type: String, required: true },
    position: { type: String, required: false },
    isVisible: { type: Boolean, required: true }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  const _component_TooltipBase = _resolveComponent("TooltipBase")

  return (_openBlock(), _createBlock(_component_TooltipBase, {
      text: __props.text,
      isVisible: __props.isVisible,
      position: __props.position,
      "tooltip-attr": { 'aria-live': 'polite' }
    }, {
      default: _withCtx(() => [
        _renderSlot(_ctx.$slots, "default")
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["text", "isVisible", "position", "tooltip-attr"]))
}
}

})
