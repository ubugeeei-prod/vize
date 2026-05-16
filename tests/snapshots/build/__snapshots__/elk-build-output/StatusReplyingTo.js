import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri-discuss-line": "true", "text-blue": "true" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { "i-ri-chat-1-line": "true", "text-blue": "true" })
import type { mastodon } from 'masto'
import { fetchAccountById } from '~/composables/cache'

type WatcherType = [status?: mastodon.v1.Status, v?: boolean]

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusReplyingTo',
  props: {
    status: { type: null, required: true },
    isSelfReply: { type: Boolean, required: true }
  },
  setup(__props: any) {

const link = ref()
const targetIsVisible = ref(false)
const isSelf = computed(() => __props.status.inReplyToAccountId === __props.status.account.id)
const account = ref<mastodon.v1.Account | null | undefined>(isSelf.value ? __props.status.account : undefined)
useIntersectionObserver(
  link,
  ([{ intersectionRatio }]) => {
    targetIsVisible.value = intersectionRatio > 0.1
  },
)
watch(
  () => [__props.status, targetIsVisible.value] satisfies WatcherType,
  ([newStatus, newVisible]) => {
    if (newStatus.account && newStatus.inReplyToAccountId === newStatus.account.id) {
      account.value = newStatus.account
      return
    }
    if (!newVisible)
      return
    const newId = newStatus.inReplyToAccountId
    if (newId) {
      fetchAccountById(newStatus.inReplyToAccountId).then((acc) => {
        if (newId === __props.status.inReplyToAccountId)
          account.value = acc
      })
      return
    }
    account.value = undefined
  },
  { immediate: true, flush: 'post' },
)

return (_ctx: any,_cache: any) => {
  const _component_AccountInlineInfo = _resolveComponent("AccountInlineInfo")
  const _component_i18n_t = _resolveComponent("i18n-t")
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (__props.status.inReplyToId)
      ? (_openBlock(), _createBlock(_component_NuxtLink, {
        key: 0,
        ref: "link",
        flex: "~ gap2",
        "items-center": "",
        "h-auto": "",
        "text-sm": "",
        "text-secondary": "",
        to: _ctx.getStatusInReplyToRoute(__props.status),
        title: _ctx.$t('status.replying_to', [account.value ? _ctx.getDisplayName(account.value) : _ctx.$t('status.someone')]),
        "text-blue": "",
        "saturate-50": "",
        "hover:saturate-100": ""
      }, {
        default: _withCtx(() => [
          (__props.isSelfReply)
            ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
              _hoisted_1,
              _createElementVNode("span", null, _toDisplayString(_ctx.$t('status.show_full_thread')), 1 /* TEXT */)
            ], 64 /* STABLE_FRAGMENT */))
            : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
              _hoisted_2,
              _createElementVNode("div", {
                "ws-nowrap": "",
                flex: ""
              }, [
                _createVNode(_component_i18n_t, { keypath: "status.replying_to" }, {
                  default: _withCtx(() => [
                    (account.value)
                      ? (_openBlock(), _createBlock(_component_AccountInlineInfo, {
                        key: 0,
                        account: account.value,
                        link: false,
                        "m-inline-2": ""
                      }, null, 8 /* PROPS */, ["account", "link"]))
                      : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                        _toDisplayString(_ctx.$t('status.someone'))
                      ], 64 /* STABLE_FRAGMENT */))
                  ]),
                  _: 1 /* STABLE */
                })
              ])
            ], 64 /* STABLE_FRAGMENT */))
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["to", "title"]))
      : _createCommentVNode("v-if", true)
}
}

})
