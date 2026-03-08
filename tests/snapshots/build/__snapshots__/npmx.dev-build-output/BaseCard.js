import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'BaseCard',
  props: {
    isExactMatch: { type: Boolean, required: false }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("article", {
      class: _normalizeClass(["group bg-bg-subtle border border-border rounded-lg p-4 sm:p-6 transition-[border-color,background-color] duration-200 hover:(border-border-hover bg-bg-muted) cursor-pointer relative focus-within:outline-none focus-within:ring-2 focus-within:ring-offset-bg focus-within:ring-offset-2 focus-within:ring-fg/50 focus-within:bg-bg-muted focus-within:border-border-hover", {
        'border-accent/30 contrast-more:border-accent/90 bg-accent/5': __props.isExactMatch,
      }])
    }, [ (__props.isExactMatch) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "absolute -inset-px rounded-lg bg-gradient-to-r from-accent/0 via-accent/0 to-accent/10 opacity-100 blur-sm -z-1 pointer-events-none motion-reduce:opacity-50",
          "aria-hidden": "true"
        })) : _createCommentVNode("v-if", true), _renderSlot(_ctx.$slots, "default") ], 2 /* CLASS */))
}
}

})
