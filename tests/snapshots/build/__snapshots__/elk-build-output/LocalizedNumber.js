import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, mergeProps as _mergeProps, withCtx as _withCtx } from "vue"


const _hoisted_1 = { "sr-only": "true" }

export default /*@__PURE__*/Object.assign({
  inheritAttrs: false,
}, {
  __name: 'LocalizedNumber',
  props: {
    count: { type: Number, required: true },
    keypath: { type: String, required: true }
  },
  setup(__props: any) {

const { formatHumanReadableNumber, formatNumber, forSR } = useHumanReadableNumber()
const useSR = computed(() => forSR(__props.count))
const rawNumber = computed(() => formatNumber(__props.count))
const humanReadableNumber = computed(() => formatHumanReadableNumber(__props.count))

return (_ctx: any,_cache: any) => {
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")
  const _component_i18n_t = _resolveComponent("i18n-t")

  return (_openBlock(), _createBlock(_component_i18n_t, {
      keypath: __props.keypath,
      plural: __props.count,
      tag: "span",
      class: "flex gap-x-1"
    }, {
      default: _withCtx(() => [
        (useSR.value)
          ? (_openBlock(), _createBlock(_component_CommonTooltip, {
            key: 0,
            content: rawNumber.value,
            placement: "bottom"
          }, {
            default: _withCtx(() => [
              _createElementVNode("span", _mergeProps(_ctx.$attrs, {
                "aria-hidden": "true"
              }), _toDisplayString(humanReadableNumber.value), 17 /* TEXT, FULL_PROPS */),
              _createElementVNode("span", _hoisted_1, _toDisplayString(rawNumber.value), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["content"]))
          : (_openBlock(), _createElementBlock("span", _mergeProps(_ctx.$attrs, { key: 1 }), _toDisplayString(humanReadableNumber.value), 1 /* TEXT */))
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["keypath", "plural"]))
}
}

})
