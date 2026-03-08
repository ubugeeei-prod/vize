import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, TransitionGroup as _TransitionGroup, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-up" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { style: "height: 1em; width: 1px; background: var(--MI_THEME-divider);" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-down" })
import { ref, useTemplateRef, computed, onMounted, onBeforeUnmount, onDeactivated, onActivated } from 'vue'
import * as Misskey from 'misskey-js'
import { getScrollContainer } from '@@/js/scroll.js'
import XMessage from './XMessage.vue'
import XForm from './room.form.vue'
import XSearch from './room.search.vue'
import XMembers from './room.members.vue'
import XInfo from './room.info.vue'
import type { MenuItem } from '@/types/menu.js'
import type { PageHeaderItem } from '@/types/page-header.js'
import * as os from '@/os.js'
import { useStream } from '@/stream.js'
import * as sound from '@/utility/sound.js'
import { i18n } from '@/i18n.js'
import { ensureSignin } from '@/i.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { definePage } from '@/page.js'
import { prefer } from '@/preferences.js'
import MkButton from '@/components/MkButton.vue'
import { useRouter } from '@/router.js'
import { useMutationObserver } from '@/composables/use-mutation-observer.js'
import MkInfo from '@/components/MkInfo.vue'
import { makeDateSeparatedTimelineComputedRef } from '@/utility/timeline-date-separate.js'

export type NormalizedChatMessage = Omit<Misskey.entities.ChatMessageLite, 'fromUser' | 'reactions'> & {
	fromUser: Misskey.entities.UserLite;
	reactions: (Misskey.entities.ChatMessageLite['reactions'][number] & {
		user: Misskey.entities.UserLite;
	})[];
};
const SCROLL_HEAD_THRESHOLD = 200;

export default /*@__PURE__*/_defineComponent({
  __name: 'room',
  props: {
    userId: { type: String, required: false },
    roomId: { type: String, required: false }
  },
  setup(__props: any) {

const props = __props
const $i = ensureSignin();
const router = useRouter();
const initializing = ref(false);
const initialized = ref(false);
const moreFetching = ref(false);
const messages = ref<NormalizedChatMessage[]>([]);
const canFetchMore = ref(false);
const user = ref<Misskey.entities.UserDetailed | null>(null);
const room = ref<Misskey.entities.ChatRoom | null>(null);
const connection = ref<Misskey.IChannelConnection<Misskey.Channels['chatUser']> | Misskey.IChannelConnection<Misskey.Channels['chatRoom']> | null>(null);
const showIndicator = ref(false);
const timelineEl = useTemplateRef('timelineEl');
const timeline = makeDateSeparatedTimelineComputedRef(messages);
// column-reverseなので本来はスクロール位置の最下部への追従は不要なはずだが、おそらくブラウザのバグにより、最下部にスクロールした状態でも追従されない場合がある(スクロール位置が少数になることがあるのが関わっていそう)
// そのため補助としてMutationObserverを使って追従を行う
useMutationObserver(timelineEl, {
	subtree: true,
	childList: true,
	attributes: false,
}, () => {
	const scrollContainer = getScrollContainer(timelineEl.value)!;
	// column-reverseなのでscrollTopは負になる
	if (-scrollContainer.scrollTop < SCROLL_HEAD_THRESHOLD) {
		scrollContainer.scrollTo({
			top: 0,
			behavior: 'instant',
		});
	}
});
function normalizeMessage(message: Misskey.entities.ChatMessageLite | Misskey.entities.ChatMessage): NormalizedChatMessage {
	return {
		...message,
		fromUser: message.fromUser ?? (message.fromUserId === $i.id ? $i : user.value!),
		reactions: message.reactions.map(record => ({
			...record,
			user: record.user ?? (message.fromUserId === $i.id ? user.value! : $i),
		})),
	};
}
async function initialize() {
	const LIMIT = 20;
	if (initializing.value) return;
	initializing.value = true;
	initialized.value = false;
	if (props.userId) {
		const [u, m] = await Promise.all([
			misskeyApi('users/show', { userId: props.userId }),
			misskeyApi('chat/messages/user-timeline', { userId: props.userId, limit: LIMIT }),
		]);
		user.value = u;
		messages.value = m.map(x => normalizeMessage(x));
		if (messages.value.length === LIMIT) {
			canFetchMore.value = true;
		}
		connection.value = useStream().useChannel('chatUser', {
			otherId: user.value.id,
		});
		connection.value.on('message', onMessage);
		connection.value.on('deleted', onDeleted);
		connection.value.on('react', onReact);
		connection.value.on('unreact', onUnreact);
	} else if (props.roomId) {
		const [rResult, mResult] = await Promise.allSettled([
			misskeyApi('chat/rooms/show', { roomId: props.roomId }),
			misskeyApi('chat/messages/room-timeline', { roomId: props.roomId, limit: LIMIT }),
		]);
		if (rResult.status === 'rejected') {
			os.alert({
				type: 'error',
				text: i18n.ts.somethingHappened,
			});
			initializing.value = false;
			return;
		}
		const r = rResult.value as Misskey.entities.ChatRoomsShowResponse;
		if (r.invitationExists) {
			const confirm = await os.confirm({
				type: 'question',
				title: r.name,
				text: i18n.ts._chat.youAreNotAMemberOfThisRoomButInvited + '\n' + i18n.ts._chat.doYouAcceptInvitation,
			});
			if (confirm.canceled) {
				initializing.value = false;
				router.push('/chat');
				return;
			} else {
				await os.apiWithDialog('chat/rooms/join', { roomId: r.id });
				initializing.value = false;
				initialize();
				return;
			}
		}
		const m = mResult.status === 'fulfilled' ? mResult.value as Misskey.entities.ChatMessagesRoomTimelineResponse : [];
		room.value = r;
		messages.value = m.map(x => normalizeMessage(x));
		if (messages.value.length === LIMIT) {
			canFetchMore.value = true;
		}
		connection.value = useStream().useChannel('chatRoom', {
			roomId: room.value.id,
		});
		connection.value.on('message', onMessage);
		connection.value.on('deleted', onDeleted);
		connection.value.on('react', onReact);
		connection.value.on('unreact', onUnreact);
	}
	window.document.addEventListener('visibilitychange', onVisibilitychange);
	initialized.value = true;
	initializing.value = false;
}
let isActivated = true;
onActivated(() => {
	isActivated = true;
});
onDeactivated(() => {
	isActivated = false;
});
async function fetchMore() {
	const LIMIT = 30;
	moreFetching.value = true;
	const newMessages = props.userId ? await misskeyApi('chat/messages/user-timeline', {
		userId: user.value!.id,
		limit: LIMIT,
		untilId: messages.value[messages.value.length - 1].id,
	}) : await misskeyApi('chat/messages/room-timeline', {
		roomId: room.value!.id,
		limit: LIMIT,
		untilId: messages.value[messages.value.length - 1].id,
	});
	messages.value.push(...newMessages.map(x => normalizeMessage(x)));
	canFetchMore.value = newMessages.length === LIMIT;
	moreFetching.value = false;
}
function onMessage(message: Misskey.entities.ChatMessageLite) {
	sound.playMisskeySfx('chatMessage');
	messages.value.unshift(normalizeMessage(message));
	// TODO: DOM的にバックグラウンドになっていないかどうかも考慮する
	if (message.fromUserId !== $i.id && !window.document.hidden && isActivated) {
		connection.value?.send('read', {
			id: message.id,
		});
	}
	if (message.fromUserId !== $i.id) {
		//notifyNewMessage();
	}
}
function onDeleted(id: string) {
	const index = messages.value.findIndex(m => m.id === id);
	if (index !== -1) {
		messages.value.splice(index, 1);
	}
}
function onReact(ctx: Parameters<Misskey.Channels['chatUser']['events']['react']>[0] | Parameters<Misskey.Channels['chatRoom']['events']['react']>[0]) {
	const message = messages.value.find(m => m.id === ctx.messageId);
	if (message) {
		if (room.value == null) { // 1on1の時はuserは省略される
			message.reactions.push({
				reaction: ctx.reaction,
				user: message.fromUserId === $i.id ? user.value! : $i,
			});
		} else {
			message.reactions.push({
				reaction: ctx.reaction,
				user: ctx.user!,
			});
		}
	}
}
function onUnreact(ctx: Parameters<Misskey.Channels['chatUser']['events']['unreact']>[0] | Parameters<Misskey.Channels['chatRoom']['events']['unreact']>[0]) {
	const message = messages.value.find(m => m.id === ctx.messageId);
	if (message) {
		const index = message.reactions.findIndex(r => r.reaction === ctx.reaction && r.user.id === ctx.user!.id);
		if (index !== -1) {
			message.reactions.splice(index, 1);
		}
	}
}
function onIndicatorClick() {
	showIndicator.value = false;
}
function notifyNewMessage() {
	showIndicator.value = true;
}
function onVisibilitychange() {
	if (window.document.hidden) return;
	// TODO
}
onMounted(() => {
	initialize();
});
onActivated(() => {
	if (!initialized.value) {
		initialize();
	}
});
onBeforeUnmount(() => {
	connection.value?.dispose();
	window.document.removeEventListener('visibilitychange', onVisibilitychange);
});
async function inviteUser() {
	if (room.value == null) return;
	const invitee = await os.selectUser({ includeSelf: false, localOnly: true });
	os.apiWithDialog('chat/rooms/invitations/create', {
		roomId: room.value.id,
		userId: invitee.id,
	});
}
async function leaveRoom() {
	if (room.value == null) return;
	const { canceled } = await os.confirm({
		type: 'warning',
		text: i18n.ts.areYouSure,
	});
	if (canceled) return;
	misskeyApi('chat/rooms/leave', {
		roomId: room.value.id,
	});
	router.push('/chat');
}
function showMenu(ev: PointerEvent) {
	const menuItems: MenuItem[] = [];
	if (room.value) {
		if (room.value.ownerId === $i.id) {
			menuItems.push({
				text: i18n.ts._chat.inviteUser,
				icon: 'ti ti-user-plus',
				action: () => {
					inviteUser();
				},
			});
		} else {
			menuItems.push({
				text: i18n.ts._chat.leave,
				icon: 'ti ti-x',
				action: () => {
					leaveRoom();
				},
			});
		}
	}
	os.popupMenu(menuItems, ev.currentTarget ?? ev.target);
}
const tab = ref('chat');
const headerTabs = computed(() => room.value ? [{
	key: 'chat',
	title: i18n.ts._chat.messages,
	icon: 'ti ti-messages',
}, {
	key: 'members',
	title: i18n.ts._chat.members,
	icon: 'ti ti-users',
}, {
	key: 'search',
	title: i18n.ts.search,
	icon: 'ti ti-search',
}, {
	key: 'info',
	title: i18n.ts.info,
	icon: 'ti ti-info-circle',
}] : [{
	key: 'chat',
	title: i18n.ts._chat.messages,
	icon: 'ti ti-messages',
}, {
	key: 'search',
	title: i18n.ts.search,
	icon: 'ti ti-search',
}]);
const headerActions = computed<PageHeaderItem[]>(() => [{
	icon: 'ti ti-dots',
	handler: showMenu,
}]);
definePage(computed(() => {
	if (initialized.value) {
		if (user.value) {
			return {
				userName: user.value,
				title: user.value.name ?? user.value.username,
				avatar: user.value,
			};
		} else if (room.value) {
			return {
				title: room.value.name,
				icon: 'ti ti-users',
			};
		} else {
			return {
				title: i18n.ts.directMessage,
			};
		}
	} else {
		return {
			title: i18n.ts.directMessage,
		};
	}
}));

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      reversed: tab.value === 'chat',
      tabs: headerTabs.value,
      actions: headerActions.value,
      tab: tab.value,
      "onUpdate:tab": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
    }, {
      footer: _withCtx(() => [
        (tab.value === 'chat')
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.footer)
          }, [
            _createElementVNode("div", { class: "_gaps" }, [
              _createVNode(_Transition, { name: "fade" }, {
                default: _withCtx(() => [
                  _withDirectives(_createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.new)
                  }, [
                    _createElementVNode("button", {
                      class: _normalizeClass(["_buttonPrimary", _ctx.$style.newButton]),
                      onClick: onIndicatorClick
                    }, [
                      _createElementVNode("i", {
                        class: _normalizeClass(["fas ti-fw fa-arrow-circle-down", _ctx.$style.newIcon])
                      }),
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._chat.newMessage), 1 /* TEXT */)
                    ])
                  ], 512 /* NEED_PATCH */), [
                    [_vShow, showIndicator.value]
                  ])
                ]),
                _: 1 /* STABLE */
              }),
              (initialized.value)
                ? (_openBlock(), _createBlock(XForm, {
                  key: 0,
                  user: user.value,
                  room: room.value,
                  class: _normalizeClass(_ctx.$style.form)
                }, null, 8 /* PROPS */, ["user", "room"]))
                : _createCommentVNode("v-if", true)
            ])
          ]))
          : _createCommentVNode("v-if", true)
      ]),
      default: _withCtx(() => [
        (tab.value === 'chat')
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "_spacer",
            style: "--MI_SPACER-w: 700px;"
          }, [
            _createElementVNode("div", { class: "_gaps" }, [
              (initializing.value)
                ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                  _createVNode(_component_MkLoading)
                ]))
                : (messages.value.length === 0)
                  ? (_openBlock(), _createElementBlock("div", { key: 1 }, [
                    _createElementVNode("div", {
                      class: "_gaps",
                      style: "text-align: center;"
                    }, [
                      _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._chat.noMessagesYet), 1 /* TEXT */),
                      (user.value)
                        ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                          (user.value.chatScope === 'followers')
                            ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_unref(i18n).ts._chat.thisUserAllowsChatOnlyFromFollowers), 1 /* TEXT */))
                            : (user.value.chatScope === 'following')
                              ? (_openBlock(), _createElementBlock("div", { key: 1 }, _toDisplayString(_unref(i18n).ts._chat.thisUserAllowsChatOnlyFromFollowing), 1 /* TEXT */))
                            : (user.value.chatScope === 'mutual')
                              ? (_openBlock(), _createElementBlock("div", { key: 2 }, _toDisplayString(_unref(i18n).ts._chat.thisUserAllowsChatOnlyFromMutualFollowing), 1 /* TEXT */))
                            : (user.value.chatScope === 'none')
                              ? (_openBlock(), _createElementBlock("div", { key: 3 }, _toDisplayString(_unref(i18n).ts._chat.thisUserNotAllowedChatAnyone), 1 /* TEXT */))
                            : _createCommentVNode("v-if", true)
                        ], 64 /* STABLE_FRAGMENT */))
                        : (room.value)
                          ? (_openBlock(), _createElementBlock("div", { key: 1 }, _toDisplayString(_unref(i18n).ts._chat.inviteUserToChat), 1 /* TEXT */))
                        : _createCommentVNode("v-if", true)
                    ])
                  ]))
                : (_openBlock(), _createElementBlock("div", {
                  key: 2,
                  ref: "timelineEl",
                  class: "_gaps"
                }, [
                  (canFetchMore.value)
                    ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                      _createVNode(MkButton, {
                        class: _normalizeClass(_ctx.$style.more),
                        wait: moreFetching.value,
                        primary: "",
                        rounded: "",
                        onClick: fetchMore
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.loadMore), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["wait"])
                    ]))
                    : _createCommentVNode("v-if", true),
                  _createVNode(_TransitionGroup, {
                    enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_x_enterActive : '',
                    leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_x_leaveActive : '',
                    enterFromClass: _unref(prefer).s.animation ? _ctx.$style.transition_x_enterFrom : '',
                    leaveToClass: _unref(prefer).s.animation ? _ctx.$style.transition_x_leaveTo : '',
                    moveClass: _unref(prefer).s.animation ? _ctx.$style.transition_x_move : '',
                    tag: "div",
                    class: "_gaps"
                  }, {
                    default: _withCtx(() => [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(timeline).toReversed(), (item) => {
                        return (_openBlock(), _createElementBlock(_Fragment, { key: item.id }, [
                          (item.type === 'item')
                            ? (_openBlock(), _createBlock(XMessage, {
                              key: 0,
                              message: item.data
                            }, null, 8 /* PROPS */, ["message"]))
                            : (item.type === 'date')
                              ? (_openBlock(), _createElementBlock("div", {
                                key: 1,
                                class: _normalizeClass(_ctx.$style.dateDivider)
                              }, [
                                _createElementVNode("span", null, [
                                  _hoisted_1,
                                  _createTextVNode(" " + _toDisplayString(item.nextText), 1 /* TEXT */)
                                ]),
                                _hoisted_2,
                                _createElementVNode("span", null, [
                                  _createTextVNode(_toDisplayString(item.prevText) + " ", 1 /* TEXT */),
                                  _hoisted_3
                                ])
                              ]))
                            : _createCommentVNode("v-if", true)
                        ], 64 /* STABLE_FRAGMENT */))
                      }), 128 /* KEYED_FRAGMENT */))
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass", "moveClass"])
                ])),
              (user.value && (!user.value.canChat || user.value.host !== null))
                ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                  _createVNode(MkInfo, { warn: "" }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._chat.chatNotAvailableInOtherAccount), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]))
                : _createCommentVNode("v-if", true),
              (_unref($i).policies.chatAvailability !== 'available')
                ? (_openBlock(), _createBlock(MkInfo, {
                  key: 0,
                  warn: ""
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref($i).policies.chatAvailability === 'readonly' ? _unref(i18n).ts._chat.chatIsReadOnlyForThisAccountOrServer : _unref(i18n).ts._chat.chatNotAvailableForThisAccountOrServer), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }))
                : _createCommentVNode("v-if", true)
            ])
          ]))
          : (tab.value === 'search')
            ? (_openBlock(), _createElementBlock("div", {
              key: 1,
              class: "_spacer",
              style: "--MI_SPACER-w: 700px;"
            }, [
              _createVNode(XSearch, {
                userId: __props.userId,
                roomId: __props.roomId
              }, null, 8 /* PROPS */, ["userId", "roomId"])
            ]))
          : (tab.value === 'members')
            ? (_openBlock(), _createElementBlock("div", {
              key: 2,
              class: "_spacer",
              style: "--MI_SPACER-w: 700px;"
            }, [
              (room.value != null)
                ? (_openBlock(), _createBlock(XMembers, {
                  key: 0,
                  room: room.value,
                  onInviteUser: inviteUser
                }, null, 8 /* PROPS */, ["room"]))
                : _createCommentVNode("v-if", true)
            ]))
          : (tab.value === 'info')
            ? (_openBlock(), _createElementBlock("div", {
              key: 3,
              class: "_spacer",
              style: "--MI_SPACER-w: 700px;"
            }, [
              (room.value != null)
                ? (_openBlock(), _createBlock(XInfo, {
                  key: 0,
                  room: room.value
                }, null, 8 /* PROPS */, ["room"]))
                : _createCommentVNode("v-if", true)
            ]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["reversed", "tabs", "actions", "tab"]))
}
}

})
