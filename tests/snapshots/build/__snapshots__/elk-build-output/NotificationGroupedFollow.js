import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:arrow-down-s-line": "true", "mx-1": "true", "text-secondary": "true", "text-xl": "true", "aria-hidden": "true" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:arrow-up-s-line": "true", "ms-2": "true", "text-secondary": "true", "text-xl": "true", "aria-hidden": "true" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { "ps-2": "true", "text-base": "true" }, "Hide")
import type { GroupedNotifications } from '#shared/types'
const maxVisibleFollows = 5

export default /*@__PURE__*/_defineComponent({
  __name: 'NotificationGroupedFollow',
  props: {
    items: { type: null, required: true }
  },
  setup(__props: any) {

const follows = computed(() => __props.items.items)
const visibleFollows = computed(() => follows.value.slice(0, maxVisibleFollows))
const count = computed(() => follows.value.length)
const countPlus = computed(() => Math.max(count.value - maxVisibleFollows, 0))
const isExpanded = ref(false)
const lang = computed(() => {
  return (count.value > 1 || count.value === 0) ? undefined : __props.items.items[0].status?.language
})
const timeAgoOptions = useTimeAgoOptions(true)
const timeAgoCreatedAt = computed(() => follows.value[0].createdAt)
const timeAgo = useTimeAgo(() => timeAgoCreatedAt.value, timeAgoOptions)

return (_ctx: any,_cache: any) => {
  const _component_AccountDisplayName = _resolveComponent("AccountDisplayName")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_AccountHoverWrapper = _resolveComponent("AccountHoverWrapper")
  const _component_CommonLocalizedNumber = _resolveComponent("CommonLocalizedNumber")
  const _component_AccountAvatar = _resolveComponent("AccountAvatar")
  const _component_StatusAccountDetails = _resolveComponent("StatusAccountDetails")

  return (_openBlock(), _createElementBlock("article", {
      flex: "",
      "flex-col": "",
      relative: "",
      lang: lang.value ?? undefined
    }, [ _createElementVNode("div", {
        flex: "",
        "items-center": "",
        "top-0": "",
        "left-2": "",
        "pt-2": "",
        "px-3": ""
      }, [ _createElementVNode("div", {
          class: _normalizeClass(count.value > 1 ? 'i-ri-group-line' : 'i-ri-user-3-line'),
          "me-3": "",
          "color-blue": "",
          "text-xl": "",
          "aria-hidden": "true"
        }, null, 2 /* CLASS */), (count.value > 1) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createVNode(_component_AccountHoverWrapper, { account: follows.value[0].account }, {
              default: _withCtx(() => [
                _createVNode(_component_NuxtLink, { to: _ctx.getAccountRoute(follows.value[0].account) }, {
                  default: _withCtx(() => [
                    _createVNode(_component_AccountDisplayName, {
                      account: follows.value[0].account,
                      "text-primary": "",
                      "font-bold": "",
                      "line-clamp-1": "",
                      "ws-pre-wrap": "",
                      "break-all": "",
                      "hover:underline": ""
                    }, null, 8 /* PROPS */, ["account"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["to"])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["account"]), _createTextVNode("\n        &nbsp;"), _toDisplayString(_ctx.$t('notification.and')), _createTextVNode("&nbsp;\n        "), _createVNode(_component_CommonLocalizedNumber, {
              keypath: "notification.others",
              count: count.value - 1,
              "text-primary": "",
              "font-bold": "",
              "line-clamp-1": "",
              "ws-pre-wrap": "",
              "break-all": ""
            }, null, 8 /* PROPS */, ["count"]), _createTextVNode("\n        &nbsp;"), _toDisplayString(_ctx.$t('notification.followed_you')), _createTextVNode("\n        "), _createElementVNode("time", {
              "text-secondary": "",
              datetime: timeAgoCreatedAt.value
            }, "\n          ・" + _toDisplayString(_unref(timeAgo)), 9 /* TEXT, PROPS */, ["datetime"]) ], 64 /* STABLE_FRAGMENT */)) : (count.value === 1) ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ _createVNode(_component_NuxtLink, { to: _ctx.getAccountRoute(follows.value[0].account) }, {
                default: _withCtx(() => [
                  _createVNode(_component_AccountDisplayName, {
                    account: follows.value[0].account,
                    "text-primary": "",
                    "me-1": "",
                    "font-bold": "",
                    "line-clamp-1": "",
                    "ws-pre-wrap": "",
                    "break-all": "",
                    "hover:underline": ""
                  }, null, 8 /* PROPS */, ["account"])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to"]), _createElementVNode("span", {
                "me-1": "",
                "ws-nowrap": ""
              }, [ _createTextVNode(_toDisplayString(_ctx.$t('notification.followed_you')) + "\n          ", 1 /* TEXT */), _createElementVNode("time", {
                  "text-secondary": "",
                  datetime: timeAgoCreatedAt.value
                }, "\n            ・" + _toDisplayString(_unref(timeAgo)), 9 /* TEXT, PROPS */, ["datetime"]) ]) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", {
        "pb-2": "",
        ps8: ""
      }, [ (!isExpanded.value && count.value > 1) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            flex: "~ wrap gap-1.75",
            p4: "",
            "items-center": "",
            "cursor-pointer": "",
            onClick: _cache[0] || (_cache[0] = ($event: any) => (isExpanded.value = !isExpanded.value))
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(visibleFollows.value, (follow) => {
              return (_openBlock(), _createBlock(_component_AccountHoverWrapper, {
                key: follow.id,
                account: follow.account
              }, {
                default: _withCtx(() => [
                  _createVNode(_component_NuxtLink, { to: _ctx.getAccountRoute(follow.account) }, {
                    default: _withCtx(() => [
                      _createVNode(_component_AccountAvatar, {
                        account: follow.account,
                        "w-12": "",
                        "h-12": ""
                      }, null, 8 /* PROPS */, ["account"])
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 8 /* PROPS */, ["to"])
                ]),
                _: 2 /* DYNAMIC */
              }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["account"]))
            }), 128 /* KEYED_FRAGMENT */)), _createElementVNode("div", {
              flex: "~ 1",
              "items-center": ""
            }, [ (countPlus.value > 0) ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  "ps-2": "",
                  text: "base lg"
                }, "+" + _toDisplayString(countPlus.value), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _hoisted_1 ]) ])) : (_openBlock(), _createElementBlock("div", { key: 1 }, [ (count.value > 1) ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                flex: "",
                "p-4": "",
                "pb-2": "",
                "cursor-pointer": "",
                onClick: _cache[1] || (_cache[1] = ($event: any) => (isExpanded.value = !isExpanded.value))
              }, [ _hoisted_2, _hoisted_3 ])) : _createCommentVNode("v-if", true), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(follows.value, (follow) => {
              return (_openBlock(), _createBlock(_component_AccountHoverWrapper, {
                key: follow.id,
                account: follow.account
              }, {
                default: _withCtx(() => [
                  _createVNode(_component_NuxtLink, {
                    to: _ctx.getAccountRoute(follow.account),
                    flex: "",
                    "gap-4": "",
                    "px-4": "",
                    "py-2": ""
                  }, {
                    default: _withCtx(() => [
                      _createVNode(_component_AccountAvatar, {
                        account: follow.account,
                        "w-12": "",
                        "h-12": ""
                      }, null, 8 /* PROPS */, ["account"]),
                      _createVNode(_component_StatusAccountDetails, { account: follow.account }, null, 8 /* PROPS */, ["account"])
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 8 /* PROPS */, ["to"])
                ]),
                _: 2 /* DYNAMIC */
              }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["account"]))
            }), 128 /* KEYED_FRAGMENT */)) ])) ]) ], 8 /* PROPS */, ["lang"]))
}
}

})
