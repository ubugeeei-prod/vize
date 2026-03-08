import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-equal-not" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("summary", null, "raw")
import * as Misskey from 'misskey-js'
import { CodeDiff } from 'v-code-diff'
import JSON5 from 'json5'
import { i18n } from '@/i18n.js'
import MkFolder from '@/components/MkFolder.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'modlog.ModLog',
  props: {
    log: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(), _createBlock(MkFolder, null, {
      label: _withCtx(() => [
        _createElementVNode("b", {
          class: _normalizeClass({
  				[_ctx.$style.logGreen]: [
  					'createRole',
  					'addCustomEmoji',
  					'createGlobalAnnouncement',
  					'createUserAnnouncement',
  					'createAd',
  					'createInvitation',
  					'createAvatarDecoration',
  					'createSystemWebhook',
  					'createAbuseReportNotificationRecipient',
  				].includes(__props.log.type),
  				[_ctx.$style.logYellow]: [
  					'markSensitiveDriveFile',
  					'resetPassword',
  					'suspendRemoteInstance',
  				].includes(__props.log.type),
  				[_ctx.$style.logRed]: [
  					'suspend',
  					'deleteRole',
  					'deleteGlobalAnnouncement',
  					'deleteUserAnnouncement',
  					'deleteCustomEmoji',
  					'deleteNote',
  					'deleteDriveFile',
  					'deleteAd',
  					'deleteAvatarDecoration',
  					'deleteSystemWebhook',
  					'deleteAbuseReportNotificationRecipient',
  					'deleteAccount',
  					'deletePage',
  					'deleteFlash',
  					'deleteGalleryPost',
  					'deleteChatRoom',
  				].includes(__props.log.type)
  			})
        }, _toDisplayString(_unref(i18n).ts._moderationLogTypes[__props.log.type]), 3 /* TEXT, CLASS */),
        (__props.log.type === 'updateUserNote')
          ? (_openBlock(), _createElementBlock("span", { key: 0 }, ": @" + _toDisplayString(__props.log.info.userUsername) + _toDisplayString(__props.log.info.userHost ? '@' + __props.log.info.userHost : ''), 1 /* TEXT */))
          : (__props.log.type === 'suspend')
            ? (_openBlock(), _createElementBlock("span", { key: 1 }, ": @" + _toDisplayString(__props.log.info.userUsername) + _toDisplayString(__props.log.info.userHost ? '@' + __props.log.info.userHost : ''), 1 /* TEXT */))
          : (__props.log.type === 'unsuspend')
            ? (_openBlock(), _createElementBlock("span", { key: 2 }, ": @" + _toDisplayString(__props.log.info.userUsername) + _toDisplayString(__props.log.info.userHost ? '@' + __props.log.info.userHost : ''), 1 /* TEXT */))
          : (__props.log.type === 'resetPassword')
            ? (_openBlock(), _createElementBlock("span", { key: 3 }, ": @" + _toDisplayString(__props.log.info.userUsername) + _toDisplayString(__props.log.info.userHost ? '@' + __props.log.info.userHost : ''), 1 /* TEXT */))
          : (__props.log.type === 'assignRole')
            ? (_openBlock(), _createElementBlock("span", { key: 4 }, [
              _createTextVNode(": @"),
              _toDisplayString(__props.log.info.userUsername),
              _toDisplayString(__props.log.info.userHost ? '@' + __props.log.info.userHost : ''),
              _createTextVNode(),
              _hoisted_1,
              _createTextVNode(),
              _toDisplayString(__props.log.info.roleName)
            ]))
          : (__props.log.type === 'unassignRole')
            ? (_openBlock(), _createElementBlock("span", { key: 5 }, [
              _createTextVNode(": @"),
              _toDisplayString(__props.log.info.userUsername),
              _toDisplayString(__props.log.info.userHost ? '@' + __props.log.info.userHost : ''),
              _createTextVNode(),
              _hoisted_2,
              _createTextVNode(),
              _toDisplayString(__props.log.info.roleName)
            ]))
          : (__props.log.type === 'createRole')
            ? (_openBlock(), _createElementBlock("span", { key: 6 }, ": " + _toDisplayString(__props.log.info.role.name), 1 /* TEXT */))
          : (__props.log.type === 'updateRole')
            ? (_openBlock(), _createElementBlock("span", { key: 7 }, ": " + _toDisplayString(__props.log.info.before.name), 1 /* TEXT */))
          : (__props.log.type === 'deleteRole')
            ? (_openBlock(), _createElementBlock("span", { key: 8 }, ": " + _toDisplayString(__props.log.info.role.name), 1 /* TEXT */))
          : (__props.log.type === 'addCustomEmoji')
            ? (_openBlock(), _createElementBlock("span", { key: 9 }, ": " + _toDisplayString(__props.log.info.emoji.name), 1 /* TEXT */))
          : (__props.log.type === 'updateCustomEmoji')
            ? (_openBlock(), _createElementBlock("span", { key: 10 }, ": " + _toDisplayString(__props.log.info.before.name), 1 /* TEXT */))
          : (__props.log.type === 'deleteCustomEmoji')
            ? (_openBlock(), _createElementBlock("span", { key: 11 }, ": " + _toDisplayString(__props.log.info.emoji.name), 1 /* TEXT */))
          : (__props.log.type === 'markSensitiveDriveFile')
            ? (_openBlock(), _createElementBlock("span", { key: 12 }, ": @" + _toDisplayString(__props.log.info.fileUserUsername) + _toDisplayString(__props.log.info.fileUserHost ? '@' + __props.log.info.fileUserHost : ''), 1 /* TEXT */))
          : (__props.log.type === 'unmarkSensitiveDriveFile')
            ? (_openBlock(), _createElementBlock("span", { key: 13 }, ": @" + _toDisplayString(__props.log.info.fileUserUsername) + _toDisplayString(__props.log.info.fileUserHost ? '@' + __props.log.info.fileUserHost : ''), 1 /* TEXT */))
          : (__props.log.type === 'suspendRemoteInstance')
            ? (_openBlock(), _createElementBlock("span", { key: 14 }, ": " + _toDisplayString(__props.log.info.host), 1 /* TEXT */))
          : (__props.log.type === 'unsuspendRemoteInstance')
            ? (_openBlock(), _createElementBlock("span", { key: 15 }, ": " + _toDisplayString(__props.log.info.host), 1 /* TEXT */))
          : (__props.log.type === 'createGlobalAnnouncement')
            ? (_openBlock(), _createElementBlock("span", { key: 16 }, ": " + _toDisplayString(__props.log.info.announcement.title), 1 /* TEXT */))
          : (__props.log.type === 'updateGlobalAnnouncement')
            ? (_openBlock(), _createElementBlock("span", { key: 17 }, ": " + _toDisplayString(__props.log.info.before.title), 1 /* TEXT */))
          : (__props.log.type === 'deleteGlobalAnnouncement')
            ? (_openBlock(), _createElementBlock("span", { key: 18 }, ": " + _toDisplayString(__props.log.info.announcement.title), 1 /* TEXT */))
          : (__props.log.type === 'createUserAnnouncement')
            ? (_openBlock(), _createElementBlock("span", { key: 19 }, ": @" + _toDisplayString(__props.log.info.userUsername) + _toDisplayString(__props.log.info.userHost ? '@' + __props.log.info.userHost : ''), 1 /* TEXT */))
          : (__props.log.type === 'updateUserAnnouncement')
            ? (_openBlock(), _createElementBlock("span", { key: 20 }, ": @" + _toDisplayString(__props.log.info.userUsername) + _toDisplayString(__props.log.info.userHost ? '@' + __props.log.info.userHost : ''), 1 /* TEXT */))
          : (__props.log.type === 'deleteUserAnnouncement')
            ? (_openBlock(), _createElementBlock("span", { key: 21 }, ": @" + _toDisplayString(__props.log.info.userUsername) + _toDisplayString(__props.log.info.userHost ? '@' + __props.log.info.userHost : ''), 1 /* TEXT */))
          : (__props.log.type === 'deleteNote')
            ? (_openBlock(), _createElementBlock("span", { key: 22 }, ": @" + _toDisplayString(__props.log.info.noteUserUsername) + _toDisplayString(__props.log.info.noteUserHost ? '@' + __props.log.info.noteUserHost : ''), 1 /* TEXT */))
          : (__props.log.type === 'deleteDriveFile')
            ? (_openBlock(), _createElementBlock("span", { key: 23 }, ": @" + _toDisplayString(__props.log.info.fileUserUsername) + _toDisplayString(__props.log.info.fileUserHost ? '@' + __props.log.info.fileUserHost : ''), 1 /* TEXT */))
          : (__props.log.type === 'createAvatarDecoration')
            ? (_openBlock(), _createElementBlock("span", { key: 24 }, ": " + _toDisplayString(__props.log.info.avatarDecoration.name), 1 /* TEXT */))
          : (__props.log.type === 'updateAvatarDecoration')
            ? (_openBlock(), _createElementBlock("span", { key: 25 }, ": " + _toDisplayString(__props.log.info.before.name), 1 /* TEXT */))
          : (__props.log.type === 'deleteAvatarDecoration')
            ? (_openBlock(), _createElementBlock("span", { key: 26 }, ": " + _toDisplayString(__props.log.info.avatarDecoration.name), 1 /* TEXT */))
          : (__props.log.type === 'createSystemWebhook')
            ? (_openBlock(), _createElementBlock("span", { key: 27 }, ": " + _toDisplayString(__props.log.info.webhook.name), 1 /* TEXT */))
          : (__props.log.type === 'updateSystemWebhook')
            ? (_openBlock(), _createElementBlock("span", { key: 28 }, ": " + _toDisplayString(__props.log.info.before.name), 1 /* TEXT */))
          : (__props.log.type === 'deleteSystemWebhook')
            ? (_openBlock(), _createElementBlock("span", { key: 29 }, ": " + _toDisplayString(__props.log.info.webhook.name), 1 /* TEXT */))
          : (__props.log.type === 'createAbuseReportNotificationRecipient')
            ? (_openBlock(), _createElementBlock("span", { key: 30 }, ": " + _toDisplayString(__props.log.info.recipient.name), 1 /* TEXT */))
          : (__props.log.type === 'updateAbuseReportNotificationRecipient')
            ? (_openBlock(), _createElementBlock("span", { key: 31 }, ": " + _toDisplayString(__props.log.info.before.name), 1 /* TEXT */))
          : (__props.log.type === 'deleteAbuseReportNotificationRecipient')
            ? (_openBlock(), _createElementBlock("span", { key: 32 }, ": " + _toDisplayString(__props.log.info.recipient.name), 1 /* TEXT */))
          : (__props.log.type === 'deleteAccount')
            ? (_openBlock(), _createElementBlock("span", { key: 33 }, ": @" + _toDisplayString(__props.log.info.userUsername) + _toDisplayString(__props.log.info.userHost ? '@' + __props.log.info.userHost : ''), 1 /* TEXT */))
          : (__props.log.type === 'deletePage')
            ? (_openBlock(), _createElementBlock("span", { key: 34 }, ": @" + _toDisplayString(__props.log.info.pageUserUsername), 1 /* TEXT */))
          : (__props.log.type === 'deleteFlash')
            ? (_openBlock(), _createElementBlock("span", { key: 35 }, ": @" + _toDisplayString(__props.log.info.flashUserUsername), 1 /* TEXT */))
          : (__props.log.type === 'deleteGalleryPost')
            ? (_openBlock(), _createElementBlock("span", { key: 36 }, ": @" + _toDisplayString(__props.log.info.postUserUsername), 1 /* TEXT */))
          : (__props.log.type === 'deleteChatRoom')
            ? (_openBlock(), _createElementBlock("span", { key: 37 }, ": @" + _toDisplayString(__props.log.info.room.name), 1 /* TEXT */))
          : _createCommentVNode("v-if", true)
      ]),
      icon: _withCtx(() => [
        (__props.log.type === 'updateServerSettings')
          ? (_openBlock(), _createElementBlock("i", {
            key: 0,
            class: "ti ti-settings"
          }))
          : (__props.log.type === 'updateUserNote')
            ? (_openBlock(), _createElementBlock("i", {
              key: 1,
              class: "ti ti-pencil"
            }))
          : (__props.log.type === 'suspend')
            ? (_openBlock(), _createElementBlock("i", {
              key: 2,
              class: "ti ti-user-x"
            }))
          : (__props.log.type === 'unsuspend')
            ? (_openBlock(), _createElementBlock("i", {
              key: 3,
              class: "ti ti-user-check"
            }))
          : (__props.log.type === 'resetPassword')
            ? (_openBlock(), _createElementBlock("i", {
              key: 4,
              class: "ti ti-key"
            }))
          : (__props.log.type === 'assignRole')
            ? (_openBlock(), _createElementBlock("i", {
              key: 5,
              class: "ti ti-user-plus"
            }))
          : (__props.log.type === 'unassignRole')
            ? (_openBlock(), _createElementBlock("i", {
              key: 6,
              class: "ti ti-user-minus"
            }))
          : (__props.log.type === 'createRole')
            ? (_openBlock(), _createElementBlock("i", {
              key: 7,
              class: "ti ti-plus"
            }))
          : (__props.log.type === 'updateRole')
            ? (_openBlock(), _createElementBlock("i", {
              key: 8,
              class: "ti ti-pencil"
            }))
          : (__props.log.type === 'deleteRole')
            ? (_openBlock(), _createElementBlock("i", {
              key: 9,
              class: "ti ti-trash"
            }))
          : (__props.log.type === 'addCustomEmoji')
            ? (_openBlock(), _createElementBlock("i", {
              key: 10,
              class: "ti ti-plus"
            }))
          : (__props.log.type === 'updateCustomEmoji')
            ? (_openBlock(), _createElementBlock("i", {
              key: 11,
              class: "ti ti-pencil"
            }))
          : (__props.log.type === 'deleteCustomEmoji')
            ? (_openBlock(), _createElementBlock("i", {
              key: 12,
              class: "ti ti-trash"
            }))
          : (__props.log.type === 'markSensitiveDriveFile')
            ? (_openBlock(), _createElementBlock("i", {
              key: 13,
              class: "ti ti-eye-exclamation"
            }))
          : (__props.log.type === 'unmarkSensitiveDriveFile')
            ? (_openBlock(), _createElementBlock("i", {
              key: 14,
              class: "ti ti-eye"
            }))
          : (__props.log.type === 'suspendRemoteInstance')
            ? (_openBlock(), _createElementBlock("i", {
              key: 15,
              class: "ti ti-x"
            }))
          : (__props.log.type === 'unsuspendRemoteInstance')
            ? (_openBlock(), _createElementBlock("i", {
              key: 16,
              class: "ti ti-check"
            }))
          : (__props.log.type === 'createGlobalAnnouncement')
            ? (_openBlock(), _createElementBlock("i", {
              key: 17,
              class: "ti ti-plus"
            }))
          : (__props.log.type === 'updateGlobalAnnouncement')
            ? (_openBlock(), _createElementBlock("i", {
              key: 18,
              class: "ti ti-pencil"
            }))
          : (__props.log.type === 'deleteGlobalAnnouncement')
            ? (_openBlock(), _createElementBlock("i", {
              key: 19,
              class: "ti ti-trash"
            }))
          : (__props.log.type === 'createUserAnnouncement')
            ? (_openBlock(), _createElementBlock("i", {
              key: 20,
              class: "ti ti-plus"
            }))
          : (__props.log.type === 'updateUserAnnouncement')
            ? (_openBlock(), _createElementBlock("i", {
              key: 21,
              class: "ti ti-pencil"
            }))
          : (__props.log.type === 'deleteUserAnnouncement')
            ? (_openBlock(), _createElementBlock("i", {
              key: 22,
              class: "ti ti-trash"
            }))
          : (__props.log.type === 'deleteNote')
            ? (_openBlock(), _createElementBlock("i", {
              key: 23,
              class: "ti ti-trash"
            }))
          : (__props.log.type === 'deleteDriveFile')
            ? (_openBlock(), _createElementBlock("i", {
              key: 24,
              class: "ti ti-trash"
            }))
          : (__props.log.type === 'createAd')
            ? (_openBlock(), _createElementBlock("i", {
              key: 25,
              class: "ti ti-plus"
            }))
          : (__props.log.type === 'updateAd')
            ? (_openBlock(), _createElementBlock("i", {
              key: 26,
              class: "ti ti-pencil"
            }))
          : (__props.log.type === 'deleteAd')
            ? (_openBlock(), _createElementBlock("i", {
              key: 27,
              class: "ti ti-trash"
            }))
          : (__props.log.type === 'createAvatarDecoration')
            ? (_openBlock(), _createElementBlock("i", {
              key: 28,
              class: "ti ti-plus"
            }))
          : (__props.log.type === 'updateAvatarDecoration')
            ? (_openBlock(), _createElementBlock("i", {
              key: 29,
              class: "ti ti-pencil"
            }))
          : (__props.log.type === 'deleteAvatarDecoration')
            ? (_openBlock(), _createElementBlock("i", {
              key: 30,
              class: "ti ti-trash"
            }))
          : (__props.log.type === 'createSystemWebhook')
            ? (_openBlock(), _createElementBlock("i", {
              key: 31,
              class: "ti ti-plus"
            }))
          : (__props.log.type === 'updateSystemWebhook')
            ? (_openBlock(), _createElementBlock("i", {
              key: 32,
              class: "ti ti-pencil"
            }))
          : (__props.log.type === 'deleteSystemWebhook')
            ? (_openBlock(), _createElementBlock("i", {
              key: 33,
              class: "ti ti-trash"
            }))
          : (__props.log.type === 'createAbuseReportNotificationRecipient')
            ? (_openBlock(), _createElementBlock("i", {
              key: 34,
              class: "ti ti-plus"
            }))
          : (__props.log.type === 'updateAbuseReportNotificationRecipient')
            ? (_openBlock(), _createElementBlock("i", {
              key: 35,
              class: "ti ti-pencil"
            }))
          : (__props.log.type === 'deleteAbuseReportNotificationRecipient')
            ? (_openBlock(), _createElementBlock("i", {
              key: 36,
              class: "ti ti-trash"
            }))
          : (__props.log.type === 'deleteAccount')
            ? (_openBlock(), _createElementBlock("i", {
              key: 37,
              class: "ti ti-trash"
            }))
          : (__props.log.type === 'deletePage')
            ? (_openBlock(), _createElementBlock("i", {
              key: 38,
              class: "ti ti-trash"
            }))
          : (__props.log.type === 'deleteFlash')
            ? (_openBlock(), _createElementBlock("i", {
              key: 39,
              class: "ti ti-trash"
            }))
          : (__props.log.type === 'deleteGalleryPost')
            ? (_openBlock(), _createElementBlock("i", {
              key: 40,
              class: "ti ti-trash"
            }))
          : (__props.log.type === 'deleteChatRoom')
            ? (_openBlock(), _createElementBlock("i", {
              key: 41,
              class: "ti ti-trash"
            }))
          : _createCommentVNode("v-if", true)
      ]),
      suffix: _withCtx(() => [
        _createVNode(_component_MkTime, { time: __props.log.createdAt }, null, 8 /* PROPS */, ["time"])
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", null, [
          _createElementVNode("div", { style: "display: flex; gap: var(--MI-margin); flex-wrap: wrap;" }, [
            _createElementVNode("div", { style: "flex: 1;" }, [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.moderator) + ": ", 1 /* TEXT */),
              _createVNode(_component_MkA, {
                to: `/admin/user/${__props.log.userId}`,
                class: "_link"
              }, {
                default: _withCtx(() => [
                  _createTextVNode("@"),
                  _createTextVNode(_toDisplayString(__props.log.user?.username), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to"])
            ]),
            _createElementVNode("div", { style: "flex: 1;" }, [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.dateAndTime) + ": ", 1 /* TEXT */),
              _createVNode(_component_MkTime, {
                time: __props.log.createdAt,
                mode: "detail"
              }, null, 8 /* PROPS */, ["time"])
            ])
          ]),
          (__props.log.type === 'updateServerSettings')
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(_ctx.$style.diff)
            }, [
              _createVNode(CodeDiff, {
                context: 5,
                hideHeader: true,
                oldString: JSON5.stringify(__props.log.info.before, null, '\t'),
                newString: JSON5.stringify(__props.log.info.after, null, '\t'),
                language: "javascript",
                maxHeight: "300px"
              }, null, 8 /* PROPS */, ["context", "hideHeader", "oldString", "newString"])
            ]))
            : (__props.log.type === 'updateUserNote')
              ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts.user) + ": " + _toDisplayString(__props.log.info.userId), 1 /* TEXT */),
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.diff)
                }, [
                  _createVNode(CodeDiff, {
                    context: 5,
                    hideHeader: true,
                    oldString: __props.log.info.before ?? '',
                    newString: __props.log.info.after ?? '',
                    maxHeight: "300px"
                  }, null, 8 /* PROPS */, ["context", "hideHeader", "oldString", "newString"])
                ])
              ], 64 /* STABLE_FRAGMENT */))
            : (__props.log.type === 'suspend')
              ? (_openBlock(), _createElementBlock("div", { key: 2 }, [
                _toDisplayString(_unref(i18n).ts.user),
                _createTextVNode(": "),
                _createVNode(_component_MkA, {
                  to: `/admin/user/${__props.log.info.userId}`,
                  class: "_link"
                }, {
                  default: _withCtx(() => [
                    _createTextVNode("@"),
                    _createTextVNode(_toDisplayString(__props.log.info.userUsername), 1 /* TEXT */),
                    _createTextVNode(_toDisplayString(__props.log.info.userHost ? '@' + __props.log.info.userHost : ''), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["to"])
              ]))
            : (__props.log.type === 'unsuspend')
              ? (_openBlock(), _createElementBlock("div", { key: 3 }, [
                _toDisplayString(_unref(i18n).ts.user),
                _createTextVNode(": "),
                _createVNode(_component_MkA, {
                  to: `/admin/user/${__props.log.info.userId}`,
                  class: "_link"
                }, {
                  default: _withCtx(() => [
                    _createTextVNode("@"),
                    _createTextVNode(_toDisplayString(__props.log.info.userUsername), 1 /* TEXT */),
                    _createTextVNode(_toDisplayString(__props.log.info.userHost ? '@' + __props.log.info.userHost : ''), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["to"])
              ]))
            : (__props.log.type === 'updateRole')
              ? (_openBlock(), _createElementBlock("div", {
                key: 4,
                class: _normalizeClass(_ctx.$style.diff)
              }, [
                _createVNode(CodeDiff, {
                  context: 5,
                  hideHeader: true,
                  oldString: JSON5.stringify(__props.log.info.before, null, '\t'),
                  newString: JSON5.stringify(__props.log.info.after, null, '\t'),
                  language: "javascript",
                  maxHeight: "300px"
                }, null, 8 /* PROPS */, ["context", "hideHeader", "oldString", "newString"])
              ]))
            : (__props.log.type === 'assignRole')
              ? (_openBlock(), _createElementBlock(_Fragment, { key: 5 }, [
                _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts.user) + ": " + _toDisplayString(__props.log.info.userId), 1 /* TEXT */),
                _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts.role) + ": " + _toDisplayString(__props.log.info.roleName) + " [" + _toDisplayString(__props.log.info.roleId) + "]", 1 /* TEXT */)
              ], 64 /* STABLE_FRAGMENT */))
            : (__props.log.type === 'unassignRole')
              ? (_openBlock(), _createElementBlock(_Fragment, { key: 6 }, [
                _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts.user) + ": " + _toDisplayString(__props.log.info.userId), 1 /* TEXT */),
                _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts.role) + ": " + _toDisplayString(__props.log.info.roleName) + " [" + _toDisplayString(__props.log.info.roleId) + "]", 1 /* TEXT */)
              ], 64 /* STABLE_FRAGMENT */))
            : (__props.log.type === 'updateCustomEmoji')
              ? (_openBlock(), _createElementBlock(_Fragment, { key: 7 }, [
                _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts.emoji) + ": " + _toDisplayString(__props.log.info.emojiId), 1 /* TEXT */),
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.diff)
                }, [
                  _createVNode(CodeDiff, {
                    context: 5,
                    hideHeader: true,
                    oldString: JSON5.stringify(__props.log.info.before, null, '\t'),
                    newString: JSON5.stringify(__props.log.info.after, null, '\t'),
                    language: "javascript",
                    maxHeight: "300px"
                  }, null, 8 /* PROPS */, ["context", "hideHeader", "oldString", "newString"])
                ])
              ], 64 /* STABLE_FRAGMENT */))
            : (__props.log.type === 'updateAd')
              ? (_openBlock(), _createElementBlock("div", {
                key: 8,
                class: _normalizeClass(_ctx.$style.diff)
              }, [
                _createVNode(CodeDiff, {
                  context: 5,
                  hideHeader: true,
                  oldString: JSON5.stringify(__props.log.info.before, null, '\t'),
                  newString: JSON5.stringify(__props.log.info.after, null, '\t'),
                  language: "javascript",
                  maxHeight: "300px"
                }, null, 8 /* PROPS */, ["context", "hideHeader", "oldString", "newString"])
              ]))
            : (__props.log.type === 'updateGlobalAnnouncement')
              ? (_openBlock(), _createElementBlock("div", {
                key: 9,
                class: _normalizeClass(_ctx.$style.diff)
              }, [
                _createVNode(CodeDiff, {
                  context: 5,
                  hideHeader: true,
                  oldString: JSON5.stringify(__props.log.info.before, null, '\t'),
                  newString: JSON5.stringify(__props.log.info.after, null, '\t'),
                  language: "javascript",
                  maxHeight: "300px"
                }, null, 8 /* PROPS */, ["context", "hideHeader", "oldString", "newString"])
              ]))
            : (__props.log.type === 'updateUserAnnouncement')
              ? (_openBlock(), _createElementBlock("div", {
                key: 10,
                class: _normalizeClass(_ctx.$style.diff)
              }, [
                _createVNode(CodeDiff, {
                  context: 5,
                  hideHeader: true,
                  oldString: JSON5.stringify(__props.log.info.before, null, '\t'),
                  newString: JSON5.stringify(__props.log.info.after, null, '\t'),
                  language: "javascript",
                  maxHeight: "300px"
                }, null, 8 /* PROPS */, ["context", "hideHeader", "oldString", "newString"])
              ]))
            : (__props.log.type === 'updateAvatarDecoration')
              ? (_openBlock(), _createElementBlock("div", {
                key: 11,
                class: _normalizeClass(_ctx.$style.diff)
              }, [
                _createVNode(CodeDiff, {
                  context: 5,
                  hideHeader: true,
                  oldString: JSON5.stringify(__props.log.info.before, null, '\t'),
                  newString: JSON5.stringify(__props.log.info.after, null, '\t'),
                  language: "javascript",
                  maxHeight: "300px"
                }, null, 8 /* PROPS */, ["context", "hideHeader", "oldString", "newString"])
              ]))
            : (__props.log.type === 'updateRemoteInstanceNote')
              ? (_openBlock(), _createElementBlock("div", {
                key: 12,
                class: _normalizeClass(_ctx.$style.diff)
              }, [
                _createVNode(CodeDiff, {
                  context: 5,
                  hideHeader: true,
                  oldString: __props.log.info.before ?? '',
                  newString: __props.log.info.after ?? '',
                  maxHeight: "300px"
                }, null, 8 /* PROPS */, ["context", "hideHeader", "oldString", "newString"])
              ]))
            : (__props.log.type === 'updateSystemWebhook')
              ? (_openBlock(), _createElementBlock("div", {
                key: 13,
                class: _normalizeClass(_ctx.$style.diff)
              }, [
                _createVNode(CodeDiff, {
                  context: 5,
                  hideHeader: true,
                  oldString: JSON5.stringify(__props.log.info.before, null, '\t'),
                  newString: JSON5.stringify(__props.log.info.after, null, '\t'),
                  language: "javascript",
                  maxHeight: "300px"
                }, null, 8 /* PROPS */, ["context", "hideHeader", "oldString", "newString"])
              ]))
            : (__props.log.type === 'updateAbuseReportNotificationRecipient')
              ? (_openBlock(), _createElementBlock("div", {
                key: 14,
                class: _normalizeClass(_ctx.$style.diff)
              }, [
                _createVNode(CodeDiff, {
                  context: 5,
                  hideHeader: true,
                  oldString: JSON5.stringify(__props.log.info.before, null, '\t'),
                  newString: JSON5.stringify(__props.log.info.after, null, '\t'),
                  language: "javascript",
                  maxHeight: "300px"
                }, null, 8 /* PROPS */, ["context", "hideHeader", "oldString", "newString"])
              ]))
            : (__props.log.type === 'updateAbuseReportNote')
              ? (_openBlock(), _createElementBlock("div", {
                key: 15,
                class: _normalizeClass(_ctx.$style.diff)
              }, [
                _createVNode(CodeDiff, {
                  context: 5,
                  hideHeader: true,
                  oldString: __props.log.info.before ?? '',
                  newString: __props.log.info.after ?? '',
                  maxHeight: "300px"
                }, null, 8 /* PROPS */, ["context", "hideHeader", "oldString", "newString"])
              ]))
            : (__props.log.type === 'updateProxyAccountDescription')
              ? (_openBlock(), _createElementBlock("div", {
                key: 16,
                class: _normalizeClass(_ctx.$style.diff)
              }, [
                _createVNode(CodeDiff, {
                  context: 5,
                  hideHeader: true,
                  oldString: __props.log.info.before ?? '',
                  newString: __props.log.info.after ?? '',
                  maxHeight: "300px"
                }, null, 8 /* PROPS */, ["context", "hideHeader", "oldString", "newString"])
              ]))
            : _createCommentVNode("v-if", true),
          _createElementVNode("details", null, [
            _hoisted_3,
            _createElementVNode("pre", null, _toDisplayString(JSON5.stringify(__props.log, null, '\t')), 1 /* TEXT */)
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
