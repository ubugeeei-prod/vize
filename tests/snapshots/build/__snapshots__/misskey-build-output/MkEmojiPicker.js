import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, setBlockTracking as _setBlockTracking, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-settings" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-clock ti-fw" })
const _hoisted_3 = { class: "_acrylic" }
const _hoisted_4 = { class: "_acrylic" }
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-asterisk ti-fw" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-mood-happy ti-fw" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-leaf ti-fw" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-hash ti-fw" })
import { ref, useTemplateRef, computed, watch, onMounted } from 'vue'
import * as Misskey from 'misskey-js'
import { emojilist, emojiCharByCategory, unicodeEmojiCategories as categories, getEmojiName, getUnicodeEmoji } from '@@/js/emojilist.js'
import type { UnicodeEmojiDef, CustomEmojiFolderTree } from '@@/js/emojilist.js'
import XSection from '@/components/MkEmojiPicker.section.vue'
import MkRippleEffect from '@/components/MkRippleEffect.vue'
import * as os from '@/os.js'
import { isTouchUsing } from '@/utility/touch.js'
import { deviceKind } from '@/utility/device-kind.js'
import { i18n } from '@/i18n.js'
import { store } from '@/store.js'
import { customEmojiCategories, customEmojis, customEmojisMap } from '@/custom-emojis.js'
import { $i } from '@/i.js'
import { checkReactionPermissions } from '@/utility/check-reaction-permissions.js'
import { prefer } from '@/preferences.js'
import { useRouter } from '@/router.js'
import { haptic } from '@/utility/haptic.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkEmojiPicker',
  props: {
    showPinned: { type: Boolean, required: false, default: true },
    pinnedEmojis: { type: Array, required: false },
    maxHeight: { type: Number, required: false },
    asDrawer: { type: Boolean, required: false },
    asWindow: { type: Boolean, required: false },
    asReactionPicker: { type: Boolean, required: false },
    targetNote: { type: null, required: false }
  },
  emits: ["chosen", "esc"],
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const router = useRouter();
const searchEl = useTemplateRef('searchEl');
const emojisEl = useTemplateRef('emojisEl');
const {
	emojiPickerScale,
	emojiPickerWidth,
	emojiPickerHeight,
} = prefer.r;
const recentlyUsedEmojis = store.r.recentlyUsedEmojis;
const recentlyUsedEmojisDef = computed(() => {
	return recentlyUsedEmojis.value.map(getDef);
});
const pinnedEmojisDef = computed(() => {
	return pinned.value?.map(getDef);
});
const pinned = computed(() => props.pinnedEmojis);
const size = computed(() => emojiPickerScale.value);
const width = computed(() => emojiPickerWidth.value);
const height = computed(() => emojiPickerHeight.value);
const q = ref<string>('');
const searchResultCustom = ref<Misskey.entities.EmojiSimple[]>([]);
const searchResultUnicode = ref<UnicodeEmojiDef[]>([]);
const tab = ref<'index' | 'custom' | 'unicode' | 'tags'>('index');
const customEmojiFolderRoot: CustomEmojiFolderTree = { value: '', category: '', children: [] };
function parseAndMergeCategories(input: string, root: CustomEmojiFolderTree): CustomEmojiFolderTree {
	const parts = input.split('/').map(p => p.trim());
	let currentNode: CustomEmojiFolderTree = root;
	for (const part of parts) {
		let existingNode = currentNode.children.find((node) => node.value === part);
		if (!existingNode) {
			const newNode: CustomEmojiFolderTree = { value: part, category: input, children: [] };
			currentNode.children.push(newNode);
			existingNode = newNode;
		}
		currentNode = existingNode;
	}
	return currentNode;
}
customEmojiCategories.value.forEach(ec => {
	if (ec !== null) {
		parseAndMergeCategories(ec, customEmojiFolderRoot);
	}
});
parseAndMergeCategories('', customEmojiFolderRoot);
watch(q, () => {
	if (emojisEl.value) emojisEl.value.scrollTop = 0;
	if (q.value === '') {
		searchResultCustom.value = [];
		searchResultUnicode.value = [];
		return;
	}
	const newQ = q.value.replace(/:/g, '').toLowerCase();
	const searchCustom = () => {
		const max = 100;
		const emojis = customEmojis.value;
		const matches = new Set<Misskey.entities.EmojiSimple>();

		const exactMatch = emojis.find(emoji => emoji.name === newQ);
		if (exactMatch) matches.add(exactMatch);

		if (newQ.includes(' ')) { // AND検索
			const keywords = newQ.split(' ');

			// 名前にキーワードが含まれている
			for (const emoji of emojis) {
				if (keywords.every(keyword => emoji.name.includes(keyword))) {
					matches.add(emoji);
					if (matches.size >= max) break;
				}
			}
			if (matches.size >= max) return matches;

			// 名前またはエイリアスにキーワードが含まれている
			for (const emoji of emojis) {
				if (keywords.every(keyword => emoji.name.includes(keyword) || emoji.aliases.some(alias => alias.includes(keyword)))) {
					matches.add(emoji);
					if (matches.size >= max) break;
				}
			}
		} else {
			if (customEmojisMap.has(newQ)) {
				matches.add(customEmojisMap.get(newQ)!);
			}
			if (matches.size >= max) return matches;

			for (const emoji of emojis) {
				if (emoji.aliases.some(alias => alias === newQ)) {
					matches.add(emoji);
					if (matches.size >= max) break;
				}
			}
			if (matches.size >= max) return matches;

			for (const emoji of emojis) {
				if (emoji.name.startsWith(newQ)) {
					matches.add(emoji);
					if (matches.size >= max) break;
				}
			}
			if (matches.size >= max) return matches;

			for (const emoji of emojis) {
				if (emoji.aliases.some(alias => alias.startsWith(newQ))) {
					matches.add(emoji);
					if (matches.size >= max) break;
				}
			}
			if (matches.size >= max) return matches;

			for (const emoji of emojis) {
				if (emoji.name.includes(newQ)) {
					matches.add(emoji);
					if (matches.size >= max) break;
				}
			}
			if (matches.size >= max) return matches;

			for (const emoji of emojis) {
				if (emoji.aliases.some(alias => alias.includes(newQ))) {
					matches.add(emoji);
					if (matches.size >= max) break;
				}
			}
		}

		return matches;
	};
	const searchUnicode = () => {
		const max = 100;
		const emojis = emojilist;
		const matches = new Set<UnicodeEmojiDef>();

		const exactMatch = emojis.find(emoji => emoji.name === newQ);
		if (exactMatch) matches.add(exactMatch);

		if (newQ.includes(' ')) { // AND検索
			const keywords = newQ.split(' ');

			for (const emoji of emojis) {
				if (keywords.every(keyword => emoji.name.includes(keyword))) {
					matches.add(emoji);
					if (matches.size >= max) break;
				}
			}
			if (matches.size >= max) return matches;

			for (const index of Object.values(store.s.additionalUnicodeEmojiIndexes)) {
				for (const emoji of emojis) {
					if (keywords.every(keyword => index[emoji.char]?.some(k => k.includes(keyword)))) {
						matches.add(emoji);
						if (matches.size >= max) break;
					}
				}
			}
		} else {
			for (const emoji of emojis) {
				if (emoji.name.startsWith(newQ)) {
					matches.add(emoji);
					if (matches.size >= max) break;
				}
			}
			if (matches.size >= max) return matches;

			for (const index of Object.values(store.s.additionalUnicodeEmojiIndexes)) {
				for (const emoji of emojis) {
					if (index[emoji.char]?.some(k => k.startsWith(newQ))) {
						matches.add(emoji);
						if (matches.size >= max) break;
					}
				}
			}

			for (const emoji of emojis) {
				if (emoji.name.includes(newQ)) {
					matches.add(emoji);
					if (matches.size >= max) break;
				}
			}
			if (matches.size >= max) return matches;

			for (const index of Object.values(store.s.additionalUnicodeEmojiIndexes)) {
				for (const emoji of emojis) {
					if (index[emoji.char]?.some(k => k.includes(newQ))) {
						matches.add(emoji);
						if (matches.size >= max) break;
					}
				}
			}
		}

		return matches;
	};
	searchResultCustom.value = Array.from(searchCustom());
	searchResultUnicode.value = Array.from(searchUnicode());
});
function canReact(emoji: Misskey.entities.EmojiSimple | UnicodeEmojiDef | string): boolean {
	return !props.targetNote || checkReactionPermissions($i!, props.targetNote, emoji);
}
function filterCategory(emoji: Misskey.entities.EmojiSimple, category: string): boolean {
	return category === '' ? (emoji.category === 'null' || !emoji.category) : emoji.category === category;
}
function focus() {
	if (!['smartphone', 'tablet'].includes(deviceKind) && !isTouchUsing) {
		searchEl.value?.focus({
			preventScroll: true,
		});
	}
}
function reset() {
	if (emojisEl.value) emojisEl.value.scrollTop = 0;
	q.value = '';
}
function getKey(emoji: string | Misskey.entities.EmojiSimple | UnicodeEmojiDef): string {
	return typeof emoji === 'string' ? emoji : 'char' in emoji ? emoji.char : `:${emoji.name}:`;
}
function getDef(emoji: string): string | Misskey.entities.EmojiSimple | UnicodeEmojiDef {
	if (emoji.includes(':')) {
		// カスタム絵文字が存在する場合はその情報を持つオブジェクトを返し、
		// サーバの管理画面から削除された等で情報が見つからない場合は名前の文字列をそのまま返しておく（undefinedを返すとエラーになるため）
		const name = emoji.replaceAll(':', '');
		return customEmojisMap.get(name) ?? emoji;
	} else {
		return getUnicodeEmoji(emoji);
	}
}
/** @see MkEmojiPicker.section.vue */
function computeButtonTitle(ev: PointerEvent): void {
	const elm = ev.target as HTMLElement;
	const emoji = elm.dataset.emoji as string;
	elm.title = getEmojiName(emoji);
}
function chosen(emoji: string | Misskey.entities.EmojiSimple | UnicodeEmojiDef, ev?: PointerEvent) {
	const el = ev && (ev.currentTarget ?? ev.target) as HTMLElement | null | undefined;
	if (el && prefer.s.animation) {
		const rect = el.getBoundingClientRect();
		const x = rect.left + (el.offsetWidth / 2);
		const y = rect.top + (el.offsetHeight / 2);
		const { dispose } = os.popup(MkRippleEffect, { x, y }, {
			end: () => dispose(),
		});
	}
	const key = getKey(emoji);
	emit('chosen', key);
	haptic();
	// 最近使った絵文字更新
	if (!pinned.value?.includes(key)) {
		let recents = store.s.recentlyUsedEmojis;
		recents = recents.filter((emoji) => emoji !== key);
		recents.unshift(key);
		store.set('recentlyUsedEmojis', recents.splice(0, 32));
	}
}
function input(): void {
	// Using custom input event instead of v-model to respond immediately on
	// Android, where composition happens on all languages
	// (v-model does not update during composition)
	q.value = searchEl.value?.value.trim() ?? '';
}
function paste(event: ClipboardEvent): void {
	const pasted = event.clipboardData?.getData('text') ?? '';
	if (done(pasted)) {
		event.preventDefault();
	}
}
function onKeydown(ev: KeyboardEvent) {
	if (ev.isComposing || ev.key === 'Process' || ev.keyCode === 229) return;
	if (ev.key === 'Enter') {
		ev.preventDefault();
		ev.stopPropagation();
		done();
	}
	if (ev.key === 'Escape') {
		ev.preventDefault();
		ev.stopPropagation();
		emit('esc');
	}
}
function done(query?: string): boolean | void {
	if (query == null) query = q.value;
	if (query == null || typeof query !== 'string') return;
	const q2 = query.replace(/:/g, '');
	const exactMatchCustom = customEmojisMap.get(q2);
	if (exactMatchCustom) {
		chosen(exactMatchCustom);
		return true;
	}
	const exactMatchUnicode = emojilist.find(emoji => emoji.char === q2 || emoji.name === q2);
	if (exactMatchUnicode) {
		chosen(exactMatchUnicode);
		return true;
	}
	if (searchResultCustom.value.length > 0) {
		chosen(searchResultCustom.value[0]);
		return true;
	}
	if (searchResultUnicode.value.length > 0) {
		chosen(searchResultUnicode.value[0]);
		return true;
	}
}
function settings() {
	emit('esc');
	router.push('/settings/emoji-palette');
}
onMounted(() => {
	focus();
});
__expose({
	focus,
	reset,
})

return (_ctx: any,_cache: any) => {
  const _component_MkCustomEmoji = _resolveComponent("MkCustomEmoji")
  const _component_MkEmoji = _resolveComponent("MkEmoji")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["omfetrab", ['s' + size.value, 'w' + width.value, 'h' + height.value, { asDrawer: __props.asDrawer, asWindow: __props.asWindow }]]),
      style: _normalizeStyle({ maxHeight: __props.maxHeight ? __props.maxHeight + 'px' : undefined })
    }, [ _createElementVNode("input", {
        ref_key: "searchEl", ref: searchEl,
        value: q.value,
        "data-prevent-emoji-insert": "",
        class: _normalizeClass(["search", { filled: q.value != null && q.value != '' }]),
        placeholder: _unref(i18n).ts.search,
        type: "search",
        autocapitalize: "off",
        onInput: _cache[0] || (_cache[0] = ($event: any) => (input())),
        onPaste: _withModifiers(paste, ["stop"]),
        onKeydown: onKeydown
      }, null, 42 /* CLASS, PROPS, NEED_HYDRATION */, ["value", "placeholder"]), _createTextVNode("\n\t" + "\n\t"), _createElementVNode("div", {
        ref_key: "emojisEl", ref: emojisEl,
        class: "emojis",
        tabindex: "-1"
      }, [ _createElementVNode("section", { class: "result" }, [ (searchResultCustom.value.length > 0) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "body"
            }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(searchResultCustom.value, (emoji) => {
                return (_openBlock(), _createElementBlock("button", {
                  key: emoji.name,
                  class: "_button item",
                  disabled: !canReact(emoji),
                  title: emoji.name,
                  tabindex: "0",
                  onClick: _cache[1] || (_cache[1] = ($event: any) => (chosen(emoji, $event)))
                }, [
                  _createVNode(_component_MkCustomEmoji, {
                    class: "emoji",
                    name: emoji.name,
                    fallbackToImage: true
                  }, null, 8 /* PROPS */, ["name", "fallbackToImage"])
                ], 8 /* PROPS */, ["disabled", "title"]))
              }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), (searchResultUnicode.value.length > 0) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "body"
            }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(searchResultUnicode.value, (emoji) => {
                return (_openBlock(), _createElementBlock("button", {
                  key: emoji.name,
                  class: "_button item",
                  title: emoji.name,
                  tabindex: "0",
                  onClick: _cache[2] || (_cache[2] = ($event: any) => (chosen(emoji, $event)))
                }, [
                  _createVNode(_component_MkEmoji, {
                    class: "emoji",
                    emoji: emoji.char
                  }, null, 8 /* PROPS */, ["emoji"])
                ], 8 /* PROPS */, ["title"]))
              }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true) ]), (tab.value === 'index') ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "group index"
          }, [ (__props.showPinned && (pinned.value && pinned.value.length > 0)) ? (_openBlock(), _createElementBlock("section", { key: 0 }, [ _createElementVNode("div", { class: "body" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(pinnedEmojisDef.value, (emoji) => {
                    return (_openBlock(), _createElementBlock("button", {
                      key: getKey(emoji),
                      "data-emoji": getKey(emoji),
                      class: "_button item",
                      disabled: !canReact(emoji),
                      tabindex: "0",
                      onPointerenter: computeButtonTitle,
                      onClick: _cache[3] || (_cache[3] = ($event: any) => (chosen(emoji, $event)))
                    }, [
                      (!emoji.hasOwnProperty('char'))
                        ? (_openBlock(), _createBlock(_component_MkCustomEmoji, {
                          key: 0,
                          class: "emoji",
                          name: getKey(emoji),
                          normal: true
                        }, null, 8 /* PROPS */, ["name", "normal"]))
                        : (_openBlock(), _createBlock(_component_MkEmoji, {
                          key: 1,
                          class: "emoji",
                          emoji: getKey(emoji),
                          normal: true
                        }, null, 8 /* PROPS */, ["emoji", "normal"]))
                    ], 40 /* PROPS, NEED_HYDRATION */, ["data-emoji", "disabled"]))
                  }), 128 /* KEYED_FRAGMENT */)), _createElementVNode("button", {
                    class: "_button config",
                    onClick: settings
                  }, [ _hoisted_1 ]) ]) ])) : _createCommentVNode("v-if", true), _createElementVNode("section", null, [ _createElementVNode("header", { class: "_acrylic" }, [ _hoisted_2, _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.recentUsed), 1 /* TEXT */) ]), _createElementVNode("div", { class: "body" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(recentlyUsedEmojisDef.value, (emoji) => {
                  return (_openBlock(), _createElementBlock("button", {
                    key: getKey(emoji),
                    class: "_button item",
                    disabled: !canReact(emoji),
                    "data-emoji": getKey(emoji),
                    onPointerenter: computeButtonTitle,
                    onClick: _cache[4] || (_cache[4] = ($event: any) => (chosen(emoji, $event)))
                  }, [
                    (!emoji.hasOwnProperty('char'))
                      ? (_openBlock(), _createBlock(_component_MkCustomEmoji, {
                        key: 0,
                        class: "emoji",
                        name: getKey(emoji),
                        normal: true
                      }, null, 8 /* PROPS */, ["name", "normal"]))
                      : (_openBlock(), _createBlock(_component_MkEmoji, {
                        key: 1,
                        class: "emoji",
                        emoji: getKey(emoji),
                        normal: true
                      }, null, 8 /* PROPS */, ["emoji", "normal"]))
                  ], 40 /* PROPS, NEED_HYDRATION */, ["disabled", "data-emoji"]))
                }), 128 /* KEYED_FRAGMENT */)) ]) ]) ])) : _createCommentVNode("v-if", true), _cache[5] || ( _setBlockTracking(-1, true), (_cache[5] = _createElementVNode("div", {
            class: "group"
          }, [ _createElementVNode("header", _hoisted_3, _toDisplayString(_unref(i18n).ts.customEmojis), 1 /* TEXT */), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(customEmojiFolderRoot.children, (child) => {
              return (_openBlock(), _createBlock(XSection, {
                key: `custom:${child.value}`,
                initialShown: false,
                emojis: _unref(computed)(() => _unref(customEmojis).filter(e => filterCategory(e, child.value)).map(e => `:${e.name}:`)),
                disabledEmojis: _unref(computed)(() => _unref(customEmojis).filter(e => filterCategory(e, child.value)).filter(e => !canReact(e)).map(e => `:${e.name}:`)),
                hasChildSection: child.children.length !== 0,
                customEmojiTree: child.children,
                onChosen: chosen
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(child.value || _unref(i18n).ts.other), 1 /* TEXT */)
                ]),
                _: 2 /* DYNAMIC */
              }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["initialShown", "emojis", "disabledEmojis", "hasChildSection", "customEmojiTree"]))
            }), 128 /* KEYED_FRAGMENT */)) ])).cacheIndex = 5, _setBlockTracking(1), _cache[5] ), _cache[6] || ( _setBlockTracking(-1, true), (_cache[6] = _createElementVNode("div", {
            class: "group"
          }, [ _createElementVNode("header", _hoisted_4, _toDisplayString(_unref(i18n).ts.emoji), 1 /* TEXT */), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(categories), (category) => {
              return (_openBlock(), _createBlock(XSection, {
                key: category,
                emojis: _unref(emojiCharByCategory).get(category) ?? [],
                hasChildSection: false,
                onChosen: chosen
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(category), 1 /* TEXT */)
                ]),
                _: 2 /* DYNAMIC */
              }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["emojis", "hasChildSection"]))
            }), 128 /* KEYED_FRAGMENT */)) ])).cacheIndex = 6, _setBlockTracking(1), _cache[6] ) ], 512 /* NEED_PATCH */), _createElementVNode("div", { class: "tabs" }, [ _createElementVNode("button", {
          class: _normalizeClass(["_button tab", { active: tab.value === 'index' }]),
          onClick: _cache[7] || (_cache[7] = ($event: any) => (tab.value = 'index'))
        }, [ _hoisted_5 ], 2 /* CLASS */), _createElementVNode("button", {
          class: _normalizeClass(["_button tab", { active: tab.value === 'custom' }]),
          onClick: _cache[8] || (_cache[8] = ($event: any) => (tab.value = 'custom'))
        }, [ _hoisted_6 ], 2 /* CLASS */), _createElementVNode("button", {
          class: _normalizeClass(["_button tab", { active: tab.value === 'unicode' }]),
          onClick: _cache[9] || (_cache[9] = ($event: any) => (tab.value = 'unicode'))
        }, [ _hoisted_7 ], 2 /* CLASS */), _createElementVNode("button", {
          class: _normalizeClass(["_button tab", { active: tab.value === 'tags' }]),
          onClick: _cache[10] || (_cache[10] = ($event: any) => (tab.value = 'tags'))
        }, [ _hoisted_8 ], 2 /* CLASS */) ]) ], 6 /* CLASS, STYLE */))
}
}

})
