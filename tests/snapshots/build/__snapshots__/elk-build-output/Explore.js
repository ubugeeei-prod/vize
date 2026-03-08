import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"

import type { CommonRouteTabOption } from '#shared/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'explore',
  setup(__props) {

const { t } = useI18n()
const search = ref<{ input?: HTMLInputElement }>()
const route = useRoute()
watchEffect(() => {
  if (isMediumOrLargeScreen && route.name === 'explore' && search.value?.input)
    search.value?.input?.focus()
})
onActivated(() =>
  search.value?.input?.focus(),
)
onDeactivated(() => search.value?.input?.blur())
const userSettings = useUserSettings()
const tabs = computed<CommonRouteTabOption[]>(() => [
  {
    to: isHydrated.value ? `/${currentServer.value}/explore` : '/explore',
    display: t('tab.posts'),
  },
  {
    to: isHydrated.value ? `/${currentServer.value}/explore/tags` : '/explore/tags',
    display: t('tab.hashtags'),
  },
  {
    to: isHydrated.value ? `/${currentServer.value}/explore/links` : '/explore/links',
    display: t('tab.news'),
    hide: userSettings.value.preferences.hideNews,
  },
  // This section can only be accessed after logging in
  {
    to: isHydrated.value ? `/${currentServer.value}/explore/users` : '/explore/users',
    display: t('tab.for_you'),
    disabled: !isHydrated.value || !currentUser.value,
  },
])

return (_ctx: any,_cache: any) => {
  const _component_MainTitle = _resolveComponent("MainTitle")
  const _component_CommonRouteTabs = _resolveComponent("CommonRouteTabs")
  const _component_NuxtPage = _resolveComponent("NuxtPage")
  const _component_MainContent = _resolveComponent("MainContent")

  return (_openBlock(), _createBlock(_component_MainContent, null, {
      title: _withCtx(() => [
        _createVNode(_component_MainTitle, { icon: "i-ri:compass-3-line" }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(t)('nav.explore')), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })
      ]),
      header: _withCtx(() => [
        _createVNode(_component_CommonRouteTabs, {
          replace: "",
          options: tabs.value
        }, null, 8 /* PROPS */, ["options"])
      ]),
      default: _withCtx(() => [
        (_ctx.isHydrated)
          ? (_openBlock(), _createBlock(_component_NuxtPage, { key: 0 }))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
