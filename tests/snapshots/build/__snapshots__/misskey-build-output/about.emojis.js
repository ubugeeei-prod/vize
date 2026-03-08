import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-search" })
import { watch, ref } from 'vue'
import * as Misskey from 'misskey-js'
import XEmoji from './emojis.emoji.vue'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import MkFoldableSection from '@/components/MkFoldableSection.vue'
import { customEmojis, customEmojiCategories } from '@/custom-emojis.js'
import { i18n } from '@/i18n.js'
import { $i } from '@/i.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'about.emojis',
  setup(__props) {

const q = ref('');
const searchEmojis = ref<Misskey.entities.EmojiSimple[] | null>(null);
function search() {
	if (q.value === '' || q.value == null) {
		searchEmojis.value = null;
		return;
	}
	const queryarry = q.value.match(/\:([a-z0-9_]*)\:/g);
	if (queryarry) {
		searchEmojis.value = customEmojis.value.filter(emoji =>
			queryarry.includes(`:${emoji.name}:`),
		);
	} else {
		searchEmojis.value = customEmojis.value.filter(emoji => emoji.name.includes(q.value) || emoji.aliases.includes(q.value));
	}
}
watch(q, () => {
	search();
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ (_unref($i) && (_unref($i).isModerator || _unref($i).policies.canManageCustomEmojis)) ? (_openBlock(), _createBlock(MkButton, {
          key: 0,
          primary: "",
          link: "",
          to: "/custom-emojis-manager"
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.manageCustomEmojis), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })) : _createCommentVNode("v-if", true), _createElementVNode("div", { class: "query" }, [ _createVNode(MkInput, {
          class: "",
          placeholder: _unref(i18n).ts.search,
          autocapitalize: "off",
          modelValue: q.value,
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((q).value = $event))
        }, {
          prefix: _withCtx(() => [
            _hoisted_1
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["placeholder", "modelValue"]) ]), (searchEmojis.value) ? (_openBlock(), _createBlock(MkFoldableSection, { key: 0 }, {
          header: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.searchResult), 1 /* TEXT */)
          ]),
          default: _withCtx(() => [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.emojis)
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(searchEmojis.value, (emoji) => {
                return (_openBlock(), _createBlock(XEmoji, {
                  key: emoji.name,
                  emoji: emoji
                }, null, 8 /* PROPS */, ["emoji"]))
              }), 128 /* KEYED_FRAGMENT */))
            ])
          ]),
          _: 1 /* STABLE */
        })) : _createCommentVNode("v-if", true), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(customEmojiCategories), (category) => {
        return (_openBlock(), _createBlock(MkFoldableSection, {
          key: category ?? '___root___',
          expanded: false
        }, {
          header: _withCtx(() => [
            _createTextVNode(_toDisplayString(category || _unref(i18n).ts.other), 1 /* TEXT */)
          ]),
          default: _withCtx(() => [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.emojis)
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(customEmojis).filter(e => e.category === category), (emoji) => {
                return (_openBlock(), _createBlock(XEmoji, {
                  key: emoji.name,
                  emoji: emoji
                }, null, 8 /* PROPS */, ["emoji"]))
              }), 128 /* KEYED_FRAGMENT */))
            ])
          ]),
          _: 2 /* DYNAMIC */
        }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["expanded"]))
      }), 128 /* KEYED_FRAGMENT */)) ]))
}
}

})
