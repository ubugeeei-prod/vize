import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import type { mastodon } from 'masto'
import { toggleBlockAccount, toggleMuteAccount, useRelationship } from '~/composables/masto/relationship'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusActionsMore',
  props: {
    status: { type: null, required: true },
    details: { type: Boolean, required: false },
    command: { type: Boolean, required: false }
  },
  emits: ["afterEdit"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const focusEditor = inject<typeof noop>('focus-editor', noop)
const {
  status,
  isLoading,
  toggleBookmark,
  toggleFavourite,
  togglePin,
  toggleReblog,
  toggleMute,
} = useStatusActions({ status: __props.status })
const clipboard = useClipboard()
const router = useRouter()
const route = useRoute()
const { t } = useI18n()
const userSettings = useUserSettings()
const useStarFavoriteIcon = usePreferences('useStarFavoriteIcon')
const isAuthor = computed(() => status.value.account.id === currentUser.value?.account.id)
const { client } = useMasto()
function getPermalinkUrl(status: mastodon.v1.Status) {
  const url = getStatusPermalinkRoute(status)
  if (url)
    return `${location.origin}/${url}`
  return null
}
async function copyLink(status: mastodon.v1.Status) {
  const url = getPermalinkUrl(status)
  if (url)
    await clipboard.copy(url)
}
async function copyOriginalLink(status: mastodon.v1.Status) {
  const url = status.url
  if (url)
    await clipboard.copy(url)
}
const { share, isSupported: isShareSupported } = useShare()
async function shareLink(status: mastodon.v1.Status) {
  const url = getPermalinkUrl(status)
  if (url)
    await share({ url })
}
async function deleteStatus() {
  const confirmDelete = await openConfirmDialog({
    title: t('confirm.delete_posts.title'),
    description: t('confirm.delete_posts.description'),
    confirm: t('confirm.delete_posts.confirm'),
    cancel: t('confirm.delete_posts.cancel'),
  })
  if (confirmDelete.choice !== 'confirm')
    return
  removeCachedStatus(status.value.id)
  await client.value.v1.statuses.$select(status.value.id).remove()
  if (route.name === 'status')
    router.back()
  // TODO when timeline, remove this item
}
async function deleteAndRedraft() {
  const confirmDelete = await openConfirmDialog({
    title: t('confirm.delete_posts.title'),
    description: t('confirm.delete_posts.description'),
    confirm: t('confirm.delete_posts.confirm'),
    cancel: t('confirm.delete_posts.cancel'),
  })
  if (confirmDelete.choice !== 'confirm')
    return
  if (import.meta.dev) {
    // eslint-disable-next-line no-alert
    const result = confirm('[DEV] Are you sure you want to delete and re-draft this post?')
    if (!result)
      return
  }
  removeCachedStatus(status.value.id)
  await client.value.v1.statuses.$select(status.value.id).remove()
  await openPublishDialog('dialog', await getDraftFromStatus(status.value), true)
  // Go to the new status, if the page is the old status
  if (lastPublishDialogStatus.value && route.name === 'status')
    router.push(getStatusRoute(lastPublishDialogStatus.value))
}
function reply() {
  if (!checkLogin())
    return
  if (__props.details) {
    focusEditor()
  }
  else {
    const { key, draft } = getReplyDraft(status.value)
    openPublishDialog(key, draft())
  }
}
async function editStatus() {
  await openPublishDialog(`edit-${status.value.id}`, {
    ...await getDraftFromStatus(status.value),
    editingStatus: status.value,
  }, true)
  emit('afterEdit')
}
function showReactedBy() {
  openReactedByDialog(status.value.id)
}

return (_ctx: any,_cache: any) => {
  const _component_StatusActionButton = _resolveComponent("StatusActionButton")
  const _component_CommonDropdownItem = _resolveComponent("CommonDropdownItem")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_CommonDropdown = _resolveComponent("CommonDropdown")

  return (_openBlock(), _createBlock(_component_CommonDropdown, {
      "flex-none": "",
      ms3: "",
      placement: "bottom",
      "eager-mount": __props.command
    }, {
      popper: _withCtx(() => [
        _createElementVNode("div", { flex: "~ col" }, [
          (_ctx.getPreferences(_unref(userSettings), 'zenMode') && !__props.details)
            ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
              _createVNode(_component_CommonDropdownItem, {
                is: "button",
                text: _ctx.$t('action.reply'),
                icon: "i-ri:chat-1-line",
                command: __props.command,
                onClick: _cache[0] || (_cache[0] = ($event: any) => (reply()))
              }, null, 8 /* PROPS */, ["text", "command"]),
              _createVNode(_component_CommonDropdownItem, {
                is: "button",
                text: _unref(status).reblogged ? _ctx.$t('action.boosted') : _ctx.$t('action.boost'),
                icon: "i-ri:repeat-fill",
                class: _normalizeClass(_unref(status).reblogged ? 'text-green' : ''),
                command: __props.command,
                disabled: _unref(isLoading).reblogged,
                onClick: _cache[1] || (_cache[1] = ($event: any) => (_unref(toggleReblog)()))
              }, null, 10 /* CLASS, PROPS */, ["text", "command", "disabled"]),
              _createVNode(_component_CommonDropdownItem, {
                is: "button",
                text: _unref(status).favourited ? _ctx.$t('action.favourited') : _ctx.$t('action.favourite'),
                icon: _unref(useStarFavoriteIcon)
                ? _unref(status).favourited ? 'i-ri:star-fill' : 'i-ri:star-line'
                : _unref(status).favourited ? 'i-ri:heart-3-fill' : 'i-ri:heart-3-line',
                class: _normalizeClass(_unref(status).favourited
                ? _unref(useStarFavoriteIcon) ? 'text-yellow' : 'text-rose'
                : ''
              ),
                command: __props.command,
                disabled: _unref(isLoading).favourited,
                onClick: _cache[2] || (_cache[2] = ($event: any) => (_unref(toggleFavourite)()))
              }, null, 10 /* CLASS, PROPS */, ["text", "icon", "command", "disabled"]),
              _createVNode(_component_CommonDropdownItem, {
                is: "button",
                text: _unref(status).bookmarked ? _ctx.$t('action.bookmarked') : _ctx.$t('action.bookmark'),
                icon: _unref(status).bookmarked ? 'i-ri:bookmark-fill' : 'i-ri:bookmark-line',
                class: _normalizeClass(_unref(status).bookmarked
                ? _unref(useStarFavoriteIcon) ? 'text-rose' : 'text-yellow'
                : ''
              ),
                command: __props.command,
                disabled: _unref(isLoading).bookmarked,
                onClick: _cache[3] || (_cache[3] = ($event: any) => (_unref(toggleBookmark)()))
              }, null, 10 /* CLASS, PROPS */, ["text", "icon", "command", "disabled"])
            ], 64 /* STABLE_FRAGMENT */))
            : _createCommentVNode("v-if", true),
          _createVNode(_component_CommonDropdownItem, {
            is: "button",
            text: _ctx.$t('menu.show_reacted_by'),
            icon: "i-ri:hearts-line",
            command: __props.command,
            onClick: _cache[4] || (_cache[4] = ($event: any) => (showReactedBy()))
          }, null, 8 /* PROPS */, ["text", "command"]),
          _createVNode(_component_CommonDropdownItem, {
            is: "button",
            text: _ctx.$t('menu.copy_link_to_post'),
            icon: "i-ri:link",
            command: __props.command,
            onClick: _cache[5] || (_cache[5] = ($event: any) => (copyLink(_unref(status))))
          }, null, 8 /* PROPS */, ["text", "command"]),
          _createVNode(_component_CommonDropdownItem, {
            is: "button",
            text: _ctx.$t('menu.copy_original_link_to_post'),
            icon: "i-ri:links-fill",
            command: __props.command,
            onClick: _cache[6] || (_cache[6] = ($event: any) => (copyOriginalLink(_unref(status))))
          }, null, 8 /* PROPS */, ["text", "command"]),
          (_unref(isShareSupported))
            ? (_openBlock(), _createBlock(_component_CommonDropdownItem, {
              key: 0,
              is: "button",
              text: _ctx.$t('menu.share_post'),
              icon: "i-ri:share-line",
              command: __props.command,
              onClick: _cache[7] || (_cache[7] = ($event: any) => (shareLink(_unref(status))))
            }, null, 8 /* PROPS */, ["text", "command"]))
            : _createCommentVNode("v-if", true),
          (_ctx.currentUser && (_unref(status).account.id === _ctx.currentUser.account.id || _unref(status).mentions.some((m) => m.id === _ctx.currentUser.account.id)))
            ? (_openBlock(), _createBlock(_component_CommonDropdownItem, {
              key: 0,
              is: "button",
              text: _unref(status).muted ? _ctx.$t('menu.unmute_conversation') : _ctx.$t('menu.mute_conversation'),
              icon: _unref(status).muted ? 'i-ri:eye-line' : 'i-ri:eye-off-line',
              command: __props.command,
              disabled: _unref(isLoading).muted,
              onClick: _cache[8] || (_cache[8] = ($event: any) => (_unref(toggleMute)()))
            }, null, 8 /* PROPS */, ["text", "icon", "command", "disabled"]))
            : _createCommentVNode("v-if", true),
          (_unref(status).url)
            ? (_openBlock(), _createBlock(_component_NuxtLink, {
              key: 0,
              to: _unref(status).url,
              external: "",
              target: "_blank"
            }, {
              default: _withCtx(() => [
                _createVNode(_component_CommonDropdownItem, {
                  text: _ctx.$t('menu.open_in_original_site'),
                  icon: "i-ri:arrow-right-up-line",
                  command: __props.command
                }, null, 8 /* PROPS */, ["text", "command"])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to"]))
            : _createCommentVNode("v-if", true),
          (_ctx.isHydrated && _ctx.currentUser)
            ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
              (isAuthor.value)
                ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                  _createVNode(_component_CommonDropdownItem, {
                    is: "button",
                    text: _unref(status).pinned ? _ctx.$t('menu.unpin_on_profile') : _ctx.$t('menu.pin_on_profile'),
                    icon: "i-ri:pushpin-line",
                    command: __props.command,
                    onClick: _cache[9] || (_cache[9] = (...args) => (togglePin && togglePin(...args)))
                  }, null, 8 /* PROPS */, ["text", "command"]),
                  _createVNode(_component_CommonDropdownItem, {
                    is: "button",
                    text: _ctx.$t('menu.edit'),
                    icon: "i-ri:edit-line",
                    command: __props.command,
                    onClick: editStatus
                  }, null, 8 /* PROPS */, ["text", "command"]),
                  _createVNode(_component_CommonDropdownItem, {
                    is: "button",
                    text: _ctx.$t('menu.delete'),
                    icon: "i-ri:delete-bin-line",
                    "text-red-600": "",
                    command: __props.command,
                    onClick: deleteStatus
                  }, null, 8 /* PROPS */, ["text", "command"]),
                  _createVNode(_component_CommonDropdownItem, {
                    is: "button",
                    text: _ctx.$t('menu.delete_and_redraft'),
                    icon: "i-ri:eraser-line",
                    "text-red-600": "",
                    command: __props.command,
                    onClick: deleteAndRedraft
                  }, null, 8 /* PROPS */, ["text", "command"])
                ], 64 /* STABLE_FRAGMENT */))
                : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                  _createVNode(_component_CommonDropdownItem, {
                    is: "button",
                    text: _ctx.$t('menu.mention_account', [`@${_unref(status).account.acct}`]),
                    icon: "i-ri:at-line",
                    command: __props.command,
                    onClick: _cache[10] || (_cache[10] = ($event: any) => (_ctx.mentionUser(_unref(status).account)))
                  }, null, 8 /* PROPS */, ["text", "command"]),
                  (!_unref(useRelationship)(_unref(status).account).value?.muting)
                    ? (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                      key: 0,
                      is: "button",
                      text: _ctx.$t('menu.mute_account', [`@${_unref(status).account.acct}`]),
                      icon: "i-ri:volume-mute-line",
                      command: __props.command,
                      onClick: _cache[11] || (_cache[11] = ($event: any) => (_unref(toggleMuteAccount)(_unref(useRelationship)(_unref(status).account).value, _unref(status).account)))
                    }, null, 8 /* PROPS */, ["text", "command"]))
                    : (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                      key: 1,
                      is: "button",
                      text: _ctx.$t('menu.unmute_account', [`@${_unref(status).account.acct}`]),
                      icon: "i-ri:volume-up-fill",
                      command: __props.command,
                      onClick: _cache[12] || (_cache[12] = ($event: any) => (_unref(toggleMuteAccount)(_unref(useRelationship)(_unref(status).account).value, _unref(status).account)))
                    }, null, 8 /* PROPS */, ["text", "command"])),
                  (!_unref(useRelationship)(_unref(status).account).value?.blocking)
                    ? (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                      key: 0,
                      is: "button",
                      text: _ctx.$t('menu.block_account', [`@${_unref(status).account.acct}`]),
                      icon: "i-ri:forbid-2-line",
                      command: __props.command,
                      onClick: _cache[13] || (_cache[13] = ($event: any) => (_unref(toggleBlockAccount)(_unref(useRelationship)(_unref(status).account).value, _unref(status).account)))
                    }, null, 8 /* PROPS */, ["text", "command"]))
                    : (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                      key: 1,
                      is: "button",
                      text: _ctx.$t('menu.unblock_account', [`@${_unref(status).account.acct}`]),
                      icon: "i-ri:checkbox-circle-line",
                      command: __props.command,
                      onClick: _cache[14] || (_cache[14] = ($event: any) => (_unref(toggleBlockAccount)(_unref(useRelationship)(_unref(status).account).value, _unref(status).account)))
                    }, null, 8 /* PROPS */, ["text", "command"])),
                  (_ctx.getServerName(_unref(status).account) && _ctx.getServerName(_unref(status).account) !== _ctx.currentServer)
                    ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                      (!_unref(useRelationship)(_unref(status).account).value?.domainBlocking)
                        ? (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                          key: 0,
                          is: "button",
                          text: _ctx.$t('menu.block_domain', [_ctx.getServerName(_unref(status).account)]),
                          icon: "i-ri:shut-down-line",
                          command: __props.command,
                          onClick: _cache[15] || (_cache[15] = ($event: any) => (_ctx.toggleBlockDomain(_unref(useRelationship)(_unref(status).account).value, _unref(status).account)))
                        }, null, 8 /* PROPS */, ["text", "command"]))
                        : (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                          key: 1,
                          is: "button",
                          text: _ctx.$t('menu.unblock_domain', [_ctx.getServerName(_unref(status).account)]),
                          icon: "i-ri:restart-line",
                          command: __props.command,
                          onClick: _cache[16] || (_cache[16] = ($event: any) => (_ctx.toggleBlockDomain(_unref(useRelationship)(_unref(status).account).value, _unref(status).account)))
                        }, null, 8 /* PROPS */, ["text", "command"]))
                    ], 64 /* STABLE_FRAGMENT */))
                    : _createCommentVNode("v-if", true),
                  _createVNode(_component_CommonDropdownItem, {
                    is: "button",
                    text: _ctx.$t('menu.report_account', [`@${_unref(status).account.acct}`]),
                    icon: "i-ri:flag-2-line",
                    command: __props.command,
                    onClick: _cache[17] || (_cache[17] = ($event: any) => (_ctx.openReportDialog(_unref(status).account, _unref(status))))
                  }, null, 8 /* PROPS */, ["text", "command"])
                ], 64 /* STABLE_FRAGMENT */))
            ], 64 /* STABLE_FRAGMENT */))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      default: _withCtx(() => [
        _createVNode(_component_StatusActionButton, {
          content: _ctx.$t('action.more'),
          color: "text-primary",
          hover: "text-primary",
          "elk-group-hover": "bg-primary-light",
          icon: "i-ri:more-line",
          "my--2": ""
        }, null, 8 /* PROPS */, ["content"])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["eager-mount"]))
}
}

})
