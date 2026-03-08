import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("hr")
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-cloud-cog" })
import { ref } from 'vue'
import type { MenuItem } from '@/types/menu'
import MkButton from '@/components/MkButton.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { prefer } from '@/preferences.js'
import { mute as muteEmoji, unmute as unmuteEmoji, extractCustomEmojiName as customEmojiName, extractCustomEmojiHost as customEmojiHost } from '@/utility/emoji-mute.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'mute-block.emoji-mute',
  setup(__props) {

const emojis = prefer.model('mutingEmojis');
function getHTMLElement(ev: PointerEvent): HTMLElement {
	const target = ev.currentTarget ?? ev.target;
	return target as HTMLElement;
}
function add(ev: PointerEvent) {
	os.pickEmoji(getHTMLElement(ev), { showPinned: false }).then((emoji) => {
		if (emoji) {
			muteEmoji(emoji);
		}
	});
}
function onEmojiClick(ev: PointerEvent, emoji: string) {
	const menuItems : MenuItem[] = [{
		type: 'label',
		text: emoji,
	}, {
		text: i18n.ts.emojiUnmute,
		icon: 'ti ti-mood-off',
		action: () => unmute(emoji),
	}];
	os.popupMenu(menuItems, ev.currentTarget ?? ev.target);
}
function unmute(emoji: string) {
	os.confirm({
		type: 'question',
		title: i18n.tsx.unmuteX({ x: emoji }),
	}).then(({ canceled }) => {
		if (canceled) {
			return;
		}
		unmuteEmoji(emoji);
	});
}
const syncEnabled = ref(prefer.isSyncEnabled('mutingEmojis'));
function changeSyncEnabled(value: boolean) {
	if (value) {
		prefer.enableSync('mutingEmojis').then((res) => {
			if (res == null) return;
			if (res.enabled) syncEnabled.value = true;
		});
	} else {
		prefer.disableSync('mutingEmojis');
		syncEnabled.value = false;
	}
}

return (_ctx: any,_cache: any) => {
  const _component_MkCustomEmoji = _resolveComponent("MkCustomEmoji")
  const _component_MkEmoji = _resolveComponent("MkEmoji")
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_SearchMarker = _resolveComponent("SearchMarker")

  return (_openBlock(), _createElementBlock("div", { class: "_gaps_m" }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.emojis)
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(emojis), (emoji) => {
          return (_openBlock(), _createElementBlock("div", {
            key: `emojiMute-${emoji}`,
            class: _normalizeClass(_ctx.$style.emoji),
            onClick: _cache[0] || (_cache[0] = ($event: any) => (onEmojiClick($event, emoji)))
          }, [
            (emoji.startsWith(':'))
              ? (_openBlock(), _createBlock(_component_MkCustomEmoji, {
                key: 0,
                name: _unref(customEmojiName)(emoji),
                host: _unref(customEmojiHost)(emoji),
                normal: true,
                menu: false,
                menuReaction: false,
                ignoreMuted: true
              }, null, 8 /* PROPS */, ["name", "host", "normal", "menu", "menuReaction", "ignoreMuted"]))
              : (_openBlock(), _createBlock(_component_MkEmoji, {
                key: 1,
                emoji: emoji,
                menu: false,
                menuReaction: false,
                ignoreMuted: true
              }, null, 8 /* PROPS */, ["emoji", "menu", "menuReaction", "ignoreMuted"]))
          ]))
        }), 128 /* KEYED_FRAGMENT */)) ]), _createVNode(MkButton, {
        primary: "",
        inline: "",
        onClick: add
      }, {
        default: _withCtx(() => [
          _hoisted_1,
          _createTextVNode(" "),
          _createTextVNode(_toDisplayString(_unref(i18n).ts.add), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }), _hoisted_2, _createVNode(_component_SearchMarker, { keywords: ['sync', 'devices'] }, {
        default: _withCtx(() => [
          _createVNode(MkSwitch, {
            modelValue: syncEnabled.value,
            "onUpdate:modelValue": changeSyncEnabled
          }, {
            label: _withCtx(() => [
              _hoisted_3,
              _createTextVNode(" "),
              _createVNode(_component_SearchLabel, null, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.syncBetweenDevices), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["keywords"]) ]))
}
}

})
