import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri-user-3-line": "true", "text-xl": "true", "me-3": "true", "color-blue": "true" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:user-add-line": "true", "text-xl": "true", "me-2": "true", "color-purple": "true" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:flag-line": "true", "text-xl": "true", "me-2": "true", "color-purple": "true" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("div", { "i-ri-user-shared-line": "true", "text-xl": "true", "me-3": "true", "color-blue": "true" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:edit-2-fill": "true", "text-xl": "true", "me-1": "true", "text-secondary": "true" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("div", { "i-mdi:party-popper": "true", "text-xl": "true", "me-4": "true", "color-purple": "true" })
import type { mastodon } from 'masto'

type NotificationType = mastodon.v1.Notification['type'] | 'annual_report'
type Notification = Omit<mastodon.v1.Notification, 'type'> & { type: NotificationType }

export default /*@__PURE__*/_defineComponent({
  __name: 'NotificationCard',
  props: {
    notification: { type: Object, required: true }
  },
  setup(__props: any) {

// Add undocumented 'annual_report' type introduced in v4.3
// ref. https://github.com/mastodon/documentation/issues/1211#:~:text=api/v1/annual_reports
const { t } = useI18n()
// list of notification types Elk currently implemented
// type 'favourite' and 'reblog' should always rendered by NotificationGroupedLikes
const supportedNotificationTypes: NotificationType[] = [
  'follow',
  'admin.sign_up',
  'admin.report',
  'follow_request',
  'update',
  'mention',
  'poll',
  'update',
  'status',
  'annual_report',
  'quote',
]
// well-known emoji reactions types Elk does not support yet
const unsupportedEmojiReactionTypes = ['pleroma:emoji_reaction', 'reaction']
if (unsupportedEmojiReactionTypes.includes(__props.notification.type) || !supportedNotificationTypes.includes(__props.notification.type)) {
  console.warn(`[DEV] ${t('notification.missing_type')} '${__props.notification.type}' (notification.id: ${__props.notification.id})`)
}
const timeAgoOptions = useTimeAgoOptions(true)
const timeAgo = useTimeAgo(() => __props.notification.createdAt, timeAgoOptions)

return (_ctx: any,_cache: any) => {
  const _component_AccountDisplayName = _resolveComponent("AccountDisplayName")
  const _component_AccountBigCard = _resolveComponent("AccountBigCard")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_i18n_t = _resolveComponent("i18n-t")
  const _component_AccountFollowRequestButton = _resolveComponent("AccountFollowRequestButton")
  const _component_AccountCard = _resolveComponent("AccountCard")
  const _component_AccountInlineInfo = _resolveComponent("AccountInlineInfo")
  const _component_StatusCard = _resolveComponent("StatusCard")

  return (_openBlock(), _createElementBlock("article", {
      flex: "",
      "flex-col": "",
      relative: ""
    }, [ (__props.notification.type === 'follow') ? (_openBlock(), _createBlock(_component_NuxtLink, {
          key: 0,
          to: _ctx.getAccountRoute(__props.notification.account)
        }, {
          default: _withCtx(() => [
            _createElementVNode("div", {
              flex: "",
              "items-center": "",
              absolute: "",
              "ps-3": "",
              "pe-4": "",
              "inset-is-0": "",
              "rounded-ie-be-3": "",
              "py-3": "",
              "bg-base": "",
              "top-0": ""
            }, [
              _hoisted_1,
              _createVNode(_component_AccountDisplayName, {
                account: __props.notification.account,
                "text-primary": "",
                "me-1": "",
                "font-bold": "",
                "line-clamp-1": "",
                "ws-pre-wrap": "",
                "break-all": ""
              }, null, 8 /* PROPS */, ["account"]),
              _createElementVNode("span", { "ws-nowrap": "" }, [
                _createTextVNode(_toDisplayString(_ctx.$t('notification.followed_you')) + "\n            ", 1 /* TEXT */),
                _createElementVNode("time", {
                  "text-secondary": "",
                  datetime: __props.notification.createdAt
                }, "\n              ・" + _toDisplayString(_unref(timeAgo)), 9 /* TEXT, PROPS */, ["datetime"])
              ])
            ]),
            _createVNode(_component_AccountBigCard, {
              ms10: "",
              account: __props.notification.account
            }, null, 8 /* PROPS */, ["account"])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["to"])) : (__props.notification.type === 'admin.sign_up') ? (_openBlock(), _createBlock(_component_NuxtLink, {
            key: 1,
            to: _ctx.getAccountRoute(__props.notification.account)
          }, {
            default: _withCtx(() => [
              _createElementVNode("div", {
                flex: "",
                p4: "",
                "items-center": "",
                "bg-shaded": ""
              }, [
                _hoisted_2,
                _createVNode(_component_AccountDisplayName, {
                  account: __props.notification.account,
                  "text-purple": "",
                  "me-1": "",
                  "font-bold": "",
                  "line-clamp-1": "",
                  "ws-pre-wrap": "",
                  "break-all": ""
                }, null, 8 /* PROPS */, ["account"]),
                _createElementVNode("span", null, [
                  _createTextVNode(_toDisplayString(_ctx.$t("notification.signed_up")) + "\n            ", 1 /* TEXT */),
                  _createElementVNode("time", {
                    "text-secondary": "",
                    datetime: __props.notification.createdAt
                  }, "\n              ・" + _toDisplayString(_unref(timeAgo)), 9 /* TEXT, PROPS */, ["datetime"])
                ])
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["to"])) : (__props.notification.type === 'admin.report') ? (_openBlock(), _createBlock(_component_NuxtLink, {
            key: 2,
            to: _ctx.getReportRoute(__props.notification.report?.id)
          }, {
            default: _withCtx(() => [
              _createElementVNode("div", {
                flex: "",
                p4: "",
                "items-center": "",
                "bg-shaded": ""
              }, [
                _hoisted_3,
                _createVNode(_component_i18n_t, { keypath: "notification.reported" }, {
                  default: _withCtx(() => [
                    _createVNode(_component_AccountDisplayName, {
                      account: __props.notification.account,
                      "text-purple": "",
                      "me-1": "",
                      "font-bold": "",
                      "line-clamp-1": "",
                      "ws-pre-wrap": "",
                      "break-all": ""
                    }, null, 8 /* PROPS */, ["account"]),
                    _createVNode(_component_AccountDisplayName, {
                      account: __props.notification.report?.targetAccount,
                      "text-purple": "",
                      "ms-1": "",
                      "font-bold": "",
                      "line-clamp-1": "",
                      "ws-pre-wrap": "",
                      "break-all": ""
                    }, null, 8 /* PROPS */, ["account"])
                  ]),
                  _: 1 /* STABLE */
                })
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["to"])) : (__props.notification.type === 'follow_request') ? (_openBlock(), _createElementBlock(_Fragment, { key: 3 }, [ _createElementVNode("div", {
              flex: "",
              "px-3": "",
              "py-2": ""
            }, [ _hoisted_4, _createVNode(_component_AccountDisplayName, {
                account: __props.notification.account,
                "text-primary": "",
                "me-1": "",
                "font-bold": "",
                "line-clamp-1": "",
                "ws-pre-wrap": "",
                "break-all": ""
              }, null, 8 /* PROPS */, ["account"]), _createElementVNode("span", {
                "me-1": "",
                "ws-nowrap": ""
              }, [ _createTextVNode(_toDisplayString(_ctx.$t('notification.request_to_follow')) + "\n          ", 1 /* TEXT */), _createElementVNode("time", {
                  "text-secondary": "",
                  datetime: __props.notification.createdAt
                }, "\n            ・" + _toDisplayString(_unref(timeAgo)), 9 /* TEXT, PROPS */, ["datetime"]) ]) ]), _createVNode(_component_AccountCard, {
              p: "s-2 e-4 b-2",
              "hover-card": "",
              account: __props.notification.account
            }, {
              default: _withCtx(() => [
                _createVNode(_component_AccountFollowRequestButton, { account: __props.notification.account }, null, 8 /* PROPS */, ["account"])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["account"]) ], 64 /* STABLE_FRAGMENT */)) : (__props.notification.type === 'update') ? (_openBlock(), _createBlock(_component_StatusCard, {
            key: 4,
            status: __props.notification.status,
            "in-notification": true,
            actions: false
          }, {
            meta: _withCtx(() => [
              _createElementVNode("div", {
                flex: "~",
                "gap-1": "",
                "items-center": "",
                mt1: ""
              }, [
                _hoisted_5,
                _createVNode(_component_AccountInlineInfo, {
                  account: __props.notification.account,
                  me1: ""
                }, null, 8 /* PROPS */, ["account"]),
                _createElementVNode("span", { "ws-nowrap": "" }, [
                  _createTextVNode(_toDisplayString(_ctx.$t('notification.update_status')) + "\n              ", 1 /* TEXT */),
                  _createElementVNode("time", {
                    "text-secondary": "",
                    datetime: __props.notification.createdAt
                  }, "\n                ・" + _toDisplayString(_unref(timeAgo)), 9 /* TEXT, PROPS */, ["datetime"])
                ])
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["status", "in-notification", "actions"])) : ( __props.notification.type === 'mention' || __props.notification.type === 'poll' || __props.notification.type === 'status' || __props.notification.type === 'quote' ) ? (_openBlock(), _createBlock(_component_StatusCard, {
            key: 5,
            status: __props.notification.status
          }, null, 8 /* PROPS */, ["status"])) : (__props.notification.type === 'annual_report') ? (_openBlock(), _createElementBlock("div", {
            key: 6,
            flex: "",
            p4: "",
            "items-center": "",
            "bg-shaded": ""
          }, [ _hoisted_6, _createElementVNode("div", { class: "content-rich" }, [ _createElementVNode("p", null, [ _createTextVNode("\n            Your 2024 "), _createVNode(_component_NuxtLink, { to: "/tags/Wrapstodon" }, {
                  default: _withCtx(() => [
                    _createTextVNode("\n              #Wrapstodon\n            ")
                  ]),
                  _: 1 /* STABLE */
                }), _createTextVNode(" awaits! Unveil your year's highlights and memorable moments on Mastodon!\n          ") ]), _createElementVNode("p", null, [ _createVNode(_component_NuxtLink, {
                  to: `https://${_ctx.currentServer}/notifications`,
                  target: "_blank"
                }, {
                  default: _withCtx(() => [
                    _createTextVNode("\n              View #Wrapstodon on Mastodon\n            ")
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["to"]) ]) ]) ])) : _createCommentVNode("v-if", true) ]))
}
}

})
