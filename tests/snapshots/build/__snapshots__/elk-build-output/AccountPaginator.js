import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, createSlots as _createSlots, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


const _hoisted_1 = { italic: "true" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:external-link-fill": "true" })
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'AccountPaginator',
  props: {
    paginator: { type: null, required: true },
    context: { type: String, required: false },
    account: { type: null, required: false },
    relationshipContext: { type: String, required: false }
  },
  setup(__props: any) {

const fallbackContext = computed(() => {
  return ['following', 'followers'].includes(__props.context!)
})
const showOriginSite = computed(() =>
  __props.account && __props.account.id !== currentUser.value?.account.id && getServerName(__props.account) !== currentServer.value,
)

return (_ctx: any,_cache: any) => {
  const _component_AccountCard = _resolveComponent("AccountCard")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_CommonPaginator = _resolveComponent("CommonPaginator")

  return (_openBlock(), _createBlock(_component_CommonPaginator, { paginator: __props.paginator }, _createSlots({ _: 2 /* DYNAMIC */ }, [ {
        name: "default",
        fn: _withCtx(({ item }) => [
          _createVNode(_component_AccountCard, {
            account: item,
            "relationship-context": __props.relationshipContext,
            "hover-card": "",
            border: "b base",
            py2: "",
            px4: ""
          }, null, 8 /* PROPS */, ["account", "relationship-context"])
        ])
      }, (fallbackContext.value && showOriginSite.value) ? {
          name: "done",
          fn: _withCtx(() => [
            _createElementVNode("div", {
              p5: "",
              "text-secondary": "",
              "text-center": "",
              flex: "",
              "flex-col": "",
              "items-center": "",
              gap1: ""
            }, [
              _createElementVNode("span", _hoisted_1, _toDisplayString(_ctx.$t(`account.view_other_${__props.context}`)), 1 /* TEXT */),
              _createVNode(_component_NuxtLink, {
                href: __props.account.url,
                target: "_blank",
                external: "",
                flex: "~ gap-1",
                "items-center": "",
                "text-primary": "",
                hover: "underline text-primary-active"
              }, {
                default: _withCtx(() => [
                  _hoisted_2,
                  _createTextVNode("\n          "),
                  _createTextVNode(_toDisplayString(_ctx.$t('menu.open_in_original_site')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["href"])
            ])
          ]),
          key: "0"
        } : undefined ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["paginator"]))
}
}

})
