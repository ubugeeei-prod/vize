import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, resolveComponent as _resolveComponent, mergeProps as _mergeProps } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'AccountBigAvatar',
  props: {
    account: { type: null, required: true },
    square: { type: Boolean, required: false }
  },
  setup(__props: any) {

// Avatar with a background base achieving a 3px border to be used in status cards
// The border is used for Avatar on Avatar for reblogs and connecting replies

return (_ctx: any,_cache: any) => {
  const _component_AccountAvatar = _resolveComponent("AccountAvatar")

  return (_openBlock(), _createElementBlock("div", _mergeProps(_ctx.$attrs, {
      key: __props.account.avatar,
      style: { 'clip-path': __props.square ? `url(#avatar-mask)` : 'none' },
      class: { 'rounded-full': !__props.square },
      "bg-base": "",
      "w-54px": "",
      "h-54px": "",
      flex: "",
      "items-center": "",
      "justify-center": ""
    }), [ _createVNode(_component_AccountAvatar, {
        account: __props.account,
        "w-48px": "",
        "h-48px": "",
        square: __props.square
      }, null, 8 /* PROPS */, ["account", "square"]) ], 16 /* FULL_PROPS */))
}
}

})
