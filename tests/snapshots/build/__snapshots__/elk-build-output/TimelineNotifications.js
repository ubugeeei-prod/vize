import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, resolveComponent as _resolveComponent, normalizeProps as _normalizeProps, guardReactiveProps as _guardReactiveProps, unref as _unref } from "vue"

import type { mastodon } from 'masto'
import { STORAGE_KEY_LAST_ACCESSED_NOTIFICATION_ROUTE } from '~/constants'

export default /*@__PURE__*/_defineComponent({
  __name: 'TimelineNotifications',
  props: {
    filter: { type: null, required: false }
  },
  setup(__props: any) {

const route = useRoute()
const lastAccessedNotificationRoute = useLocalStorage(STORAGE_KEY_LAST_ACCESSED_NOTIFICATION_ROUTE, '')
const options = { limit: 30, types: __props.filter ? [__props.filter] : [] }
// Default limit is 20 notifications, and servers are normally caped to 30
const paginator = useMastoClient().v1.notifications.list(options)
const stream = useStreaming(client => client.user.notification.subscribe())
lastAccessedNotificationRoute.value = route.path.replace(/\/notifications\/?/, '')
const { clearNotifications } = useNotifications()
onActivated(() => {
  clearNotifications()
  lastAccessedNotificationRoute.value = route.path.replace(/\/notifications\/?/, '')
})

return (_ctx: any,_cache: any) => {
  const _component_NotificationPaginator = _resolveComponent("NotificationPaginator")

  return (_openBlock(), _createBlock(_component_NotificationPaginator, _normalizeProps(_guardReactiveProps({ paginator: _unref(paginator), stream: _unref(stream) })), null, 16 /* FULL_PROPS */))
}
}

})
