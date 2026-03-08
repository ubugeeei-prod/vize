import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:more-2-fill": "true" })
import type { mastodon } from 'masto'
import { toggleBlockAccount, toggleBlockDomain, toggleMuteAccount } from '~/composables/masto/relationship'

export default /*@__PURE__*/_defineComponent({
  __name: 'AccountMoreButton',
  props: {
    account: { type: null, required: true },
    command: { type: Boolean, required: false }
  },
  emits: ["addNote", "removeNote"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const relationship = useRelationship(__props.account)
const isSelf = useSelfAccount(() => __props.account)
const { t } = useI18n()
const { client } = useMasto()
const useStarFavoriteIcon = usePreferences('useStarFavoriteIcon')
const { share, isSupported: isShareSupported } = useShare()
function shareAccount() {
  share({ url: location.href })
}
async function toggleReblogs() {
  if (!relationship.value!.showingReblogs) {
    const dialogChoice = await openConfirmDialog({
      title: t('confirm.show_reblogs.title'),
      description: t('confirm.show_reblogs.description', [__props.account.acct]),
      confirm: t('confirm.show_reblogs.confirm'),
      cancel: t('confirm.show_reblogs.cancel'),
    })
    if (dialogChoice.choice !== 'confirm')
      return
  }
  const showingReblogs = !relationship.value?.showingReblogs
  relationship.value = await client.value.v1.accounts.$select(__props.account.id).follow({ reblogs: showingReblogs })
}
async function addUserNote() {
  emit('addNote')
}
async function removeUserNote() {
  if (!relationship.value!.note || relationship.value!.note.length === 0)
    return
  const newNote = await client.value.v1.accounts.$select(__props.account.id).note.create({ comment: '' })
  relationship.value!.note = newNote.note
  emit('removeNote')
}

return (_ctx: any,_cache: any) => {
  const _component_CommonDropdownItem = _resolveComponent("CommonDropdownItem")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_CommonDropdown = _resolveComponent("CommonDropdown")

  return (_openBlock(), _createBlock(_component_CommonDropdown, { "eager-mount": __props.command }, {
      popper: _withCtx(() => [
        _createVNode(_component_NuxtLink, {
          to: __props.account.url,
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
        }, 8 /* PROPS */, ["to"]),
        (_unref(isShareSupported))
          ? (_openBlock(), _createBlock(_component_CommonDropdownItem, {
            key: 0,
            is: "button",
            text: _ctx.$t('menu.share_account', [`@${__props.account.acct}`]),
            icon: "i-ri:share-line",
            command: __props.command,
            onClick: _cache[0] || (_cache[0] = ($event: any) => (shareAccount()))
          }, null, 8 /* PROPS */, ["text", "command"]))
          : _createCommentVNode("v-if", true),
        (_ctx.currentUser)
          ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
            (!_unref(isSelf))
              ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                _createVNode(_component_CommonDropdownItem, {
                  is: "button",
                  text: _ctx.$t('menu.mention_account', [`@${__props.account.acct}`]),
                  icon: "i-ri:at-line",
                  command: __props.command,
                  onClick: _cache[1] || (_cache[1] = ($event: any) => (_ctx.mentionUser(__props.account)))
                }, null, 8 /* PROPS */, ["text", "command"]),
                _createVNode(_component_CommonDropdownItem, {
                  is: "button",
                  text: _ctx.$t('menu.private_mention_account', [`@${__props.account.acct}`]),
                  icon: "i-ri:message-3-line",
                  command: __props.command,
                  onClick: _cache[2] || (_cache[2] = ($event: any) => (_ctx.privateMentionUser(__props.account)))
                }, null, 8 /* PROPS */, ["text", "command"]),
                (!_unref(relationship)?.showingReblogs)
                  ? (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                    key: 0,
                    is: "button",
                    icon: "i-ri:repeat-line",
                    text: _ctx.$t('menu.show_reblogs', [`@${__props.account.acct}`]),
                    command: __props.command,
                    onClick: _cache[3] || (_cache[3] = ($event: any) => (toggleReblogs()))
                  }, null, 8 /* PROPS */, ["text", "command"]))
                  : (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                    key: 1,
                    is: "button",
                    text: _ctx.$t('menu.hide_reblogs', [`@${__props.account.acct}`]),
                    icon: "i-ri:repeat-line",
                    command: __props.command,
                    onClick: _cache[4] || (_cache[4] = ($event: any) => (toggleReblogs()))
                  }, null, 8 /* PROPS */, ["text", "command"])),
                (!_unref(relationship)?.note || _unref(relationship)?.note?.length === 0)
                  ? (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                    key: 0,
                    is: "button",
                    text: _ctx.$t('menu.add_personal_note', [`@${__props.account.acct}`]),
                    icon: "i-ri-edit-2-line",
                    command: __props.command,
                    onClick: _cache[5] || (_cache[5] = ($event: any) => (addUserNote()))
                  }, null, 8 /* PROPS */, ["text", "command"]))
                  : (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                    key: 1,
                    is: "button",
                    text: _ctx.$t('menu.remove_personal_note', [`@${__props.account.acct}`]),
                    icon: "i-ri-edit-2-line",
                    command: __props.command,
                    onClick: _cache[6] || (_cache[6] = ($event: any) => (removeUserNote()))
                  }, null, 8 /* PROPS */, ["text", "command"])),
                (!_unref(relationship)?.muting)
                  ? (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                    key: 0,
                    is: "button",
                    text: _ctx.$t('menu.mute_account', [`@${__props.account.acct}`]),
                    icon: "i-ri:volume-mute-line",
                    command: __props.command,
                    onClick: _cache[7] || (_cache[7] = ($event: any) => (_unref(toggleMuteAccount)(_unref(relationship), __props.account)))
                  }, null, 8 /* PROPS */, ["text", "command"]))
                  : (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                    key: 1,
                    is: "button",
                    text: _ctx.$t('menu.unmute_account', [`@${__props.account.acct}`]),
                    icon: "i-ri:volume-up-fill",
                    command: __props.command,
                    onClick: _cache[8] || (_cache[8] = ($event: any) => (_unref(toggleMuteAccount)(_unref(relationship), __props.account)))
                  }, null, 8 /* PROPS */, ["text", "command"])),
                (!_unref(relationship)?.blocking)
                  ? (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                    key: 0,
                    is: "button",
                    text: _ctx.$t('menu.block_account', [`@${__props.account.acct}`]),
                    icon: "i-ri:forbid-2-line",
                    command: __props.command,
                    onClick: _cache[9] || (_cache[9] = ($event: any) => (_unref(toggleBlockAccount)(_unref(relationship), __props.account)))
                  }, null, 8 /* PROPS */, ["text", "command"]))
                  : (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                    key: 1,
                    is: "button",
                    text: _ctx.$t('menu.unblock_account', [`@${__props.account.acct}`]),
                    icon: "i-ri:checkbox-circle-line",
                    command: __props.command,
                    onClick: _cache[10] || (_cache[10] = ($event: any) => (_unref(toggleBlockAccount)(_unref(relationship), __props.account)))
                  }, null, 8 /* PROPS */, ["text", "command"])),
                (_ctx.getServerName(__props.account) !== _ctx.currentServer)
                  ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                    (!_unref(relationship)?.domainBlocking)
                      ? (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                        key: 0,
                        is: "button",
                        text: _ctx.$t('menu.block_domain', [_ctx.getServerName(__props.account)]),
                        icon: "i-ri:shut-down-line",
                        command: __props.command,
                        onClick: _cache[11] || (_cache[11] = ($event: any) => (_unref(toggleBlockDomain)(_unref(relationship), __props.account)))
                      }, null, 8 /* PROPS */, ["text", "command"]))
                      : (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                        key: 1,
                        is: "button",
                        text: _ctx.$t('menu.unblock_domain', [_ctx.getServerName(__props.account)]),
                        icon: "i-ri:restart-line",
                        command: __props.command,
                        onClick: _cache[12] || (_cache[12] = ($event: any) => (_unref(toggleBlockDomain)(_unref(relationship), __props.account)))
                      }, null, 8 /* PROPS */, ["text", "command"]))
                  ], 64 /* STABLE_FRAGMENT */))
                  : _createCommentVNode("v-if", true),
                _createVNode(_component_CommonDropdownItem, {
                  is: "button",
                  text: _ctx.$t('menu.report_account', [`@${__props.account.acct}`]),
                  icon: "i-ri:flag-2-line",
                  command: __props.command,
                  onClick: _cache[13] || (_cache[13] = ($event: any) => (_ctx.openReportDialog(__props.account)))
                }, null, 8 /* PROPS */, ["text", "command"])
              ], 64 /* STABLE_FRAGMENT */))
              : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                _createVNode(_component_NuxtLink, { to: "/pinned" }, {
                  default: _withCtx(() => [
                    _createVNode(_component_CommonDropdownItem, {
                      text: _ctx.$t('account.pinned'),
                      icon: "i-ri:pushpin-line",
                      command: __props.command
                    }, null, 8 /* PROPS */, ["text", "command"])
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(_component_NuxtLink, { to: "/favourites" }, {
                  default: _withCtx(() => [
                    _createVNode(_component_CommonDropdownItem, {
                      text: _ctx.$t('account.favourites'),
                      icon: _unref(useStarFavoriteIcon) ? 'i-ri:star-line' : 'i-ri:heart-3-line',
                      command: __props.command
                    }, null, 8 /* PROPS */, ["text", "icon", "command"])
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(_component_NuxtLink, { to: "/mutes" }, {
                  default: _withCtx(() => [
                    _createVNode(_component_CommonDropdownItem, {
                      text: _ctx.$t('account.muted_users'),
                      icon: "i-ri:volume-mute-line",
                      command: __props.command
                    }, null, 8 /* PROPS */, ["text", "command"])
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(_component_NuxtLink, { to: "/blocks" }, {
                  default: _withCtx(() => [
                    _createVNode(_component_CommonDropdownItem, {
                      text: _ctx.$t('account.blocked_users'),
                      icon: "i-ri:forbid-2-line",
                      command: __props.command
                    }, null, 8 /* PROPS */, ["text", "command"])
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(_component_NuxtLink, { to: "/domain_blocks" }, {
                  default: _withCtx(() => [
                    _createVNode(_component_CommonDropdownItem, {
                      text: _ctx.$t('account.blocked_domains'),
                      icon: "i-ri:shut-down-line",
                      command: __props.command
                    }, null, 8 /* PROPS */, ["text", "command"])
                  ]),
                  _: 1 /* STABLE */
                })
              ], 64 /* STABLE_FRAGMENT */))
          ], 64 /* STABLE_FRAGMENT */))
          : _createCommentVNode("v-if", true)
      ]),
      default: _withCtx(() => [
        _createElementVNode("button", {
          flex: "",
          "gap-1": "",
          "items-center": "",
          "w-full": "",
          rounded: "",
          op75: "",
          hover: "op100 text-purple",
          group: "",
          "aria-label": _unref(t)('actions.more')
        }, [
          _createElementVNode("div", {
            "rounded-5": "",
            p2: "",
            "elk-group-hover": "bg-purple/10"
          }, [
            _hoisted_1
          ])
        ], 8 /* PROPS */, ["aria-label"])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["eager-mount"]))
}
}

})
