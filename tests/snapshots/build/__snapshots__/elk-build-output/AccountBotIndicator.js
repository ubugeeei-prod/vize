import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-mdi:robot-outline": "true" })

export default /*@__PURE__*/_defineComponent({
  __name: 'AccountBotIndicator',
  props: {
    showLabel: { type: Boolean, required: false }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")

  return (_openBlock(), _createElementBlock("div", {
      flex: "~ gap1",
      "items-center": "",
      class: _normalizeClass({ 'border border-base rounded-md px-1': __props.showLabel }),
      "text-secondary-light": ""
    }, [ _renderSlot(_ctx.$slots, "prepend"), _createVNode(_component_CommonTooltip, {
        content: _ctx.$t('account.bot'),
        disabled: __props.showLabel
      }, {
        default: _withCtx(() => [
          _hoisted_1
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["content", "disabled"]), (__props.showLabel) ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_ctx.$t('account.bot')), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 2 /* CLASS */))
}
}

})
