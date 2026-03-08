import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"

import type { GroupedAccountLike, NotificationSlot } from '#shared/types'
import type { mastodon } from 'masto'
import { DynamicScrollerItem } from 'vue-virtual-scroller'
const virtualScroller = false // TODO: fix flickering issue with virtual scroll

export default /*@__PURE__*/_defineComponent({
  __name: 'NotificationPaginator',
  props: {
    paginator: { type: null, required: true },
    stream: { type: null, required: false }
  },
  setup(__props: any) {

// @ts-expect-error missing types
const groupCapacity = Number.MAX_VALUE // No limit
const includeNotificationTypes: mastodon.v1.NotificationType[] = [
  'update',
  'mention',
  'poll',
  'status',
  'quote',
]
let id = 0
function includeNotificationsForStatusCard({ type, status }: mastodon.v1.Notification) {
  // Exclude update, mention, pool and status notifications without the status entry:
  // no makes sense to include them
  // Those notifications will be shown using StatusCard SFC:
  // check NotificationCard SFC L68 and L81 => :status="notification.status!"
  return status || !includeNotificationTypes.includes(type)
}
// Group by type (and status when applicable)
function groupId(item: mastodon.v1.Notification): string {
  // If the update is related to a status, group notifications from the same account (boost + favorite the same status)
  const id = item.status
    ? {
        status: item.status?.id,
        type: (item.type === 'reblog' || item.type === 'favourite') ? 'like' : item.type,
      }
    : {
        type: item.type,
      }
  return JSON.stringify(id)
}
function hasHeader(account: mastodon.v1.Account) {
  return !account.header.endsWith('/original/missing.png')
}
function groupItems(items: mastodon.v1.Notification[]): NotificationSlot[] {
  const results: NotificationSlot[] = []
  let currentGroupId = ''
  let currentGroup: mastodon.v1.Notification[] = []
  const processGroup = () => {
    if (currentGroup.length === 0)
      return

    const group = currentGroup
    currentGroup = []

    // Only group follow notifications when there are too many in a row
    // This normally happens when you transfer an account, if not, show
    // a big profile card for each follow
    if (group[0].type === 'follow') {
      // Order group by followers count
      const processedGroup = [...group]
      processedGroup.sort((a, b) => {
        const aHasHeader = hasHeader(a.account)
        const bHasHeader = hasHeader(b.account)
        if (bHasHeader && !aHasHeader)
          return 1
        if (aHasHeader && !bHasHeader)
          return -1
        return b.account.followersCount - a.account.followersCount
      })

      if (processedGroup.length > 0 && hasHeader(processedGroup[0].account))
        results.push(processedGroup.shift()!)

      if (processedGroup.length === 1 && hasHeader(processedGroup[0].account))
        results.push(processedGroup.shift()!)

      if (processedGroup.length > 0) {
        results.push({
          id: `grouped-${id++}`,
          type: 'grouped-follow',
          items: processedGroup,
        })
      }
      return
    }
    else if (group.length && (group[0].type === 'reblog' || group[0].type === 'favourite')) {
      if (!group[0].status) {
        // Ignore favourite or reblog if status is null, sometimes the API is sending these
        // notifications
        return
      }
      // All notifications in these group are reblogs or favourites of the same status
      const likes: GroupedAccountLike[] = []
      for (const notification of group) {
        let like = likes.find(like => like.account.id === notification.account.id)
        if (!like) {
          like = { account: notification.account }
          likes.push(like)
        }
        like[notification.type === 'reblog' ? 'reblog' : 'favourite'] = notification
      }
      likes.sort((a, b) => a.reblog
        ? (!b.reblog || (a.favourite && !b.favourite))
            ? -1
            : 0
        : 0)
      results.push({
        id: `grouped-${id++}`,
        type: 'grouped-reblogs-and-favourites',
        status: group[0].status,
        likes,
      })
      return
    }

    results.push(...group)
  }
  for (const item of items.filter(includeNotificationsForStatusCard)) {
    const itemId = groupId(item)
    // Finalize the group if it already has too many notifications
    if (currentGroupId !== itemId || currentGroup.length >= groupCapacity)
      processGroup()
    currentGroup.push(item)
    currentGroupId = itemId
  }
  // Finalize remaining groups
  processGroup()
  return results
}
function removeFiltered(items: mastodon.v1.Notification[]): mastodon.v1.Notification[] {
  return items.filter(item => !item.status?.filtered?.find(
    filter => filter.filter.filterAction === 'hide' && filter.filter.context.includes('notifications'),
  ))
}
function preprocess(items: NotificationSlot[]): NotificationSlot[] {
  const flattenedNotifications: mastodon.v1.Notification[] = []
  for (const item of items) {
    if (item.type === 'grouped-reblogs-and-favourites') {
      const group = item
      for (const like of group.likes) {
        if (like.reblog)
          flattenedNotifications.push(like.reblog)
        if (like.favourite)
          flattenedNotifications.push(like.favourite)
      }
    }
    else if (item.type === 'grouped-follow') {
      flattenedNotifications.push(...item.items)
    }
    else {
      flattenedNotifications.push(item)
    }
  }
  return groupItems(removeFiltered(flattenedNotifications))
}
const { clearNotifications } = useNotifications()
const { formatNumber } = useHumanReadableNumber()

return (_ctx: any,_cache: any) => {
  const _component_NotificationGroupedFollow = _resolveComponent("NotificationGroupedFollow")
  const _component_NotificationGroupedLikes = _resolveComponent("NotificationGroupedLikes")
  const _component_NotificationCard = _resolveComponent("NotificationCard")
  const _component_CommonPaginator = _resolveComponent("CommonPaginator")

  return (_openBlock(), _createBlock(_component_CommonPaginator, {
      paginator: __props.paginator,
      preprocess: preprocess,
      stream: __props.stream,
      eventType: "notification",
      virtualScroller: virtualScroller
    }, {
      updater: _withCtx(({ number, update }) => [
        _createElementVNode("button", {
          id: "elk_show_new_items",
          "py-4": "",
          border: "b base",
          flex: "~ col",
          "p-3": "",
          "w-full": "",
          "text-primary": "",
          "font-bold": "",
          onClick: _cache[0] || (_cache[0] = () => { _ctx.update(); _unref(clearNotifications)() })
        }, _toDisplayString(_ctx.$t('timeline.show_new_items', number, { named: { v: _unref(formatNumber)(number) } })), 1 /* TEXT */)
      ]),
      default: _withCtx(({ item, active }) => [
        (virtualScroller)
          ? (_openBlock(), _createBlock(DynamicScrollerItem, {
            key: 0,
            item: item,
            active: active,
            tag: "div"
          }, {
            default: _withCtx(() => [
              (item.type === 'grouped-follow')
                ? (_openBlock(), _createBlock(_component_NotificationGroupedFollow, {
                  key: 0,
                  items: item,
                  border: "b base"
                }, null, 8 /* PROPS */, ["items"]))
                : (item.type === 'grouped-reblogs-and-favourites')
                  ? (_openBlock(), _createBlock(_component_NotificationGroupedLikes, {
                    key: 1,
                    group: item,
                    border: "b base"
                  }, null, 8 /* PROPS */, ["group"]))
                : (_openBlock(), _createBlock(_component_NotificationCard, {
                  key: 2,
                  notification: item,
                  "hover:bg-active": "",
                  border: "b base"
                }, null, 8 /* PROPS */, ["notification"]))
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["item", "active"]))
          : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
            (item.type === 'grouped-follow')
              ? (_openBlock(), _createBlock(_component_NotificationGroupedFollow, {
                key: 0,
                items: item,
                border: "b base"
              }, null, 8 /* PROPS */, ["items"]))
              : (item.type === 'grouped-reblogs-and-favourites')
                ? (_openBlock(), _createBlock(_component_NotificationGroupedLikes, {
                  key: 1,
                  group: item,
                  border: "b base"
                }, null, 8 /* PROPS */, ["group"]))
              : (_openBlock(), _createBlock(_component_NotificationCard, {
                key: 2,
                notification: item,
                "hover:bg-active": "",
                border: "b base"
              }, null, 8 /* PROPS */, ["notification"]))
          ], 64 /* STABLE_FRAGMENT */))
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["paginator", "preprocess", "stream", "virtualScroller"]))
}
}

})
