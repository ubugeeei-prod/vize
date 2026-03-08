import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, TransitionGroup as _TransitionGroup, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots-circle-horizontal" })
import { computed, defineAsyncComponent, provide } from 'vue'
import * as mfm from 'mfm-js'
import * as Misskey from 'misskey-js'
import { url } from '@@/js/config.js'
import { isLink } from '@@/js/is-link.js'
import type { MenuItem } from '@/types/menu.js'
import type { NormalizedChatMessage } from './room.vue'
import { extractUrlFromMfm } from '@/utility/extract-url-from-mfm.js'
import MkUrlPreview from '@/components/MkUrlPreview.vue'
import { ensureSignin } from '@/i.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import MkFukidashi from '@/components/MkFukidashi.vue'
import * as os from '@/os.js'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'
import MkMediaList from '@/components/MkMediaList.vue'
import { reactionPicker } from '@/utility/reaction-picker.js'
import * as sound from '@/utility/sound.js'
import MkReactionIcon from '@/components/MkReactionIcon.vue'
import { prefer } from '@/preferences.js'
import { DI } from '@/di.js'
import { getHTMLElementOrNull } from '@/utility/get-dom-node-or-null.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'XMessage',
  props: {
    message: { type: null, required: true },
    isSearchResult: { type: Boolean, required: false }
  },
  setup(__props: any) {

const props = __props
const $i = ensureSignin();
const isMe = computed(() => props.message.fromUserId === $i.id);
const urls = computed(() => props.message.text ? extractUrlFromMfm(mfm.parse(props.message.text)) : []);
provide(DI.mfmEmojiReactCallback, (reaction) => {
	if ($i.policies.chatAvailability !== 'available') return;
	sound.playMisskeySfx('reaction');
	misskeyApi('chat/messages/react', {
		messageId: props.message.id,
		reaction: reaction,
	});
});
function react(ev: PointerEvent) {
	if ($i.policies.chatAvailability !== 'available') return;
	const targetEl = getHTMLElementOrNull(ev.currentTarget ?? ev.target);
	if (!targetEl) return;
	reactionPicker.show(targetEl, null, async (reaction) => {
		sound.playMisskeySfx('reaction');
		misskeyApi('chat/messages/react', {
			messageId: props.message.id,
			reaction: reaction,
		});
	});
}
function onReactionClick(record: Misskey.entities.ChatMessage['reactions'][0]) {
	if ($i.policies.chatAvailability !== 'available') return;
	if (record.user.id === $i.id) {
		misskeyApi('chat/messages/unreact', {
			messageId: props.message.id,
			reaction: record.reaction,
		});
	} else {
		if (!props.message.reactions.some(r => r.user.id === $i.id && r.reaction === record.reaction)) {
			sound.playMisskeySfx('reaction');
			misskeyApi('chat/messages/react', {
				messageId: props.message.id,
				reaction: record.reaction,
			});
		}
	}
}
function onContextmenu(ev: PointerEvent) {
	if (ev.target && isLink(ev.target as HTMLElement)) return;
	if (window.getSelection()?.toString() !== '') return;
	showMenu(ev, true);
}
function showMenu(ev: PointerEvent, contextmenu = false) {
	const menu: MenuItem[] = [];
	if (!isMe.value && $i.policies.chatAvailability === 'available') {
		menu.push({
			text: i18n.ts.reaction,
			icon: 'ti ti-mood-plus',
			action: (ev) => {
				react(ev);
			},
		});
		menu.push({
			type: 'divider',
		});
	}
	menu.push({
		text: i18n.ts.copyContent,
		icon: 'ti ti-copy',
		action: () => {
			copyToClipboard(props.message.text ?? '');
		},
	});
	menu.push({
		type: 'divider',
	});
	if (isMe.value && $i.policies.chatAvailability === 'available') {
		menu.push({
			text: i18n.ts.delete,
			icon: 'ti ti-trash',
			danger: true,
			action: () => {
				misskeyApi('chat/messages/delete', {
					messageId: props.message.id,
				});
			},
		});
	}
	if (!isMe.value && props.message.fromUser != null) {
		menu.push({
			text: i18n.ts.reportAbuse,
			icon: 'ti ti-exclamation-circle',
			action: async () => {
				const localUrl = `${url}/chat/messages/${props.message.id}`;
				const { dispose } = await os.popupAsyncWithDialog(import('@/components/MkAbuseReportWindow.vue').then(x => x.default), {
					user: props.message.fromUser!,
					initialComment: `${localUrl}\n-----\n`,
				}, {
					closed: () => dispose(),
				});
			},
		});
	}
	if (contextmenu) {
		os.contextMenu(menu, ev);
	} else {
		os.popupMenu(menu, ev.currentTarget ?? ev.target);
	}
}

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_Mfm = _resolveComponent("Mfm")
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.isMe]: isMe.value }])
    }, [ _createVNode(_component_MkAvatar, {
        class: _normalizeClass([_ctx.$style.avatar, _unref(prefer).s.useStickyIcons ? _ctx.$style.useSticky : null]),
        user: __props.message.fromUser,
        link: !isMe.value,
        preview: false
      }, null, 10 /* CLASS, PROPS */, ["user", "link", "preview"]), _createElementVNode("div", {
        class: _normalizeClass([_ctx.$style.body, __props.message.file != null ? _ctx.$style.fullWidth : null]),
        onContextmenu: _withModifiers(onContextmenu, ["stop"])
      }, [ _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.header)
        }, [ (!isMe.value && _unref(prefer).s['chat.showSenderName'] && __props.message.fromUser != null) ? (_openBlock(), _createBlock(_component_MkUserName, {
              key: 0,
              user: __props.message.fromUser
            }, null, 8 /* PROPS */, ["user"])) : _createCommentVNode("v-if", true) ]), _createVNode(MkFukidashi, {
          class: _normalizeClass(_ctx.$style.fukidashi),
          tail: isMe.value ? 'right' : 'left',
          fullWidth: __props.message.file != null,
          accented: isMe.value
        }, {
          default: _withCtx(() => [
            (__props.message.text)
              ? (_openBlock(), _createBlock(_component_Mfm, {
                key: 0,
                ref: "text",
                class: "_selectable",
                text: __props.message.text,
                i: _unref($i),
                nyaize: 'respect',
                enableEmojiMenu: true,
                enableEmojiMenuReaction: true
              }, null, 8 /* PROPS */, ["text", "i", "nyaize", "enableEmojiMenu", "enableEmojiMenuReaction"]))
              : _createCommentVNode("v-if", true),
            (__props.message.file)
              ? (_openBlock(), _createBlock(MkMediaList, {
                key: 0,
                mediaList: [__props.message.file]
              }, null, 8 /* PROPS */, ["mediaList"]))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["tail", "fullWidth", "accented"]), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(urls.value, (url) => {
          return (_openBlock(), _createBlock(MkUrlPreview, {
            key: _unref(url),
            url: _unref(url),
            style: "margin: 8px 0;"
          }, null, 8 /* PROPS */, ["url"]))
        }), 128 /* KEYED_FRAGMENT */)), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.footer)
        }, [ _createElementVNode("button", {
            class: "_textButton",
            style: "color: currentColor;",
            onClick: showMenu
          }, [ _hoisted_1 ]), _createVNode(_component_MkTime, {
            class: _normalizeClass(_ctx.$style.time),
            time: __props.message.createdAt
          }, null, 8 /* PROPS */, ["time"]), (__props.isSearchResult && 'toRoom' in __props.message && __props.message.toRoom != null) ? (_openBlock(), _createBlock(_component_MkA, {
              key: 0,
              to: `/chat/room/${__props.message.toRoomId}`
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(__props.message.toRoom.name), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true), (__props.isSearchResult && 'toUser' in __props.message && __props.message.toUser != null && isMe.value) ? (_openBlock(), _createBlock(_component_MkA, {
              key: 0,
              to: `/chat/user/${__props.message.toUserId}`
            }, {
              default: _withCtx(() => [
                _createTextVNode("@"),
                _createTextVNode(_toDisplayString(__props.message.toUser.username), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true) ]), _createVNode(_TransitionGroup, {
          enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_reaction_enterActive : '',
          leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_reaction_leaveActive : '',
          enterFromClass: _unref(prefer).s.animation ? _ctx.$style.transition_reaction_enterFrom : '',
          leaveToClass: _unref(prefer).s.animation ? _ctx.$style.transition_reaction_leaveTo : '',
          moveClass: _unref(prefer).s.animation ? _ctx.$style.transition_reaction_move : '',
          tag: "div",
          class: _normalizeClass(_ctx.$style.reactions)
        }, {
          default: _withCtx(() => [
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.message.reactions, (record) => {
              return (_openBlock(), _createElementBlock("div", {
                key: record.reaction + record.user.id,
                class: _normalizeClass([_ctx.$style.reaction, record.user.id === _unref($i).id ? _ctx.$style.reactionMy : null]),
                onClick: _cache[0] || (_cache[0] = ($event: any) => (onReactionClick(record)))
              }, [
                _createVNode(_component_MkAvatar, {
                  user: record.user,
                  link: false,
                  class: _normalizeClass(_ctx.$style.reactionAvatar)
                }, null, 8 /* PROPS */, ["user", "link"]),
                _createVNode(MkReactionIcon, {
                  withTooltip: true,
                  reaction: record.reaction.replace(/^:(\w+):$/, ':$1@.:'),
                  noStyle: true,
                  class: _normalizeClass(_ctx.$style.reactionIcon)
                }, null, 8 /* PROPS */, ["withTooltip", "reaction", "noStyle"])
              ], 2 /* CLASS */))
            }), 128 /* KEYED_FRAGMENT */))
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass", "moveClass"]) ], 34 /* CLASS, NEED_HYDRATION */) ], 2 /* CLASS */))
}
}

})
