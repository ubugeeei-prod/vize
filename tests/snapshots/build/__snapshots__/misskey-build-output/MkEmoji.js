import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue"

import { computed, inject } from 'vue'
import { colorizeEmoji, getEmojiName } from '@@/js/emojilist.js'
import { char2fluentEmojiFilePath, char2twemojiFilePath } from '@@/js/emoji-base.js'
import type { MenuItem } from '@/types/menu.js'
import * as os from '@/os.js'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'
import { i18n } from '@/i18n.js'
import { prefer } from '@/preferences.js'
import { DI } from '@/di.js'
import { mute as muteEmoji, unmute as unmuteEmoji, checkMuted as checkMutedEmoji } from '@/utility/emoji-mute.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkEmoji',
  props: {
    emoji: { type: String, required: true },
    menu: { type: Boolean, required: false },
    menuReaction: { type: Boolean, required: false },
    ignoreMuted: { type: Boolean, required: false }
  },
  setup(__props: any) {

const props = __props
const react = inject(DI.mfmEmojiReactCallback, null);
const char2path = prefer.s.emojiStyle === 'twemoji' ? char2twemojiFilePath : char2fluentEmojiFilePath;
const useOsNativeEmojis = computed(() => prefer.s.emojiStyle === 'native');
const url = computed(() => char2path(props.emoji));
const colorizedNativeEmoji = computed(() => colorizeEmoji(props.emoji));
const isMuted = checkMutedEmoji(props.emoji);
const shouldMute = computed(() => isMuted.value && !props.ignoreMuted);
// Searching from an array with 2000 items for every emoji felt like too energy-consuming, so I decided to do it lazily on pointerenter
function computeTitle(event: PointerEvent): void {
	(event.target as HTMLElement).title = getEmojiName(props.emoji);
}
function mute() {
	os.confirm({
		type: 'question',
		title: i18n.tsx.muteX({ x: props.emoji }),
	}).then(({ canceled }) => {
		if (canceled) {
			return;
		}
		muteEmoji(props.emoji);
	});
}
function unmute() {
	os.confirm({
		type: 'question',
		title: i18n.tsx.unmuteX({ x: props.emoji }),
	}).then(({ canceled }) => {
		if (canceled) {
			return;
		}
		unmuteEmoji(props.emoji);
	});
}
function onClick(ev: PointerEvent) {
	if (props.menu) {
		const menuItems: MenuItem[] = [];
		menuItems.push({
			type: 'label',
			text: props.emoji,
		}, {
			text: i18n.ts.copy,
			icon: 'ti ti-copy',
			action: () => {
				copyToClipboard(props.emoji);
			},
		});
		if (props.menuReaction && react) {
			menuItems.push({
				text: i18n.ts.doReaction,
				icon: 'ti ti-plus',
				action: () => {
					react(props.emoji);
				},
			});
		}
		menuItems.push({
			type: 'divider',
		}, isMuted.value ? {
			text: i18n.ts.emojiUnmute,
			icon: 'ti ti-mood-smile',
			action: () => {
				unmute();
			},
		} : {
			text: i18n.ts.emojiMute,
			icon: 'ti ti-mood-off',
			action: () => {
				mute();
			},
		});
		os.popupMenu(menuItems, ev.currentTarget ?? ev.target);
	}
}

return (_ctx: any,_cache: any) => {
  return (shouldMute.value)
      ? (_openBlock(), _createElementBlock("img", {
        key: 0,
        class: _normalizeClass(_ctx.$style.root),
        src: "/client-assets/unknown.png",
        alt: props.emoji,
        decoding: "async",
        onPointerenter: computeTitle,
        onClick: onClick
      }))
      : (!useOsNativeEmojis.value)
        ? (_openBlock(), _createElementBlock("img", {
          key: 1,
          class: _normalizeClass(_ctx.$style.root),
          src: url.value,
          alt: props.emoji,
          decoding: "async",
          onPointerenter: computeTitle,
          onClick: onClick
        }))
      : (_openBlock(), _createElementBlock("span", {
        key: 2,
        alt: props.emoji,
        onPointerenter: computeTitle,
        onClick: onClick
      }, _toDisplayString(colorizedNativeEmoji.value), 1 /* TEXT */))
}
}

})
