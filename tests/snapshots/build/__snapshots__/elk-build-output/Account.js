import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'Account',
  props: {
    account: { type: null, required: true },
    hoverCard: { type: Boolean, required: false },
    list: { type: String, required: true }
  },
  setup(__props: any) {

cacheAccount(__props.account)
const client = useMastoClient()
const isRemoved = ref(false)
async function edit() {
  try {
    if (isRemoved.value)
      await client.v1.lists.$select(__props.list).accounts.create({ accountIds: [__props.account.id] })
    else
      await client.v1.lists.$select(__props.list).accounts.remove({ accountIds: [__props.account.id] })
    isRemoved.value = !isRemoved.value
  }
  catch (err) {
    console.error(err)
  }
}

return (_ctx: any,_cache: any) => {
  const _component_AccountInfo = _resolveComponent("AccountInfo")
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")

  return (_openBlock(), _createElementBlock("div", {
      flex: "",
      "justify-between": "",
      "hover:bg-active": "",
      "transition-100": "",
      "items-center": ""
    }, [ _createVNode(_component_AccountInfo, {
        account: __props.account,
        hover: "",
        p1: "",
        as: "router-link",
        "hover-card": __props.hoverCard,
        shrink: "",
        "overflow-hidden": "",
        to: _ctx.getAccountRoute(__props.account)
      }, null, 8 /* PROPS */, ["account", "hover-card", "to"]), _createElementVNode("div", null, [ _createVNode(_component_CommonTooltip, {
          content: isRemoved.value ? _ctx.$t('list.add_account') : _ctx.$t('list.remove_account'),
          hover: isRemoved.value ? 'text-green' : 'text-red'
        }, {
          default: _withCtx(() => [
            _createElementVNode("button", {
              "text-sm": "",
              p2: "",
              "border-1": "",
              "transition-colors": "",
              "border-dark": "",
              "bg-base": "",
              "btn-action-icon": "",
              onClick: edit
            }, [
              _createElementVNode("span", {
                class: _normalizeClass(isRemoved.value ? 'i-ri:user-add-line' : 'i-ri:user-unfollow-line')
              }, null, 2 /* CLASS */)
            ])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["content", "hover"]) ]) ]))
}
}

})
