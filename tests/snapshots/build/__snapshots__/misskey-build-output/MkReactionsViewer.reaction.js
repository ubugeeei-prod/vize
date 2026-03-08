import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveDirective as _resolveDirective, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref, withModifiers as _withModifiers } from "vue"

import { computed, inject, onMounted, useTemplateRef, watch } from 'vue'
import * as Misskey from 'misskey-js'
import { getUnicodeEmojiOrNull } from '@@/js/emojilist.js'
import MkCustomEmojiDetailedDialog from './MkCustomEmojiDetailedDialog.vue'
import type { MenuItem } from '@/types/menu'
import XDetails from '@/components/MkReactionsViewer.details.vue'
import MkReactionIcon from '@/components/MkReactionIcon.vue'
import * as os from '@/os.js'
import { misskeyApi, misskeyApiGet } from '@/utility/misskey-api.js'
import { useTooltip } from '@/composables/use-tooltip.js'
import { $i } from '@/i.js'
import MkReactionEffect from '@/components/MkReactionEffect.vue'
import { i18n } from '@/i18n.js'
import * as sound from '@/utility/sound.js'
import { checkReactionPermissions } from '@/utility/check-reaction-permissions.js'
import { customEmojisMap } from '@/custom-emojis.js'
import { prefer } from '@/preferences.js'
import { DI } from '@/di.js'
import { noteEvents } from '@/composables/use-note-capture.js'
import { mute as muteEmoji, unmute as unmuteEmoji, checkMuted as isEmojiMuted } from '@/utility/emoji-mute.js'
import { haptic } from '@/utility/haptic.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkReactionsViewer.reaction',
  props: {
    noteId: { type: null, required: true },
    reaction: { type: String, required: true },
    reactionEmojis: { type: null, required: true },
    myReaction: { type: null, required: true },
    count: { type: Number, required: true },
    isInitial: { type: Boolean, required: true }
  },
  emits: ["reactionToggled"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const mock = inject(DI.mock, false);
const buttonEl = useTemplateRef('buttonEl');
const emojiName = computed(() => props.reaction.replace(/:/g, '').replace(/@\./, ''));
const canToggle = computed(() => {
	const emoji = customEmojisMap.get(emojiName.value) ?? getUnicodeEmojiOrNull(props.reaction);

	// TODO
	//return !props.reaction.match(/@\w/) && $i && emoji && checkReactionPermissions($i, props.note, emoji);
	return props.reaction.match(/@\w/) == null && $i != null && emoji != null;
});
const canGetInfo = computed(() => !props.reaction.match(/@\w/) && props.reaction.includes(':'));
const isLocalCustomEmoji = props.reaction[0] === ':' && props.reaction.includes('@.');
async function toggleReaction() {
	if (!canToggle.value) return;
	if ($i == null) return;
	const me = $i;
	const oldReaction = props.myReaction;
	if (oldReaction) {
		const confirm = await os.confirm({
			type: 'warning',
			text: oldReaction !== props.reaction ? i18n.ts.changeReactionConfirm : i18n.ts.cancelReactionConfirm,
		});
		if (confirm.canceled) return;
		if (oldReaction !== props.reaction) {
			sound.playMisskeySfx('reaction');
			haptic();
		}
		if (mock) {
			emit('reactionToggled', props.reaction, (props.count - 1));
			return;
		}
		misskeyApi('notes/reactions/delete', {
			noteId: props.noteId,
		}).then(() => {
			noteEvents.emit(`unreacted:${props.noteId}`, {
				userId: me.id,
				reaction: oldReaction,
			});
			if (oldReaction !== props.reaction) {
				misskeyApi('notes/reactions/create', {
					noteId: props.noteId,
					reaction: props.reaction,
				}).then(() => {
					const emoji = customEmojisMap.get(emojiName.value);
					if (emoji == null) return;
					noteEvents.emit(`reacted:${props.noteId}`, {
						userId: me.id,
						reaction: props.reaction,
						emoji: emoji,
					});
				});
			}
		});
	} else {
		if (prefer.s.confirmOnReact) {
			const confirm = await os.confirm({
				type: 'question',
				text: i18n.tsx.reactAreYouSure({ emoji: props.reaction.replace('@.', '') }),
			});
			if (confirm.canceled) return;
		}
		sound.playMisskeySfx('reaction');
		haptic();
		if (mock) {
			emit('reactionToggled', props.reaction, (props.count + 1));
			return;
		}
		misskeyApi('notes/reactions/create', {
			noteId: props.noteId,
			reaction: props.reaction,
		}).then(() => {
			const emoji = customEmojisMap.get(emojiName.value);
			if (emoji == null) return;
			noteEvents.emit(`reacted:${props.noteId}`, {
				userId: me.id,
				reaction: props.reaction,
				emoji: emoji,
			});
		});
		// TODO: 上位コンポーネントでやる
		//if (props.note.text && props.note.text.length > 100 && (Date.now() - new Date(props.note.createdAt).getTime() < 1000 * 3)) {
		//	claimAchievement('reactWithoutRead');
		//}
	}
}
async function menu(ev: PointerEvent) {
	let menuItems: MenuItem[] = [];
	if (canGetInfo.value) {
		menuItems.push({
			text: i18n.ts.info,
			icon: 'ti ti-info-circle',
			action: async () => {
				const { dispose } = os.popup(MkCustomEmojiDetailedDialog, {
					emoji: await misskeyApiGet('emoji', {
						name: props.reaction.replace(/:/g, '').replace(/@\./, ''),
					}),
				}, {
					closed: () => dispose(),
				});
			},
		});
	}
	if (isEmojiMuted(props.reaction).value) {
		menuItems.push({
			text: i18n.ts.emojiUnmute,
			icon: 'ti ti-mood-smile',
			action: () => {
				os.confirm({
					type: 'question',
					title: i18n.tsx.unmuteX({ x: isLocalCustomEmoji ? `:${emojiName.value}:` : props.reaction }),
				}).then(({ canceled }) => {
					if (canceled) return;
					unmuteEmoji(props.reaction);
				});
			},
		});
	} else {
		menuItems.push({
			text: i18n.ts.emojiMute,
			icon: 'ti ti-mood-off',
			action: () => {
				os.confirm({
					type: 'question',
					title: i18n.tsx.muteX({ x: isLocalCustomEmoji ? `:${emojiName.value}:` : props.reaction }),
				}).then(({ canceled }) => {
					if (canceled) return;
					muteEmoji(props.reaction);
				});
			},
		});
	}
	os.popupMenu(menuItems, ev.currentTarget ?? ev.target);
}
function anime() {
	if (window.document.hidden || !prefer.s.animation || buttonEl.value == null) return;
	const rect = buttonEl.value.getBoundingClientRect();
	const x = rect.left + 16;
	const y = rect.top + (buttonEl.value.offsetHeight / 2);
	const { dispose } = os.popup(MkReactionEffect, { reaction: props.reaction, x, y }, {
		end: () => dispose(),
	});
}
watch(() => props.count, (newCount, oldCount) => {
	if (oldCount < newCount) anime();
});
onMounted(() => {
	if (!props.isInitial) anime();
});
if (!mock) {
	useTooltip(buttonEl, async (showing) => {
		if (buttonEl.value == null) return;
		const reactions = await misskeyApiGet('notes/reactions', {
			noteId: props.noteId,
			type: props.reaction,
			limit: 10,
			_cacheKey_: props.count,
		});
		const users = reactions.map(x => x.user);
		const { dispose } = os.popup(XDetails, {
			showing,
			reaction: props.reaction,
			users,
			count: props.count,
			anchorElement: buttonEl.value,
		}, {
			closed: () => dispose(),
		});
	}, 100);
}

return (_ctx: any,_cache: any) => {
  const _directive_ripple = _resolveDirective("ripple")

  return _withDirectives((_openBlock(), _createElementBlock("button", {
      ref_key: "buttonEl", ref: buttonEl,
      class: _normalizeClass(["_button", [_ctx.$style.root, { [_ctx.$style.reacted]: __props.myReaction == __props.reaction, [_ctx.$style.canToggle]: canToggle.value, [_ctx.$style.small]: _unref(prefer).s.reactionsDisplaySize === 'small', [_ctx.$style.large]: _unref(prefer).s.reactionsDisplaySize === 'large' }]]),
      onClick: _cache[0] || (_cache[0] = ($event: any) => (toggleReaction())),
      onContextmenu: _withModifiers(menu, ["prevent","stop"])
    }, [ _createVNode(MkReactionIcon, {
        style: "pointer-events: none;",
        class: _normalizeClass(_unref(prefer).s.limitWidthOfReaction ? _ctx.$style.limitWidth : ''),
        reaction: __props.reaction,
        emojiUrl: __props.reactionEmojis[__props.reaction.substring(1, __props.reaction.length - 1)]
      }, null, 10 /* CLASS, PROPS */, ["reaction", "emojiUrl"]), _createElementVNode("span", {
        class: _normalizeClass(_ctx.$style.count)
      }, _toDisplayString(__props.count), 1 /* TEXT */) ], 34 /* CLASS, NEED_HYDRATION */)), [ [_directive_ripple, canToggle.value] ])
}
}

})
