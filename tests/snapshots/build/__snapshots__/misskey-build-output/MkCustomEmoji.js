import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue"

import { computed, defineAsyncComponent, inject, ref } from 'vue'
import type { MenuItem } from '@/types/menu.js'
import { getProxiedImageUrl, getStaticImageUrl } from '@/utility/media-proxy.js'
import { customEmojisMap } from '@/custom-emojis.js'
import * as os from '@/os.js'
import { misskeyApi, misskeyApiGet } from '@/utility/misskey-api.js'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'
import { i18n } from '@/i18n.js'
import MkCustomEmojiDetailedDialog from '@/components/MkCustomEmojiDetailedDialog.vue'
import { $i } from '@/i.js'
import { prefer } from '@/preferences.js'
import { DI } from '@/di.js'
import { makeEmojiMuteKey, mute as muteEmoji, unmute as unmuteEmoji, checkMuted as checkEmojiMuted } from '@/utility/emoji-mute'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkCustomEmoji',
  props: {
    name: { type: String, required: true },
    normal: { type: Boolean, required: false },
    noStyle: { type: Boolean, required: false },
    host: { type: String, required: false },
    url: { type: String, required: false },
    useOriginalSize: { type: Boolean, required: false },
    menu: { type: Boolean, required: false },
    menuReaction: { type: Boolean, required: false },
    fallbackToImage: { type: Boolean, required: false },
    ignoreMuted: { type: Boolean, required: false }
  },
  setup(__props: any) {

const props = __props
const react = inject(DI.mfmEmojiReactCallback);
const customEmojiName = computed(() => (props.name[0] === ':' ? props.name.substring(1, props.name.length - 1) : props.name).replace('@.', ''));
const isLocal = computed(() => !props.host && (customEmojiName.value.endsWith('@.') || !customEmojiName.value.includes('@')));
const emojiCodeToMute = makeEmojiMuteKey(props);
const isMuted = checkEmojiMuted(emojiCodeToMute);
const shouldMute = computed(() => !props.ignoreMuted && isMuted.value);
const rawUrl = computed(() => {
	if (props.url) {
		return props.url;
	}
	if (isLocal.value) {
		return customEmojisMap.get(customEmojiName.value)?.url ?? null;
	}
	return props.host ? `/emoji/${customEmojiName.value}@${props.host}.webp` : `/emoji/${customEmojiName.value}.webp`;
});
const url = computed(() => {
	if (rawUrl.value == null) return undefined;

	const proxied =
		(rawUrl.value.startsWith('/emoji/') || (props.useOriginalSize && isLocal.value))
			? rawUrl.value
			: getProxiedImageUrl(
				rawUrl.value,
				props.useOriginalSize ? undefined : 'emoji',
				false,
				true,
			);
	return prefer.s.disableShowingAnimatedImages
		? getStaticImageUrl(proxied)
		: proxied;
});
const alt = computed(() => `:${customEmojiName.value}:`);
const errored = ref(url.value == null);
function onClick(ev: PointerEvent) {
	if (props.menu) {
		const menuItems: MenuItem[] = [];
		menuItems.push({
			type: 'label',
			text: `:${props.name}:`,
		});
		if (isLocal.value) {
			menuItems.push({
				text: i18n.ts.copy,
				icon: 'ti ti-copy',
				action: () => {
					copyToClipboard(`:${props.name}:`);
				},
			});
		}
		if (props.menuReaction && react) {
			menuItems.push({
				text: i18n.ts.doReaction,
				icon: 'ti ti-plus',
				action: () => {
					react(`:${props.name}:`);
				},
			});
		}
		if (isLocal.value) {
			menuItems.push({
				type: 'divider',
			}, {
				text: i18n.ts.info,
				icon: 'ti ti-info-circle',
				action: async () => {
					const { dispose } = os.popup(MkCustomEmojiDetailedDialog, {
						emoji: await misskeyApiGet('emoji', {
							name: customEmojiName.value,
						}),
					}, {
						closed: () => dispose(),
					});
				},
			});
		}
		if (isMuted.value) {
			menuItems.push({
				text: i18n.ts.emojiUnmute,
				icon: 'ti ti-mood-smile',
				action: async () => {
					await unmute();
				},
			});
		} else {
			menuItems.push({
				text: i18n.ts.emojiMute,
				icon: 'ti ti-mood-off',
				action: async () => {
					await mute();
				},
			});
		}
		if (($i?.isModerator ?? $i?.isAdmin) && isLocal.value) {
			menuItems.push({
				text: i18n.ts.edit,
				icon: 'ti ti-pencil',
				action: async () => {
					await edit(props.name);
				},
			});
		}
		os.popupMenu(menuItems, ev.currentTarget ?? ev.target);
	}
}
async function edit(name: string) {
	const emoji = await misskeyApi('emoji', {
		name: name,
	});
	const { dispose } = await os.popupAsyncWithDialog(import('@/pages/emoji-edit-dialog.vue').then(x => x.default), {
		emoji: emoji,
	}, {
		closed: () => dispose(),
	});
}
function mute() {
	const titleEmojiName = isLocal.value
		? `:${customEmojiName.value}:`
		: emojiCodeToMute;
	os.confirm({
		type: 'question',
		title: i18n.tsx.muteX({ x: titleEmojiName }),
	}).then(({ canceled }) => {
		if (canceled) {
			return;
		}
		muteEmoji(emojiCodeToMute);
	});
}
function unmute() {
	const titleEmojiName = isLocal.value
		? `:${customEmojiName.value}:`
		: emojiCodeToMute;
	os.confirm({
		type: 'question',
		title: i18n.tsx.unmuteX({ x: titleEmojiName }),
	}).then(({ canceled }) => {
		if (canceled) {
			return;
		}
		unmuteEmoji(emojiCodeToMute);
	});
}

return (_ctx: any,_cache: any) => {
  return (shouldMute.value)
      ? (_openBlock(), _createElementBlock("img", {
        key: 0,
        class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.normal]: __props.normal, [_ctx.$style.noStyle]: __props.noStyle }]),
        src: "/client-assets/unknown.png",
        title: alt.value,
        draggable: "false",
        style: "-webkit-user-drag: none;",
        onClick: onClick
      }))
      : (errored.value && __props.fallbackToImage)
        ? (_openBlock(), _createElementBlock("img", {
          key: 1,
          class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.normal]: __props.normal, [_ctx.$style.noStyle]: __props.noStyle }]),
          src: "/client-assets/dummy.png",
          title: alt.value,
          draggable: "false",
          style: "-webkit-user-drag: none;"
        }))
      : (errored.value)
        ? (_openBlock(), _createElementBlock("span", { key: 2 }, ":" + _toDisplayString(customEmojiName.value) + ":", 1 /* TEXT */))
      : (_openBlock(), _createElementBlock("img", {
        key: 3,
        class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.normal]: __props.normal, [_ctx.$style.noStyle]: __props.noStyle }]),
        src: url.value,
        alt: alt.value,
        title: alt.value,
        decoding: "async",
        draggable: "false",
        onError: _cache[0] || (_cache[0] = ($event: any) => (errored.value = true)),
        onLoad: _cache[1] || (_cache[1] = ($event: any) => (errored.value = false)),
        onClick: onClick
      }))
}
}

})
