import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: '[filter]',
  setup(__props) {

const route = useRoute()
const { t } = useI18n()
const filter = computed<mastodon.v1.NotificationType | undefined>(() => {
  if (!isHydrated.value)
    return undefined

  const rawFilter = route.params?.filter
  const actualFilter = Array.isArray(rawFilter) ? rawFilter[0] : rawFilter
  if (isNotification(actualFilter))
    return actualFilter

  return undefined
})
useHydratedHead({
  title: () => `${t(`tab.notifications_${filter.value ?? 'all'}`)} | ${t('nav.notifications')}`,
})

return (_ctx: any,_cache: any) => {
  const _component_TimelineNotifications = _resolveComponent("TimelineNotifications")

  return (_ctx.isHydrated)
      ? (_openBlock(), _createBlock(_component_TimelineNotifications, {
        key: 0,
        filter: filter.value
      }, null, 8 /* PROPS */, ["filter"]))
      : _createCommentVNode("v-if", true)
}
}

})
