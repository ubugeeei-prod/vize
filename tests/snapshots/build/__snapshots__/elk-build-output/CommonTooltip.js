import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderSlot as _renderSlot, toDisplayString as _toDisplayString, mergeProps as _mergeProps, withCtx as _withCtx } from "vue"

import type { Popper as VTooltipType } from 'floating-vue'

export interface Props extends Partial<typeof VTooltipType> {
  content?: string
}

export default /*@__PURE__*/_defineComponent({
  __name: 'CommonTooltip',
  props: {
    content: { type: String, required: false }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  const _component_VTooltip = _resolveComponent("VTooltip")

  return (_ctx.isHydrated)
      ? (_openBlock(), _createBlock(_component_VTooltip, _mergeProps(_ctx.$attrs, {
        key: 0,
        "auto-hide": "",
        "no-auto-focus": ""
      }), {
        popper: _withCtx(() => [
          _createElementVNode("div", { "text-3": "" }, [
            _renderSlot(_ctx.$slots, "popper", {}, () => [
              _toDisplayString(__props.content)
            ])
          ])
        ]),
        default: _withCtx(() => [
          _renderSlot(_ctx.$slots, "default")
        ]),
        _: 1 /* STABLE */
      }, 16 /* FULL_PROPS */))
      : _createCommentVNode("v-if", true)
}
}

})
