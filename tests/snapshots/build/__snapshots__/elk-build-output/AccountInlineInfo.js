import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, mergeProps as _mergeProps, withCtx as _withCtx, unref as _unref } from "vue"

import type { mastodon } from 'masto'

const __default__ = {
  inheritAttrs: false,
}

export default /*@__PURE__*/Object.assign(__default__, {
  __name: 'AccountInlineInfo',
  props: {
    account: { type: null, required: true },
    link: { type: Boolean, required: false, default: true },
    avatar: { type: Boolean, required: false, default: true }
  },
  setup(__props: any) {

const userSettings = useUserSettings()

return (_ctx: any,_cache: any) => {
  const _component_AccountAvatar = _resolveComponent("AccountAvatar")
  const _component_AccountDisplayName = _resolveComponent("AccountDisplayName")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_AccountHoverWrapper = _resolveComponent("AccountHoverWrapper")

  return (_openBlock(), _createBlock(_component_AccountHoverWrapper, { account: __props.account }, {
      default: _withCtx(() => [
        _createVNode(_component_NuxtLink, _mergeProps(_ctx.$attrs, {
          to: __props.link ? _ctx.getAccountRoute(__props.account) : undefined,
          class: __props.link ? 'text-link-rounded -ml-1.5rem pl-1.5rem rtl-(ml0 pl-0.5rem -mr-1.5rem pr-1.5rem)' : '',
          "min-w-0": "",
          flex: "",
          "gap-2": "",
          "items-center": ""
        }), {
          default: _withCtx(() => [
            (__props.avatar)
              ? (_openBlock(), _createBlock(_component_AccountAvatar, {
                key: 0,
                account: __props.account,
                "w-5": "",
                "h-5": ""
              }, null, 8 /* PROPS */, ["account"]))
              : _createCommentVNode("v-if", true),
            _createVNode(_component_AccountDisplayName, {
              account: __props.account,
              "hide-emojis": _ctx.getPreferences(_unref(userSettings), 'hideUsernameEmojis'),
              "line-clamp-1": "",
              "ws-pre-wrap": "",
              "break-all": ""
            }, null, 8 /* PROPS */, ["account", "hide-emojis"])
          ]),
          _: 1 /* STABLE */
        }, 16 /* FULL_PROPS */, ["to"])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["account"]))
}
}

})
