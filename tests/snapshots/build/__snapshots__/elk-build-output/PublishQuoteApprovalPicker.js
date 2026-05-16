import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, renderList as _renderList, renderSlot as _renderSlot, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:double-quotes-l": "true", "me--2": "true" })
import type { mastodon } from 'masto'
import { statusQuoteApprovalPolicies } from '~/composables/masto/icons'

export default /*@__PURE__*/_defineComponent({
  __name: 'PublishQuoteApprovalPicker',
  props: {
    editing: { type: Boolean, required: false },
    "modelValue": {
  required: true,
}
  },
  emits: ["update:modelValue"],
  setup(__props: any) {

const modelValue = _useModel(__props, "modelValue")
const currentQuoteApprovalPolicy = computed(() =>
  statusQuoteApprovalPolicies.find(v => v.value === modelValue.value) || statusQuoteApprovalPolicies[0],
)
function chooseQuoteApprovalPolicy(quoteApprovalPolicy: mastodon.rest.v1.QuoteApprovalPolicy) {
  modelValue.value = quoteApprovalPolicy
}

return (_ctx: any,_cache: any) => {
  const _component_CommonDropdownItem = _resolveComponent("CommonDropdownItem")
  const _component_CommonDropdown = _resolveComponent("CommonDropdown")
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")

  return (_openBlock(), _createElementBlock("div", {
      flex: "",
      "items-center": ""
    }, [ _hoisted_1, _createVNode(_component_CommonTooltip, {
        placement: "top",
        content: __props.editing ? _ctx.$t(`quote_approval_policy.${currentQuoteApprovalPolicy.value}`) : _ctx.$t('tooltip.change_quote_approval_policy')
      }, {
        default: _withCtx(() => [
          _createVNode(_component_CommonDropdown, { placement: "bottom" }, {
            popper: _withCtx(() => [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(statusQuoteApprovalPolicies), (quoteApprovalPolicy) => {
                return (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                  key: quoteApprovalPolicy.value,
                  icon: quoteApprovalPolicy.icon,
                  text: _ctx.$t(`quote_approval_policy.${quoteApprovalPolicy.value}`),
                  description: _ctx.$t(`quote_approval_policy.${quoteApprovalPolicy.value}_desc`),
                  checked: quoteApprovalPolicy.value === modelValue.value,
                  onClick: _cache[0] || (_cache[0] = ($event: any) => (chooseQuoteApprovalPolicy(quoteApprovalPolicy.value)))
                }, null, 8 /* PROPS */, ["icon", "text", "description", "checked"]))
              }), 128 /* KEYED_FRAGMENT */))
            ]),
            default: _withCtx(() => [
              _renderSlot(_ctx.$slots, "default", { "quote-approval-policy": currentQuoteApprovalPolicy.value })
            ]),
            _: 1 /* STABLE */
          })
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["content"]) ]))
}
}

})
