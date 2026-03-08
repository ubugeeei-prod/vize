import { defineComponent as _defineComponent } from 'vue'
import { Teleport as _Teleport, Transition as _Transition, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, renderSlot as _renderSlot, toDisplayString as _toDisplayString, mergeProps as _mergeProps, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"

import type { HTMLAttributes } from 'vue'
import type { Placement, Strategy } from '@floating-ui/vue'
import { autoUpdate, flip, offset, shift, useFloating } from '@floating-ui/vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'Base',
  props: {
    text: { type: String, required: false },
    position: { type: String, required: false },
    isVisible: { type: Boolean, required: true },
    interactive: { type: Boolean, required: false },
    tooltipAttr: { type: null, required: false },
    to: { type: [String, null], required: false, default: 'body' },
    defer: { type: Boolean, required: false },
    offset: { type: Number, required: false, default: 4 },
    strategy: { type: null, required: false, default: 'absolute' }
  },
  setup(__props: any) {

const props = __props
const triggerRef = useTemplateRef('triggerRef')
const tooltipRef = useTemplateRef('tooltipRef')
const placement = computed<Placement>(() => props.position || 'bottom')
const { floatingStyles } = useFloating(triggerRef, tooltipRef, {
  placement,
  whileElementsMounted: autoUpdate,
  strategy: props.strategy,
  middleware: [offset(props.offset), flip(), shift({ padding: 8 })],
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      ref_key: "triggerRef", ref: triggerRef,
      class: "inline-flex"
    }, [ _renderSlot(_ctx.$slots, "default"), _createVNode(_Teleport, {
        to: props.to,
        defer: __props.defer
      }, [ _createVNode(_Transition, {
          "enter-active-class": "transition-opacity duration-150 motion-reduce:transition-none",
          "leave-active-class": "transition-opacity duration-100 motion-reduce:transition-none",
          "enter-from-class": "opacity-0",
          "leave-to-class": "opacity-0"
        }, {
          default: _withCtx(() => [
            (props.isVisible)
              ? (_openBlock(), _createElementBlock("div", _mergeProps(__props.tooltipAttr, {
                key: 0,
                ref: "tooltipRef",
                class: _normalizeClass(["px-2 py-1 font-mono text-xs text-fg bg-bg-elevated border border-border rounded shadow-lg whitespace-pre-line break-words max-w-xs z-[100]", { 'pointer-events-none': !__props.interactive }]),
                style: _normalizeStyle(_unref(floatingStyles))
              }), [
                _renderSlot(_ctx.$slots, "content", {}, () => [
                  _toDisplayString(__props.text)
                ])
              ]))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }) ], 8 /* PROPS */, ["to", "defer"]) ], 512 /* NEED_PATCH */))
}
}

})
