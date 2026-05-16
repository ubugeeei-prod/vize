import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "aria-hidden": "true", "i-ri:error-warning-fill": "true" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", w: "1.75em", h: "1.75em", "i-ri:close-line": "true" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { "inline-block": "true", "aria-hidden": "true", "i-ri:external-link-line": "true", class: "rtl-flip" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { "inline-block": "true", "aria-hidden": "true", "i-ri:external-link-line": "true", class: "rtl-flip" })

export default /*@__PURE__*/_defineComponent({
  __name: 'NotificationSubscribePushNotificationError',
  props: {
    title: { type: String, required: false },
    message: { type: String, required: true },
    "modelValue": { required: true }
  },
  emits: ["update:modelValue"],
  setup(__props: any) {

const modelValue = _useModel(__props, "modelValue")

return (_ctx: any,_cache: any) => {
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_i18n_t = _resolveComponent("i18n-t")

  return (modelValue.value)
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        role: "alert",
        "aria-describedby": "notification-failed",
        flex: "~ col",
        "gap-1": "",
        "text-sm": "",
        "pt-1": "",
        "ps-2": "",
        "pe-1": "",
        "pb-2": "",
        "text-red-600": "",
        "dark:text-red-400": "",
        border: "~ base rounded red-600 dark:red-400"
      }, [ _createElementVNode("header", {
          id: "notification-failed",
          flex: "",
          "justify-between": ""
        }, [ _createElementVNode("div", {
            flex: "",
            "items-center": "",
            "gap-x-2": "",
            "font-bold": ""
          }, [ _hoisted_1, _createElementVNode("p", null, _toDisplayString(__props.title ?? _ctx.$t('settings.notifications.push_notifications.subscription_error.title')), 1 /* TEXT */) ]), _createVNode(_component_CommonTooltip, {
            placement: "bottom",
            content: _ctx.$t('settings.notifications.push_notifications.subscription_error.clear_error')
          }, {
            default: _withCtx(() => [
              _createElementVNode("button", {
                flex: "",
                "rounded-4": "",
                p1: "",
                "hover:bg-active": "",
                "cursor-pointer": "",
                "transition-100": "",
                "aria-label": _ctx.$t('settings.notifications.push_notifications.subscription_error.clear_error'),
                onClick: _cache[0] || (_cache[0] = ($event: any) => (modelValue.value = false))
              }, [
                _hoisted_2
              ], 8 /* PROPS */, ["aria-label"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["content"]) ]), _createElementVNode("p", null, _toDisplayString(__props.message), 1 /* TEXT */), _createElementVNode("p", { "py-2": "" }, [ _createVNode(_component_i18n_t, { keypath: "settings.notifications.push_notifications.subscription_error.error_hint" }, {
            default: _withCtx(() => [
              _createVNode(_component_NuxtLink, {
                "font-bold": "",
                href: "https://docs.elk.zone/pwa#faq",
                target: "_blank",
                "inline-flex": "~ row",
                "items-center": "",
                "gap-x-2": ""
              }, {
                default: _withCtx(() => [
                  _createTextVNode("\n          https://docs.elk.zone/pwa#faq\n          "),
                  _hoisted_3
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }) ]), _createElementVNode("p", { "py-2": "" }, [ _createVNode(_component_NuxtLink, {
            "font-bold": "",
            "text-primary": "",
            href: "https://github.com/elk-zone/elk",
            target: "_blank",
            flex: "~ row",
            "items-center": "",
            "gap-x-2": ""
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_ctx.$t('settings.notifications.push_notifications.subscription_error.repo_link')), 1 /* TEXT */),
              _createTextVNode("\n        "),
              _hoisted_4
            ]),
            _: 1 /* STABLE */
          }) ]) ]))
      : _createCommentVNode("v-if", true)
}
}

})
