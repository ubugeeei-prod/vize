import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "flex-auto": "true" })
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'AccountMoved',
  props: {
    account: { type: null, required: true }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  const _component_AccountInlineInfo = _resolveComponent("AccountInlineInfo")
  const _component_AccountInfo = _resolveComponent("AccountInfo")
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (_openBlock(), _createElementBlock("div", {
      flex: "~ col gap-2",
      p4: ""
    }, [ _createElementVNode("div", {
        flex: "~ gap-1",
        "justify-center": ""
      }, [ _createVNode(_component_AccountInlineInfo, {
          account: __props.account,
          link: false
        }, null, 8 /* PROPS */, ["account", "link"]), _createTextVNode("\n      " + _toDisplayString(_ctx.$t('account.moved_title')), 1 /* TEXT */) ]), _createElementVNode("div", { flex: "" }, [ _createVNode(_component_NuxtLink, { to: _ctx.getAccountRoute(__props.account.moved) }, {
          default: _withCtx(() => [
            _createVNode(_component_AccountInfo, { account: __props.account.moved }, null, 8 /* PROPS */, ["account"])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["to"]), _hoisted_1, _createElementVNode("div", {
          flex: "",
          "items-center": ""
        }, [ _createVNode(_component_NuxtLink, {
            to: _ctx.getAccountRoute(__props.account.moved),
            "btn-solid": "",
            "inline-block": "",
            "h-fit": ""
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_ctx.$t('account.go_to_profile')), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["to"]) ]) ]) ]))
}
}

})
