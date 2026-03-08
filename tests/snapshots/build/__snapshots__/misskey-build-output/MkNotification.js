import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-heart", style: "line-height: 1;" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus", style: "line-height: 1;" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-repeat", style: "line-height: 1;" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import MkReactionIcon from '@/components/MkReactionIcon.vue'
import MkButton from '@/components/MkButton.vue'
import { getNoteSummary } from '@/utility/get-note-summary.js'
import { notePage } from '@/filters/note.js'
import { userPage } from '@/filters/user.js'
import { i18n } from '@/i18n.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { ensureSignin } from '@/i.js'

type ExportCompletedNotification = Misskey.entities.Notification & { type: 'exportCompleted' };

export default /*@__PURE__*/_defineComponent({
  __name: 'MkNotification',
  props: {
    notification: { type: null, required: true },
    withTime: { type: Boolean, required: false, default: false },
    full: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {

const props = __props
const $i = ensureSignin();
const exportEntityName = {
	antenna: i18n.ts.antennas,
	blocking: i18n.ts.blockedUsers,
	clip: i18n.ts.clips,
	customEmoji: i18n.ts.customEmojis,
	favorite: i18n.ts.favorites,
	following: i18n.ts.following,
	muting: i18n.ts.mutedUsers,
	note: i18n.ts.notes,
	userList: i18n.ts.lists,
} as const satisfies Record<ExportCompletedNotification['exportedEntity'], string>;
const followRequestDone = ref(false);
const acceptFollowRequest = () => {
	if (!('user' in props.notification)) return;
	followRequestDone.value = true;
	misskeyApi('following/requests/accept', { userId: props.notification.user.id });
};
const rejectFollowRequest = () => {
	if (!('user' in props.notification)) return;
	followRequestDone.value = true;
	misskeyApi('following/requests/reject', { userId: props.notification.user.id });
};
function getActualReactedUsersCount(notification: Misskey.entities.Notification) {
	if (notification.type !== 'reaction:grouped') return 0;
	return new Set(notification.reactions.map((reaction) => reaction.user.id)).size;
}

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_MkA = _resolveComponent("MkA")
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_Mfm = _resolveComponent("Mfm")
  const _directive_user_preview = _resolveDirective("user-preview")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.head)
      }, [ (['pollEnded', 'note'].includes(__props.notification.type) && 'note' in __props.notification) ? (_openBlock(), _createBlock(_component_MkAvatar, {
            key: 0,
            class: _normalizeClass(_ctx.$style.icon),
            user: __props.notification.note.user,
            link: "",
            preview: ""
          }, null, 8 /* PROPS */, ["user"])) : (['roleAssigned', 'achievementEarned', 'exportCompleted', 'login', 'createToken', 'scheduledNotePosted', 'scheduledNotePostFailed'].includes(__props.notification.type)) ? (_openBlock(), _createBlock(_component_MkAvatar, {
              key: 1,
              class: _normalizeClass(_ctx.$style.icon),
              user: _unref($i),
              link: "",
              preview: ""
            }, null, 8 /* PROPS */, ["user"])) : (__props.notification.type === 'reaction:grouped' && __props.notification.note.reactionAcceptance === 'likeOnly') ? (_openBlock(), _createElementBlock("div", {
              key: 2,
              class: _normalizeClass([_ctx.$style.icon, _ctx.$style.icon_reactionGroupHeart])
            }, [ _hoisted_1 ])) : (__props.notification.type === 'reaction:grouped') ? (_openBlock(), _createElementBlock("div", {
              key: 3,
              class: _normalizeClass([_ctx.$style.icon, _ctx.$style.icon_reactionGroup])
            }, [ _hoisted_2 ])) : (__props.notification.type === 'renote:grouped') ? (_openBlock(), _createElementBlock("div", {
              key: 4,
              class: _normalizeClass([_ctx.$style.icon, _ctx.$style.icon_renoteGroup])
            }, [ _hoisted_3 ])) : ('user' in __props.notification) ? (_openBlock(), _createBlock(_component_MkAvatar, {
              key: 5,
              class: _normalizeClass(_ctx.$style.icon),
              user: __props.notification.user,
              link: "",
              preview: ""
            }, null, 8 /* PROPS */, ["user"])) : ('icon' in __props.notification && __props.notification.icon != null) ? (_openBlock(), _createElementBlock("img", {
              key: 6,
              class: _normalizeClass([_ctx.$style.icon, _ctx.$style.icon_app]),
              src: __props.notification.icon,
              alt: ""
            })) : _createCommentVNode("v-if", true), _createElementVNode("div", {
          class: _normalizeClass([_ctx.$style.subIcon, {
  				[_ctx.$style.t_follow]: __props.notification.type === 'follow',
  				[_ctx.$style.t_followRequestAccepted]: __props.notification.type === 'followRequestAccepted',
  				[_ctx.$style.t_receiveFollowRequest]: __props.notification.type === 'receiveFollowRequest',
  				[_ctx.$style.t_renote]: __props.notification.type === 'renote',
  				[_ctx.$style.t_reply]: __props.notification.type === 'reply',
  				[_ctx.$style.t_mention]: __props.notification.type === 'mention',
  				[_ctx.$style.t_quote]: __props.notification.type === 'quote',
  				[_ctx.$style.t_pollEnded]: __props.notification.type === 'pollEnded',
  				[_ctx.$style.t_scheduledNotePosted]: __props.notification.type === 'scheduledNotePosted',
  				[_ctx.$style.t_scheduledNotePostFailed]: __props.notification.type === 'scheduledNotePostFailed',
  				[_ctx.$style.t_achievementEarned]: __props.notification.type === 'achievementEarned',
  				[_ctx.$style.t_exportCompleted]: __props.notification.type === 'exportCompleted',
  				[_ctx.$style.t_login]: __props.notification.type === 'login',
  				[_ctx.$style.t_createToken]: __props.notification.type === 'createToken',
  				[_ctx.$style.t_chatRoomInvitationReceived]: __props.notification.type === 'chatRoomInvitationReceived',
  				[_ctx.$style.t_roleAssigned]: __props.notification.type === 'roleAssigned' && __props.notification.role.iconUrl == null,
  			}])
        }, [ (__props.notification.type === 'follow') ? (_openBlock(), _createElementBlock("i", {
              key: 0,
              class: "ti ti-plus"
            })) : (__props.notification.type === 'receiveFollowRequest') ? (_openBlock(), _createElementBlock("i", {
                key: 1,
                class: "ti ti-clock"
              })) : (__props.notification.type === 'followRequestAccepted') ? (_openBlock(), _createElementBlock("i", {
                key: 2,
                class: "ti ti-check"
              })) : (__props.notification.type === 'renote') ? (_openBlock(), _createElementBlock("i", {
                key: 3,
                class: "ti ti-repeat"
              })) : (__props.notification.type === 'reply') ? (_openBlock(), _createElementBlock("i", {
                key: 4,
                class: "ti ti-arrow-back-up"
              })) : (__props.notification.type === 'mention') ? (_openBlock(), _createElementBlock("i", {
                key: 5,
                class: "ti ti-at"
              })) : (__props.notification.type === 'quote') ? (_openBlock(), _createElementBlock("i", {
                key: 6,
                class: "ti ti-quote"
              })) : (__props.notification.type === 'pollEnded') ? (_openBlock(), _createElementBlock("i", {
                key: 7,
                class: "ti ti-chart-arrows"
              })) : (__props.notification.type === 'scheduledNotePosted') ? (_openBlock(), _createElementBlock("i", {
                key: 8,
                class: "ti ti-send"
              })) : (__props.notification.type === 'scheduledNotePostFailed') ? (_openBlock(), _createElementBlock("i", {
                key: 9,
                class: "ti ti-alert-triangle"
              })) : (__props.notification.type === 'achievementEarned') ? (_openBlock(), _createElementBlock("i", {
                key: 10,
                class: "ti ti-medal"
              })) : (__props.notification.type === 'exportCompleted') ? (_openBlock(), _createElementBlock("i", {
                key: 11,
                class: "ti ti-archive"
              })) : (__props.notification.type === 'login') ? (_openBlock(), _createElementBlock("i", {
                key: 12,
                class: "ti ti-login-2"
              })) : (__props.notification.type === 'createToken') ? (_openBlock(), _createElementBlock("i", {
                key: 13,
                class: "ti ti-key"
              })) : (__props.notification.type === 'chatRoomInvitationReceived') ? (_openBlock(), _createElementBlock("i", {
                key: 14,
                class: "ti ti-messages"
              })) : (__props.notification.type === 'roleAssigned') ? (_openBlock(), _createElementBlock(_Fragment, { key: 15 }, [ (__props.notification.role.iconUrl) ? (_openBlock(), _createElementBlock("img", {
                    key: 0,
                    style: "height: 1.3em; vertical-align: -22%;",
                    src: __props.notification.role.iconUrl,
                    alt: ""
                  })) : (_openBlock(), _createElementBlock("i", {
                    key: 1,
                    class: "ti ti-badges"
                  })) ], 64 /* STABLE_FRAGMENT */)) : (__props.notification.type === 'reaction') ? (_openBlock(), _createBlock(MkReactionIcon, {
                key: 16,
                withTooltip: true,
                reaction: __props.notification.reaction.replace(/^:(\w+):$/, ':$1@.:'),
                noStyle: true,
                style: "width: 100%; height: 100% !important; object-fit: contain;"
              }, null, 8 /* PROPS */, ["withTooltip", "reaction", "noStyle"])) : _createCommentVNode("v-if", true) ], 2 /* CLASS */) ]), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.tail)
      }, [ _createElementVNode("header", {
          class: _normalizeClass(_ctx.$style.header)
        }, [ (__props.notification.type === 'pollEnded') ? (_openBlock(), _createElementBlock("span", { key: 0 }, _toDisplayString(_unref(i18n).ts._notification.pollEnded), 1 /* TEXT */)) : (__props.notification.type === 'scheduledNotePosted') ? (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(_unref(i18n).ts._notification.scheduledNotePosted), 1 /* TEXT */)) : (__props.notification.type === 'scheduledNotePostFailed') ? (_openBlock(), _createElementBlock("span", { key: 2 }, _toDisplayString(_unref(i18n).ts._notification.scheduledNotePostFailed), 1 /* TEXT */)) : (__props.notification.type === 'note') ? (_openBlock(), _createElementBlock("span", { key: 3 }, [ _toDisplayString(_unref(i18n).ts._notification.newNote), _createTextVNode(": "), _createVNode(_component_MkUserName, { user: __props.notification.note.user }, null, 8 /* PROPS */, ["user"]) ])) : (__props.notification.type === 'roleAssigned') ? (_openBlock(), _createElementBlock("span", { key: 4 }, _toDisplayString(_unref(i18n).ts._notification.roleAssigned), 1 /* TEXT */)) : (__props.notification.type === 'chatRoomInvitationReceived') ? (_openBlock(), _createElementBlock("span", { key: 5 }, _toDisplayString(_unref(i18n).ts._notification.chatRoomInvitationReceived), 1 /* TEXT */)) : (__props.notification.type === 'achievementEarned') ? (_openBlock(), _createElementBlock("span", { key: 6 }, _toDisplayString(_unref(i18n).ts._notification.achievementEarned), 1 /* TEXT */)) : (__props.notification.type === 'login') ? (_openBlock(), _createElementBlock("span", { key: 7 }, _toDisplayString(_unref(i18n).ts._notification.login), 1 /* TEXT */)) : (__props.notification.type === 'createToken') ? (_openBlock(), _createElementBlock("span", { key: 8 }, _toDisplayString(_unref(i18n).ts._notification.createToken), 1 /* TEXT */)) : (__props.notification.type === 'test') ? (_openBlock(), _createElementBlock("span", { key: 9 }, _toDisplayString(_unref(i18n).ts._notification.testNotification), 1 /* TEXT */)) : (__props.notification.type === 'exportCompleted') ? (_openBlock(), _createElementBlock("span", { key: 10 }, _toDisplayString(_unref(i18n).tsx._notification.exportOfXCompleted({ x: _unref(exportEntityName)[__props.notification.exportedEntity] })), 1 /* TEXT */)) : (__props.notification.type === 'follow' || __props.notification.type === 'mention' || __props.notification.type === 'reply' || __props.notification.type === 'renote' || __props.notification.type === 'quote' || __props.notification.type === 'reaction' || __props.notification.type === 'receiveFollowRequest' || __props.notification.type === 'followRequestAccepted') ? (_openBlock(), _createBlock(_component_MkA, {
                key: 11,
                class: _normalizeClass(_ctx.$style.headerName),
                to: _unref(userPage)(__props.notification.user)
              }, {
                default: _withCtx(() => [
                  _createVNode(_component_MkUserName, { user: __props.notification.user }, null, 8 /* PROPS */, ["user"])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to"])) : (__props.notification.type === 'reaction:grouped' && __props.notification.note.reactionAcceptance === 'likeOnly') ? (_openBlock(), _createElementBlock("span", { key: 12 }, _toDisplayString(_unref(i18n).tsx._notification.likedBySomeUsers({ n: getActualReactedUsersCount(__props.notification) })), 1 /* TEXT */)) : (__props.notification.type === 'reaction:grouped') ? (_openBlock(), _createElementBlock("span", { key: 13 }, _toDisplayString(_unref(i18n).tsx._notification.reactedBySomeUsers({ n: getActualReactedUsersCount(__props.notification) })), 1 /* TEXT */)) : (__props.notification.type === 'renote:grouped') ? (_openBlock(), _createElementBlock("span", { key: 14 }, _toDisplayString(_unref(i18n).tsx._notification.renotedBySomeUsers({ n: __props.notification.users.length })), 1 /* TEXT */)) : (__props.notification.type === 'app') ? (_openBlock(), _createElementBlock("span", { key: 15 }, _toDisplayString(__props.notification.header), 1 /* TEXT */)) : _createCommentVNode("v-if", true), (__props.withTime) ? (_openBlock(), _createBlock(_component_MkTime, {
              key: 0,
              time: __props.notification.createdAt,
              class: _normalizeClass(_ctx.$style.headerTime)
            }, null, 8 /* PROPS */, ["time"])) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", null, [ (__props.notification.type === 'reaction' || __props.notification.type === 'reaction:grouped') ? (_openBlock(), _createBlock(_component_MkA, {
              key: 0,
              class: _normalizeClass(_ctx.$style.text),
              to: _unref(notePage)(__props.notification.note),
              title: _unref(getNoteSummary)(__props.notification.note)
            }, {
              default: _withCtx(() => [
                _createElementVNode("i", {
                  class: _normalizeClass(["ti ti-quote", _ctx.$style.quote])
                }),
                _createVNode(_component_Mfm, {
                  text: _unref(getNoteSummary)(__props.notification.note),
                  plain: true,
                  nowrap: true,
                  author: __props.notification.note.user
                }, null, 8 /* PROPS */, ["text", "plain", "nowrap", "author"]),
                _createElementVNode("i", {
                  class: _normalizeClass(["ti ti-quote", _ctx.$style.quote])
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to", "title"])) : (__props.notification.type === 'renote' || __props.notification.type === 'renote:grouped') ? (_openBlock(), _createBlock(_component_MkA, {
                key: 1,
                class: _normalizeClass(_ctx.$style.text),
                to: _unref(notePage)(__props.notification.note),
                title: _unref(getNoteSummary)(__props.notification.note.renote)
              }, {
                default: _withCtx(() => [
                  _createElementVNode("i", {
                    class: _normalizeClass(["ti ti-quote", _ctx.$style.quote])
                  }),
                  _createVNode(_component_Mfm, {
                    text: _unref(getNoteSummary)(__props.notification.note.renote),
                    plain: true,
                    nowrap: true,
                    author: __props.notification.note.renote?.user
                  }, null, 8 /* PROPS */, ["text", "plain", "nowrap", "author"]),
                  _createElementVNode("i", {
                    class: _normalizeClass(["ti ti-quote", _ctx.$style.quote])
                  })
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to", "title"])) : (__props.notification.type === 'reply') ? (_openBlock(), _createBlock(_component_MkA, {
                key: 2,
                class: _normalizeClass(_ctx.$style.text),
                to: _unref(notePage)(__props.notification.note),
                title: _unref(getNoteSummary)(__props.notification.note)
              }, {
                default: _withCtx(() => [
                  _createVNode(_component_Mfm, {
                    text: _unref(getNoteSummary)(__props.notification.note),
                    plain: true,
                    nowrap: true,
                    author: __props.notification.note.user
                  }, null, 8 /* PROPS */, ["text", "plain", "nowrap", "author"])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to", "title"])) : (__props.notification.type === 'mention') ? (_openBlock(), _createBlock(_component_MkA, {
                key: 3,
                class: _normalizeClass(_ctx.$style.text),
                to: _unref(notePage)(__props.notification.note),
                title: _unref(getNoteSummary)(__props.notification.note)
              }, {
                default: _withCtx(() => [
                  _createVNode(_component_Mfm, {
                    text: _unref(getNoteSummary)(__props.notification.note),
                    plain: true,
                    nowrap: true,
                    author: __props.notification.note.user
                  }, null, 8 /* PROPS */, ["text", "plain", "nowrap", "author"])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to", "title"])) : (__props.notification.type === 'quote') ? (_openBlock(), _createBlock(_component_MkA, {
                key: 4,
                class: _normalizeClass(_ctx.$style.text),
                to: _unref(notePage)(__props.notification.note),
                title: _unref(getNoteSummary)(__props.notification.note)
              }, {
                default: _withCtx(() => [
                  _createVNode(_component_Mfm, {
                    text: _unref(getNoteSummary)(__props.notification.note),
                    plain: true,
                    nowrap: true,
                    author: __props.notification.note.user
                  }, null, 8 /* PROPS */, ["text", "plain", "nowrap", "author"])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to", "title"])) : (__props.notification.type === 'note') ? (_openBlock(), _createBlock(_component_MkA, {
                key: 5,
                class: _normalizeClass(_ctx.$style.text),
                to: _unref(notePage)(__props.notification.note),
                title: _unref(getNoteSummary)(__props.notification.note)
              }, {
                default: _withCtx(() => [
                  _createVNode(_component_Mfm, {
                    text: _unref(getNoteSummary)(__props.notification.note),
                    plain: true,
                    nowrap: true,
                    author: __props.notification.note.user
                  }, null, 8 /* PROPS */, ["text", "plain", "nowrap", "author"])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to", "title"])) : (__props.notification.type === 'pollEnded') ? (_openBlock(), _createBlock(_component_MkA, {
                key: 6,
                class: _normalizeClass(_ctx.$style.text),
                to: _unref(notePage)(__props.notification.note),
                title: _unref(getNoteSummary)(__props.notification.note)
              }, {
                default: _withCtx(() => [
                  _createElementVNode("i", {
                    class: _normalizeClass(["ti ti-quote", _ctx.$style.quote])
                  }),
                  _createVNode(_component_Mfm, {
                    text: _unref(getNoteSummary)(__props.notification.note),
                    plain: true,
                    nowrap: true,
                    author: __props.notification.note.user
                  }, null, 8 /* PROPS */, ["text", "plain", "nowrap", "author"]),
                  _createElementVNode("i", {
                    class: _normalizeClass(["ti ti-quote", _ctx.$style.quote])
                  })
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to", "title"])) : (__props.notification.type === 'scheduledNotePosted') ? (_openBlock(), _createBlock(_component_MkA, {
                key: 7,
                class: _normalizeClass(_ctx.$style.text),
                to: _unref(notePage)(__props.notification.note),
                title: _unref(getNoteSummary)(__props.notification.note)
              }, {
                default: _withCtx(() => [
                  _createElementVNode("i", {
                    class: _normalizeClass(["ti ti-quote", _ctx.$style.quote])
                  }),
                  _createVNode(_component_Mfm, {
                    text: _unref(getNoteSummary)(__props.notification.note),
                    plain: true,
                    nowrap: true,
                    author: __props.notification.note.user
                  }, null, 8 /* PROPS */, ["text", "plain", "nowrap", "author"]),
                  _createElementVNode("i", {
                    class: _normalizeClass(["ti ti-quote", _ctx.$style.quote])
                  })
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to", "title"])) : (__props.notification.type === 'roleAssigned') ? (_openBlock(), _createElementBlock("div", {
                key: 8,
                class: _normalizeClass(_ctx.$style.text)
              }, _toDisplayString(__props.notification.role.name), 1 /* TEXT */)) : (__props.notification.type === 'chatRoomInvitationReceived') ? (_openBlock(), _createElementBlock("div", {
                key: 9,
                class: _normalizeClass(_ctx.$style.text)
              }, _toDisplayString(__props.notification.invitation.room.name), 1 /* TEXT */)) : (__props.notification.type === 'achievementEarned') ? (_openBlock(), _createBlock(_component_MkA, {
                key: 10,
                class: _normalizeClass(_ctx.$style.text),
                to: "/my/achievements"
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._achievements._types[`_${__props.notification.achievement}`].title), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })) : (__props.notification.type === 'exportCompleted') ? (_openBlock(), _createBlock(_component_MkA, {
                key: 11,
                class: _normalizeClass(_ctx.$style.text),
                to: `/my/drive/file/${__props.notification.fileId}`
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.showFile), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to"])) : (__props.notification.type === 'createToken') ? (_openBlock(), _createBlock(_component_MkA, {
                key: 12,
                class: _normalizeClass(_ctx.$style.text),
                to: "/settings/apps"
              }, {
                default: _withCtx(() => [
                  _createVNode(_component_Mfm, { text: _unref(i18n).tsx._notification.createTokenDescription({ text: _unref(i18n).ts.manageAccessTokens }) }, null, 8 /* PROPS */, ["text"])
                ]),
                _: 1 /* STABLE */
              })) : (__props.notification.type === 'follow') ? (_openBlock(), _createElementBlock("span", {
                key: 13,
                class: _normalizeClass(_ctx.$style.text),
                style: "opacity: 0.6;"
              }, _toDisplayString(_unref(i18n).ts.youGotNewFollower), 1 /* TEXT */)) : (__props.notification.type === 'followRequestAccepted') ? (_openBlock(), _createElementBlock(_Fragment, { key: 14 }, [ _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.text),
                  style: "opacity: 0.6;"
                }, _toDisplayString(_unref(i18n).ts.followRequestAccepted), 1 /* TEXT */), (__props.notification.message) ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.text),
                    style: "opacity: 0.6; font-style: oblique;"
                  }, [ _createElementVNode("i", {
                      class: _normalizeClass(["ti ti-quote", _ctx.$style.quote])
                    }), _createVNode(_component_Mfm, {
                      text: __props.notification.message,
                      author: __props.notification.user,
                      plain: true,
                      nowrap: true
                    }, null, 8 /* PROPS */, ["text", "author", "plain", "nowrap"]), _createElementVNode("i", {
                      class: _normalizeClass(["ti ti-quote", _ctx.$style.quote])
                    }) ])) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */)) : (__props.notification.type === 'receiveFollowRequest') ? (_openBlock(), _createElementBlock(_Fragment, { key: 15 }, [ _createElementVNode("span", {
                  class: _normalizeClass(_ctx.$style.text),
                  style: "opacity: 0.6;"
                }, _toDisplayString(_unref(i18n).ts.receiveFollowRequest), 1 /* TEXT */), (__props.full && !followRequestDone.value) ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.followRequestCommands)
                  }, [ _createVNode(MkButton, {
                      class: _normalizeClass(_ctx.$style.followRequestCommandButton),
                      rounded: "",
                      primary: "",
                      onClick: _cache[0] || (_cache[0] = ($event: any) => (acceptFollowRequest()))
                    }, {
                      default: _withCtx(() => [
                        _hoisted_4,
                        _createTextVNode(" "),
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.accept), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }), _createVNode(MkButton, {
                      class: _normalizeClass(_ctx.$style.followRequestCommandButton),
                      rounded: "",
                      danger: "",
                      onClick: _cache[1] || (_cache[1] = ($event: any) => (rejectFollowRequest()))
                    }, {
                      default: _withCtx(() => [
                        _hoisted_5,
                        _createTextVNode(" "),
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.reject), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }) ])) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */)) : (__props.notification.type === 'test') ? (_openBlock(), _createElementBlock("span", {
                key: 16,
                class: _normalizeClass(_ctx.$style.text)
              }, _toDisplayString(_unref(i18n).ts._notification.notificationWillBeDisplayedLikeThis), 1 /* TEXT */)) : (__props.notification.type === 'app') ? (_openBlock(), _createElementBlock("span", {
                key: 17,
                class: _normalizeClass(_ctx.$style.text)
              }, [ _createVNode(_component_Mfm, {
                  text: __props.notification.body,
                  nowrap: false
                }, null, 8 /* PROPS */, ["text", "nowrap"]) ])) : _createCommentVNode("v-if", true), (__props.notification.type === 'reaction:grouped') ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.notification.reactions, (reaction) => {
                return (_openBlock(), _createElementBlock("div", {
                  key: reaction.user.id + reaction.reaction,
                  class: _normalizeClass(_ctx.$style.reactionsItem)
                }, [
                  _createVNode(_component_MkAvatar, {
                    class: _normalizeClass(_ctx.$style.reactionsItemAvatar),
                    user: reaction.user,
                    link: "",
                    preview: ""
                  }, null, 8 /* PROPS */, ["user"]),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.reactionsItemReaction)
                  }, [
                    _createVNode(MkReactionIcon, {
                      withTooltip: true,
                      reaction: reaction.reaction.replace(/^:(\w+):$/, ':$1@.:'),
                      noStyle: true,
                      style: "width: 100%; height: 100% !important; object-fit: contain;"
                    }, null, 8 /* PROPS */, ["withTooltip", "reaction", "noStyle"])
                  ])
                ]))
              }), 128 /* KEYED_FRAGMENT */)) ])) : (__props.notification.type === 'renote:grouped') ? (_openBlock(), _createElementBlock("div", { key: 1 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.notification.users, (user) => {
                  return (_openBlock(), _createElementBlock("div", {
                    key: user.id,
                    class: _normalizeClass(_ctx.$style.reactionsItem)
                  }, [
                    _createVNode(_component_MkAvatar, {
                      class: _normalizeClass(_ctx.$style.reactionsItemAvatar),
                      user: user,
                      link: "",
                      preview: ""
                    }, null, 8 /* PROPS */, ["user"])
                  ]))
                }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true) ]) ]) ]))
}
}

})
