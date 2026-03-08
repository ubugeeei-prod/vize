import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, resolveDynamicComponent as _resolveDynamicComponent, renderSlot as _renderSlot, withCtx as _withCtx } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'Group',
  props: {
    as: { type: [String, null], required: false }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(_resolveDynamicComponent(props.as || 'div'), { class: "flex items-center shrink-0 [&>*:not(:first-child)]:rounded-s-none [&>*:not(:first-child)]:border-s-0 [&>*:not(:last-child)]:rounded-e-none" }, {
      default: _withCtx(() => [
        _renderSlot(_ctx.$slots, "default")
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
