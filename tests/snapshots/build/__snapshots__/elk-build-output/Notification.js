import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { class: "i-ri:notification-4-line", "text-xl": "true" })
import { STORAGE_KEY_LAST_ACCESSED_NOTIFICATION_ROUTE } from '~/constants'

export default /*@__PURE__*/_defineComponent({
  __name: 'Notification',
  props: {
    activeClass: { type: String, required: true }
  },
  setup(__props: any) {

const { notifications } = useNotifications()
const lastAccessedNotificationRoute = useLocalStorage(STORAGE_KEY_LAST_ACCESSED_NOTIFICATION_ROUTE, '')

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (_openBlock(), _createBlock(_component_NuxtLink, {
      to: `/notifications/${_unref(lastAccessedNotificationRoute)}`,
      "aria-label": _ctx.$t('nav.notifications'),
      "active-class": __props.activeClass,
      flex: "",
      "flex-row": "",
      "items-center": "",
      "place-content-center": "",
      "h-full": "",
      "flex-1": "",
      class: "coarse-pointer:select-none",
      onClick: _cache[0] || (_cache[0] = (...args) => (_ctx.$scrollToTop && _ctx.$scrollToTop(...args)))
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          flex: "",
          relative: ""
        }, [
          _hoisted_1,
          (_unref(notifications))
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "top-[-0.3rem] right-[-0.3rem]",
              absolute: "",
              "font-bold": "",
              "rounded-full": "",
              "h-4": "",
              "w-4": "",
              "text-xs": "",
              "bg-primary": "",
              "text-inverted": "",
              flex: "",
              "items-center": "",
              "justify-center": ""
            }, _toDisplayString(_unref(notifications) < 10 ? _unref(notifications) : '•'), 1 /* TEXT */))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["to", "aria-label", "active-class"]))
}
}

})
