import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, resolveComponent as _resolveComponent, renderSlot as _renderSlot, createSlots as _createSlots, withCtx as _withCtx, unref as _unref } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'App',
  props: {
    text: { type: String, required: false },
    position: { type: String, required: false },
    interactive: { type: Boolean, required: false },
    to: { type: [String, null], required: false },
    defer: { type: Boolean, required: false },
    offset: { type: Number, required: false }
  },
  setup(__props: any) {

const props = __props
const isVisible = shallowRef(false)
const tooltipId = useId()
const hideTimeout = shallowRef<ReturnType<typeof setTimeout> | null>(null)
function show() {
  if (hideTimeout.value) {
    clearTimeout(hideTimeout.value)
    hideTimeout.value = null
  }
  isVisible.value = true
}
function hide() {
  if (props.interactive) {
    // Delay hide so cursor can travel from trigger to tooltip
    hideTimeout.value = setTimeout(() => {
      isVisible.value = false
    }, 150)
  } else {
    isVisible.value = false
  }
}
const tooltipAttrs = computed(() => {
  const attrs: Record<string, unknown> = { role: 'tooltip', id: tooltipId }
  if (props.interactive) {
    attrs.onMouseenter = show
    attrs.onMouseleave = hide
  }
  return attrs
})

return (_ctx: any,_cache: any) => {
  const _component_TooltipBase = _resolveComponent("TooltipBase")

  return (_openBlock(), _createBlock(_component_TooltipBase, {
      text: __props.text,
      isVisible: isVisible.value,
      position: __props.position,
      interactive: __props.interactive,
      to: __props.to,
      defer: __props.defer,
      offset: __props.offset,
      "tooltip-attr": tooltipAttrs.value,
      onMouseenter: show,
      onMouseleave: hide,
      onFocusin: show,
      onFocusout: hide,
      "aria-describedby": isVisible.value ? _unref(tooltipId) : undefined
    }, _createSlots({ _: 2 /* DYNAMIC */ }, [ (_ctx.$slots.content) ? {
          name: "content",
          fn: _withCtx(() => [
            _renderSlot(_ctx.$slots, "content")
          ]),
          key: "0"
        } : undefined ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["text", "isVisible", "position", "interactive", "to", "defer", "offset", "tooltip-attr", "aria-describedby"]))
}
}

})
