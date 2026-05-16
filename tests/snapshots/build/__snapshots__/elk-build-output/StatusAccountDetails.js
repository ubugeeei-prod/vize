import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withCtx as _withCtx, unref as _unref } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusAccountDetails',
  props: {
    account: { type: null, required: true },
    link: { type: Boolean, required: false, default: true }
  },
  setup(__props: any) {

const userSettings = useUserSettings()

return (_ctx: any,_cache: any) => {
  const _component_AccountDisplayName = _resolveComponent("AccountDisplayName")
  const _component_AccountHandle = _resolveComponent("AccountHandle")
  const _component_AccountBotIndicator = _resolveComponent("AccountBotIndicator")
  const _component_AccountLockIndicator = _resolveComponent("AccountLockIndicator")
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (_openBlock(), _createBlock(_component_NuxtLink, {
      to: __props.link ? _ctx.getAccountRoute(__props.account) : undefined,
      flex: "~ col",
      "min-w-0": "",
      "md:flex": "~ row gap-2",
      "md:items-center": "",
      "text-link-rounded": ""
    }, {
      default: _withCtx(() => [
        _createVNode(_component_AccountDisplayName, {
          account: __props.account,
          "hide-emojis": _ctx.getPreferences(_unref(userSettings), 'hideUsernameEmojis'),
          "font-bold": "",
          "line-clamp-1": "",
          "ws-pre-wrap": "",
          "break-all": ""
        }, null, 8 /* PROPS */, ["account", "hide-emojis"]),
        _createElementVNode("div", { flex: "~ gap-1" }, [
          _createVNode(_component_AccountHandle, {
            account: __props.account,
            class: "zen-none"
          }, null, 8 /* PROPS */, ["account"]),
          (__props.account.bot)
            ? (_openBlock(), _createBlock(_component_AccountBotIndicator, {
              key: 0,
              "text-xs": ""
            }))
            : _createCommentVNode("v-if", true),
          (__props.account.locked)
            ? (_openBlock(), _createBlock(_component_AccountLockIndicator, {
              key: 0,
              "text-xs": ""
            }))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["to"]))
}
}

})
