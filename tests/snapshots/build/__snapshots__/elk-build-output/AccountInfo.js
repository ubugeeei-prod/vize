import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, mergeProps as _mergeProps, withCtx as _withCtx } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/Object.assign({
  inheritAttrs: false,
}, {
  __name: 'AccountInfo',
  props: {
    account: { type: null, required: true },
    as: { type: String, required: false, default: 'div' },
    hoverCard: { type: Boolean, required: false },
    square: { type: Boolean, required: false }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  const _component_AccountBigAvatar = _resolveComponent("AccountBigAvatar")
  const _component_AccountHoverWrapper = _resolveComponent("AccountHoverWrapper")
  const _component_AccountDisplayName = _resolveComponent("AccountDisplayName")
  const _component_AccountLockIndicator = _resolveComponent("AccountLockIndicator")
  const _component_AccountBotIndicator = _resolveComponent("AccountBotIndicator")
  const _component_AccountHandle = _resolveComponent("AccountHandle")
  const _component_AccountRolesIndicator = _resolveComponent("AccountRolesIndicator")

  return (_openBlock(), _createBlock(_resolveDynamicComponent(__props.as), _mergeProps(_ctx.$attrs, {
      flex: "",
      "items-center": "",
      "gap-3": ""
    }), {
      default: _withCtx(() => [
        _createVNode(_component_AccountHoverWrapper, {
          disabled: !__props.hoverCard,
          account: __props.account
        }, {
          default: _withCtx(() => [
            _createVNode(_component_AccountBigAvatar, {
              account: __props.account,
              "shrink-0": "",
              square: __props.square
            }, null, 8 /* PROPS */, ["account", "square"])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["disabled", "account"]),
        _createElementVNode("div", {
          flex: "~ col",
          shrink: "",
          "h-full": "",
          "overflow-hidden": "",
          "justify-center": "",
          "leading-none": "",
          "select-none": "",
          "p-1": ""
        }, [
          _createElementVNode("div", {
            flex: "~",
            "gap-2": ""
          }, [
            _createVNode(_component_AccountDisplayName, {
              account: __props.account,
              "font-bold": "",
              "line-clamp-1": "",
              "ws-pre-wrap": "",
              "break-all": "",
              "text-lg": ""
            }, null, 8 /* PROPS */, ["account"]),
            (__props.account.locked)
              ? (_openBlock(), _createBlock(_component_AccountLockIndicator, {
                key: 0,
                "text-xs": ""
              }))
              : _createCommentVNode("v-if", true),
            (__props.account.bot)
              ? (_openBlock(), _createBlock(_component_AccountBotIndicator, {
                key: 0,
                "text-xs": ""
              }))
              : _createCommentVNode("v-if", true)
          ]),
          _createVNode(_component_AccountHandle, {
            account: __props.account,
            "text-secondary-light": ""
          }, null, 8 /* PROPS */, ["account"]),
          _createElementVNode("div", {
            "self-start": "",
            "mt-1": ""
          }, [
            (__props.account.roles?.length)
              ? (_openBlock(), _createBlock(_component_AccountRolesIndicator, {
                key: 0,
                account: __props.account,
                limit: 1
              }, null, 8 /* PROPS */, ["account", "limit"]))
              : _createCommentVNode("v-if", true)
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 16 /* FULL_PROPS */))
}
}

})
