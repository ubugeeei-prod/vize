import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'AccountHoverCard',
  props: {
    account: { type: null, required: true }
  },
  setup(__props: any) {

const relationship = useRelationship(__props.account)

return (_ctx: any,_cache: any) => {
  const _component_AccountInfo = _resolveComponent("AccountInfo")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_AccountFollowButton = _resolveComponent("AccountFollowButton")
  const _component_ContentRich = _resolveComponent("ContentRich")
  const _component_AccountPostsFollowers = _resolveComponent("AccountPostsFollowers")

  return _withDirectives((_openBlock(), _createElementBlock("div", {
      flex: "~ col gap2",
      rounded: "",
      "min-w-90": "",
      "max-w-120": "",
      "z-100": "",
      "overflow-hidden": "",
      "p-4": ""
    }, [ _createElementVNode("div", {
        flex: "~ gap2",
        "items-center": ""
      }, [ _createVNode(_component_NuxtLink, {
          to: _ctx.getAccountRoute(__props.account),
          "flex-auto": "",
          "rounded-full": "",
          "hover:bg-active": "",
          "transition-100": "",
          pe5: "",
          "me-a": ""
        }, {
          default: _withCtx(() => [
            _createVNode(_component_AccountInfo, {
              account: __props.account,
              "hover-card": false
            }, null, 8 /* PROPS */, ["account", "hover-card"])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["to"]), _createVNode(_component_AccountFollowButton, {
          "text-sm": "",
          account: __props.account,
          relationship: _unref(relationship)
        }, null, 8 /* PROPS */, ["account", "relationship"]) ]), (__props.account.note) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          "max-h-100": "",
          "overflow-y-auto": ""
        }, [ _createVNode(_component_ContentRich, {
            "text-4": "",
            "text-secondary": "",
            content: __props.account.note,
            emojis: __props.account.emojis
          }, null, 8 /* PROPS */, ["content", "emojis"]) ])) : _createCommentVNode("v-if", true), _createVNode(_component_AccountPostsFollowers, {
        "text-sm": "",
        account: __props.account,
        "is-hover-card": true
      }, null, 8 /* PROPS */, ["account", "is-hover-card"]) ], 512 /* NEED_PATCH */)), [ [_vShow, _unref(relationship)] ])
}
}

})
