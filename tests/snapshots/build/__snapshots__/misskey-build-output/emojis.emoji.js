import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue"

import * as Misskey from 'misskey-js'
import type { MenuItem } from '@/types/menu.js'
import * as os from '@/os.js'
import { misskeyApiGet } from '@/utility/misskey-api.js'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'
import { i18n } from '@/i18n.js'
import MkCustomEmojiDetailedDialog from '@/components/MkCustomEmojiDetailedDialog.vue'
import { $i } from '@/i.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'emojis.emoji',
  props: {
    emoji: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
function menu(ev: PointerEvent) {
	const menuItems: MenuItem[] = [];
	menuItems.push({
		type: 'label',
		text: ':' + props.emoji.name + ':',
	}, {
		text: i18n.ts.copy,
		icon: 'ti ti-copy',
		action: () => {
			copyToClipboard(`:${props.emoji.name}:`);
		},
	}, {
		text: i18n.ts.info,
		icon: 'ti ti-info-circle',
		action: async () => {
			const { dispose } = os.popup(MkCustomEmojiDetailedDialog, {
				emoji: await misskeyApiGet('emoji', {
					name: props.emoji.name,
				}),
			}, {
				closed: () => dispose(),
			});
		},
	});
	if ($i?.isModerator ?? $i?.isAdmin) {
		menuItems.push({
			text: i18n.ts.edit,
			icon: 'ti ti-pencil',
			action: async () => {
				const detailedEmoji = await misskeyApiGet('emoji', {
					name: props.emoji.name,
				});
				const { dispose } = await os.popupAsyncWithDialog(import('@/pages/emoji-edit-dialog.vue').then(x => x.default), {
					emoji: detailedEmoji,
				}, {
					closed: () => dispose(),
				});
			},
		});
	}
	os.popupMenu(menuItems, ev.currentTarget ?? ev.target);
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("button", {
      class: _normalizeClass(["_button", _ctx.$style.root]),
      onClick: menu
    }, [ _createElementVNode("img", {
        src: __props.emoji.url,
        class: _normalizeClass(_ctx.$style.img),
        loading: "lazy"
      }, null, 8 /* PROPS */, ["src"]), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.body)
      }, [ _createElementVNode("div", {
          class: _normalizeClass(["_monospace", _ctx.$style.name])
        }, _toDisplayString(__props.emoji.name), 1 /* TEXT */), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.info)
        }, _toDisplayString(__props.emoji.aliases.join(' ')), 1 /* TEXT */) ]) ]))
}
}

})
