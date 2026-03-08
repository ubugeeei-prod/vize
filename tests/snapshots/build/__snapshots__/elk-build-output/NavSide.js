import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { class: "spacer", shrink: "true", "xl:hidden": "true" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { class: "i-ri:notification-4-line", "text-xl": "true" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("div", { class: "spacer", shrink: "true", hidden: "true", "sm:block": "true" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("div", { class: "spacer", shrink: "true", hidden: "true", "sm:block": "true" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("div", { class: "spacer", shrink: "true", hidden: "true", "sm:block": "true" })
import { STORAGE_KEY_LAST_ACCESSED_EXPLORE_ROUTE, STORAGE_KEY_LAST_ACCESSED_NOTIFICATION_ROUTE } from '~/constants'

export default /*@__PURE__*/_defineComponent({
  __name: 'NavSide',
  props: {
    command: { type: Boolean, required: false }
  },
  setup(__props: any) {

const { notifications } = useNotifications()
const useStarFavoriteIcon = usePreferences('useStarFavoriteIcon')
const lastAccessedNotificationRoute = useLocalStorage(STORAGE_KEY_LAST_ACCESSED_NOTIFICATION_ROUTE, '')
const lastAccessedExploreRoute = useLocalStorage(STORAGE_KEY_LAST_ACCESSED_EXPLORE_ROUTE, '')
const notificationsLink = computed(() => {
  const hydrated = isHydrated.value
  const user = currentUser.value
  const lastRoute = lastAccessedNotificationRoute.value
  if (!hydrated || !user || !lastRoute) {
    return '/notifications'
  }

  return `/notifications/${lastRoute}`
})
const exploreLink = computed(() => {
  const hydrated = isHydrated.value
  const server = currentServer.value
  let lastRoute = lastAccessedExploreRoute.value
  if (!hydrated) {
    return '/explore'
  }

  if (lastRoute.length) {
    lastRoute = `/${lastRoute}`
  }

  return server ? `/${server}/explore${lastRoute}` : `/explore${lastRoute}`
})

return (_ctx: any,_cache: any) => {
  const _component_NavSideItem = _resolveComponent("NavSideItem")

  return (_openBlock(), _createElementBlock("nav", {
      "sm:px3": "",
      flex: "~ col gap2",
      shrink: "",
      "text-size-base": "",
      "leading-normal": "",
      "md:text-lg": "",
      "h-full": "",
      "mt-1": "",
      "overflow-y-auto": ""
    }, [ _createVNode(_component_NavSideItem, {
        text: _ctx.$t('nav.search'),
        to: "/search",
        icon: "i-ri:search-line",
        "xl:hidden": "",
        command: __props.command
      }, null, 8 /* PROPS */, ["text", "command"]), _hoisted_1, _createVNode(_component_NavSideItem, {
        text: _ctx.$t('nav.home'),
        to: "/home",
        icon: "i-ri:home-5-line",
        "user-only": "",
        command: __props.command
      }, null, 8 /* PROPS */, ["text", "command"]), _createVNode(_component_NavSideItem, {
        text: _ctx.$t('nav.notifications'),
        to: notificationsLink.value,
        icon: "i-ri:notification-4-line",
        "user-only": "",
        command: __props.command
      }, {
        icon: _withCtx(() => [
          _createElementVNode("div", {
            flex: "",
            relative: ""
          }, [
            _hoisted_2,
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
      }, 8 /* PROPS */, ["text", "to", "command"]), _createVNode(_component_NavSideItem, {
        text: _ctx.$t('nav.conversations'),
        to: "/conversations",
        icon: "i-ri:at-line",
        "user-only": "",
        command: __props.command
      }, null, 8 /* PROPS */, ["text", "command"]), _createVNode(_component_NavSideItem, {
        text: _ctx.$t('nav.favourites'),
        to: "/favourites",
        icon: _unref(useStarFavoriteIcon) ? 'i-ri:star-line' : 'i-ri:heart-3-line',
        "user-only": "",
        command: __props.command
      }, null, 8 /* PROPS */, ["text", "icon", "command"]), _createVNode(_component_NavSideItem, {
        text: _ctx.$t('nav.bookmarks'),
        to: "/bookmarks",
        icon: "i-ri:bookmark-line",
        "user-only": "",
        command: __props.command
      }, null, 8 /* PROPS */, ["text", "command"]), _hoisted_3, _createVNode(_component_NavSideItem, {
        text: _ctx.$t('action.compose'),
        to: "/compose",
        icon: "i-ri:quill-pen-line",
        "user-only": "",
        command: __props.command
      }, null, 8 /* PROPS */, ["text", "command"]), _createVNode(_component_NavSideItem, {
        text: _ctx.$t('nav.scheduled_posts'),
        to: "/scheduled-posts",
        icon: "i-ri:calendar-schedule-line",
        "user-only": "",
        command: __props.command
      }, null, 8 /* PROPS */, ["text", "command"]), _hoisted_4, _createVNode(_component_NavSideItem, {
        text: _ctx.$t('nav.explore'),
        to: exploreLink.value,
        icon: "i-ri:compass-3-line",
        command: __props.command
      }, null, 8 /* PROPS */, ["text", "to", "command"]), _createVNode(_component_NavSideItem, {
        text: _ctx.$t('nav.local'),
        to: _ctx.isHydrated ? `/${_ctx.currentServer}/public/local` : '/public/local',
        icon: "i-ri:group-2-line ",
        command: __props.command
      }, null, 8 /* PROPS */, ["text", "to", "command"]), _createVNode(_component_NavSideItem, {
        text: _ctx.$t('nav.federated'),
        to: _ctx.isHydrated ? `/${_ctx.currentServer}/public` : '/public',
        icon: "i-ri:earth-line",
        command: __props.command
      }, null, 8 /* PROPS */, ["text", "to", "command"]), _createVNode(_component_NavSideItem, {
        text: _ctx.$t('nav.lists'),
        to: _ctx.isHydrated ? `/${_ctx.currentServer}/lists` : '/lists',
        icon: "i-ri:list-check",
        "user-only": "",
        command: __props.command
      }, null, 8 /* PROPS */, ["text", "to", "command"]), _createVNode(_component_NavSideItem, {
        text: _ctx.$t('nav.hashtags'),
        to: "/hashtags",
        icon: "i-ri:hashtag",
        "user-only": "",
        command: __props.command
      }, null, 8 /* PROPS */, ["text", "command"]), _hoisted_5, _createVNode(_component_NavSideItem, {
        text: _ctx.$t('nav.settings'),
        to: "/settings",
        icon: "i-ri:settings-3-line",
        command: __props.command
      }, null, 8 /* PROPS */, ["text", "command"]) ]))
}
}

})
