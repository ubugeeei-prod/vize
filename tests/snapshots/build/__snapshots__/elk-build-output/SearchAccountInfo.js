import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'SearchAccountInfo',
  props: {
    account: { type: null, required: true }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  const _component_AccountAvatar = _resolveComponent("AccountAvatar")
  const _component_AccountDisplayName = _resolveComponent("AccountDisplayName")
  const _component_AccountLockIndicator = _resolveComponent("AccountLockIndicator")
  const _component_AccountBotIndicator = _resolveComponent("AccountBotIndicator")
  const _component_AccountHandle = _resolveComponent("AccountHandle")

  return (_openBlock(), _createElementBlock("div", {
      flex: "",
      "gap-2": "",
      "items-center": ""
    }, [ _createVNode(_component_AccountAvatar, {
        "w-10": "",
        "h-10": "",
        account: __props.account,
        "shrink-0": ""
      }, null, 8 /* PROPS */, ["account"]), _createElementVNode("div", {
        flex: "~ col gap1",
        shrink: "",
        "h-full": "",
        "overflow-hidden": "",
        "leading-none": ""
      }, [ _createElementVNode("div", {
          flex: "~",
          "gap-2": ""
        }, [ _createVNode(_component_AccountDisplayName, {
            account: __props.account,
            "line-clamp-1": "",
            "ws-pre-wrap": "",
            "break-all": "",
            "text-base": ""
          }, null, 8 /* PROPS */, ["account"]), (__props.account.locked) ? (_openBlock(), _createBlock(_component_AccountLockIndicator, {
              key: 0,
              "text-xs": ""
            })) : _createCommentVNode("v-if", true), (__props.account.bot) ? (_openBlock(), _createBlock(_component_AccountBotIndicator, {
              key: 0,
              "text-xs": ""
            })) : _createCommentVNode("v-if", true) ]), _createVNode(_component_AccountHandle, {
          "text-sm": "",
          account: __props.account,
          "text-secondary-light": ""
        }, null, 8 /* PROPS */, ["account"]) ]) ]))
}
}

})
