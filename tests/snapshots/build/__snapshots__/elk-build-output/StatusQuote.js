import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusQuote',
  props: {
    status: { type: null, required: true },
    isNested: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {

function isQuoteType(quote: mastodon.v1.Status['quote']): quote is mastodon.v1.Quote | mastodon.v1.ShallowQuote {
  return !!quote
}
function isShallowQuoteType(quote: mastodon.v1.Quote | mastodon.v1.ShallowQuote): quote is mastodon.v1.ShallowQuote {
  return 'quotedStatusId' in quote
}
const quoteState = computed<mastodon.v1.QuoteState | null>(() => {
  if (!isQuoteType(__props.status.quote)) {
    return null
  }
  return __props.status.quote.state
})
const shallowQuotedStatus = ref<mastodon.v1.Status | null>(null)
watchEffect(async () => {
  if (!isQuoteType(__props.status.quote) || !isShallowQuoteType(__props.status.quote) || quoteState.value === 'deleted' || !__props.status.quote.quotedStatusId) {
    shallowQuotedStatus.value = null
    return
  }
  shallowQuotedStatus.value = await fetchStatus(__props.status.quote.quotedStatusId)
})
const quotedStatus = computed(() => {
  if (!isQuoteType(__props.status.quote)) {
    return null
  }
  if (isShallowQuoteType(__props.status.quote)) {
    if (!__props.status.quote.quotedStatusId) {
      return null
    }
    return shallowQuotedStatus.value
  }
  return __props.status.quote.quotedStatus
})

return (_ctx: any,_cache: any) => {
  const _component_AccountInlineInfo = _resolveComponent("AccountInlineInfo")
  const _component_StatusCard = _resolveComponent("StatusCard")

  return (quotedStatus.value)
      ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (__props.isNested && quoteState.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (quoteState.value === 'pending') ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                flex: "",
                border: "~ 1",
                "rounded-lg": "",
                "bg-card": "",
                "mt-3": "",
                "p-3": ""
              }, "\n        Post pending for approval by author\n      ")) : (quoteState.value === 'revoked') ? (_openBlock(), _createElementBlock("div", {
                  key: 1,
                  flex: "",
                  border: "~ 1",
                  "rounded-lg": "",
                  "bg-card": "",
                  "mt-3": "",
                  "p-3": ""
                }, "\n        Post removed by author\n      ")) : (quoteState.value === 'blocked_account') ? (_openBlock(), _createElementBlock("div", {
                  key: 2,
                  flex: "",
                  border: "~ 1",
                  "rounded-lg": "",
                  "bg-card": "",
                  "mt-3": "",
                  "p-3": ""
                }, "\n        Post by blocked author\n      ")) : (quoteState.value === 'blocked_domain') ? (_openBlock(), _createElementBlock("div", {
                  key: 3,
                  flex: "",
                  border: "~ 1",
                  "rounded-lg": "",
                  "bg-card": "",
                  "mt-3": "",
                  "p-3": ""
                }, "\n        Post from blocked server\n      ")) : (quoteState.value === 'muted_account') ? (_openBlock(), _createElementBlock("div", {
                  key: 4,
                  flex: "",
                  border: "~ 1",
                  "rounded-lg": "",
                  "bg-card": "",
                  "mt-3": "",
                  "p-3": ""
                }, "\n        Post by muted author\n      ")) : (quoteState.value === 'deleted' || quoteState.value === 'rejected' || quoteState.value === 'unauthorized') ? (_openBlock(), _createElementBlock("div", {
                  key: 5,
                  flex: "",
                  border: "~ 1",
                  "rounded-lg": "",
                  "bg-card": "",
                  "mt-3": "",
                  "p-3": ""
                }, "\n        Post is unavailable\n      ")) : (quoteState.value === 'accepted') ? (_openBlock(), _createElementBlock("div", {
                  key: 6,
                  flex: "",
                  border: "~ 1",
                  "rounded-lg": "",
                  "bg-card": "",
                  "mt-3": "",
                  "p-3": ""
                }, [ _createTextVNode("\n        Post by\n        "), _createVNode(_component_AccountInlineInfo, {
                    account: quotedStatus.value.account,
                    link: false,
                    "mx-1": ""
                  }, null, 8 /* PROPS */, ["account", "link"]) ])) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ (quoteState.value === 'pending') ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                flex: "",
                border: "~ 1",
                "rounded-lg": "",
                "bg-card": "",
                "mt-3": "",
                "p-3": ""
              }, "\n        Post pending for approval by author\n      ")) : (quoteState.value === 'revoked') ? (_openBlock(), _createElementBlock("div", {
                  key: 1,
                  flex: "",
                  border: "~ 1",
                  "rounded-lg": "",
                  "bg-card": "",
                  "mt-3": "",
                  "p-3": ""
                }, "\n        Post removed by author\n      ")) : (quoteState.value === 'deleted' || quoteState.value === 'rejected' || quoteState.value === 'unauthorized') ? (_openBlock(), _createElementBlock("div", {
                  key: 2,
                  flex: "",
                  border: "~ 1",
                  "rounded-lg": "",
                  "bg-card": "",
                  "mt-3": "",
                  "p-3": ""
                }, "\n        Post is unavailable\n      ")) : (quoteState.value === 'accepted') ? (_openBlock(), _createElementBlock("blockquote", {
                  key: 3,
                  cite: quotedStatus.value.uri
                }, [ _createVNode(_component_StatusCard, {
                    status: quotedStatus.value,
                    actions: false,
                    "is-nested": true,
                    border: "base 1",
                    "rounded-lg": "",
                    "hover:bg-active": "",
                    "my-3": ""
                  }, null, 8 /* PROPS */, ["status", "actions", "is-nested"]) ])) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */))
      : _createCommentVNode("v-if", true)
}
}

})
