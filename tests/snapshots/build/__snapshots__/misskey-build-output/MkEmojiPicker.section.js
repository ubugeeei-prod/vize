import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-icons" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-folder ti-fw" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-icons ti-fw" })
import { ref, computed } from 'vue'
import { getEmojiName } from '@@/js/emojilist.js'
import type { Ref } from 'vue'
import type { CustomEmojiFolderTree } from '@@/js/emojilist.js'
import { i18n } from '@/i18n.js'
import { customEmojis } from '@/custom-emojis.js'
import MkEmojiPickerSection from '@/components/MkEmojiPicker.section.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkEmojiPicker.section',
  props: {
    emojis: { type: [Array, Object], required: true },
    disabledEmojis: { type: Object, required: false },
    initialShown: { type: Boolean, required: false },
    hasChildSection: { type: Boolean, required: false },
    customEmojiTree: { type: Array, required: false }
  },
  emits: ["chosen"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const emojis = computed(() => Array.isArray(props.emojis) ? props.emojis : props.emojis.value);
const shown = ref(!!props.initialShown);
/** @see MkEmojiPicker.vue */
function computeButtonTitle(ev: PointerEvent): void {
	const elm = ev.target as HTMLElement;
	const emoji = elm.dataset.emoji as string;
	elm.title = getEmojiName(emoji);
}
function nestedChosen(emoji: string, ev: PointerEvent) {
	emit('chosen', emoji, ev);
}

return (_ctx: any,_cache: any) => {
  const _component_MkCustomEmoji = _resolveComponent("MkCustomEmoji")
  const _component_MkEmoji = _resolveComponent("MkEmoji")
  const _directive_panel = _resolveDirective("panel")

  return (!__props.hasChildSection)
      ? (_openBlock(), _createElementBlock("section", {
        key: 0,
        style: "border-radius: 6px; border-bottom: 0.5px solid var(--MI_THEME-divider);"
      }, [ _createElementVNode("header", {
          class: "_acrylic",
          onClick: _cache[0] || (_cache[0] = ($event: any) => (shown.value = !shown.value))
        }, [ _createElementVNode("i", {
            class: _normalizeClass(["toggle ti-fw", shown.value ? 'ti ti-chevron-down' : 'ti ti-chevron-up'])
          }, null, 2 /* CLASS */), _createTextVNode(), _renderSlot(_ctx.$slots, "default"), _createTextVNode(" ("), _hoisted_1, _createTextVNode(":" + _toDisplayString(emojis.value.length) + ")\n\t", 1 /* TEXT */) ]), (shown.value) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "body"
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(emojis.value, (emoji) => {
              return (_openBlock(), _createElementBlock("button", {
                key: emoji,
                "data-emoji": emoji,
                class: "_button item",
                disabled: __props.disabledEmojis?.value.includes(emoji),
                onPointerenter: computeButtonTitle,
                onClick: _cache[1] || (_cache[1] = ($event: any) => (emit('chosen', emoji, $event)))
              }, [
                (emoji[0] === ':')
                  ? (_openBlock(), _createBlock(_component_MkCustomEmoji, {
                    key: 0,
                    class: "emoji",
                    name: emoji,
                    normal: true,
                    fallbackToImage: true
                  }, null, 8 /* PROPS */, ["name", "normal", "fallbackToImage"]))
                  : (_openBlock(), _createBlock(_component_MkEmoji, {
                    key: 1,
                    class: "emoji",
                    emoji: emoji,
                    normal: true
                  }, null, 8 /* PROPS */, ["emoji", "normal"]))
              ], 40 /* PROPS, NEED_HYDRATION */, ["data-emoji", "disabled"]))
            }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true) ]))
      : (_openBlock(), _createElementBlock("section", {
        key: 1,
        style: "border-radius: 6px; border-bottom: 0.5px solid var(--MI_THEME-divider);"
      }, [ _createElementVNode("header", {
          class: "_acrylic",
          onClick: _cache[2] || (_cache[2] = ($event: any) => (shown.value = !shown.value))
        }, [ _createElementVNode("i", {
            class: _normalizeClass(["toggle ti-fw", shown.value ? 'ti ti-chevron-down' : 'ti ti-chevron-up'])
          }, null, 2 /* CLASS */), _createTextVNode(), _renderSlot(_ctx.$slots, "default"), _createTextVNode(" ("), _hoisted_2, _createTextVNode(":" + _toDisplayString(__props.customEmojiTree?.length) + " ", 1 /* TEXT */), _hoisted_3, _createTextVNode(":" + _toDisplayString(emojis.value.length) + ")\n\t", 1 /* TEXT */) ]), (shown.value) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            style: "padding-left: 9px;"
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.customEmojiTree, (child) => {
              return (_openBlock(), _createBlock(MkEmojiPickerSection, {
                key: `custom:${child.value}`,
                initialShown: __props.initialShown,
                emojis: _unref(computed)(() => _unref(customEmojis).filter(e => e.category === child.category).map(e => `:${e.name}:`)),
                hasChildSection: child.children.length !== 0,
                customEmojiTree: child.children,
                onChosen: nestedChosen
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(child.value || _unref(i18n).ts.other), 1 /* TEXT */)
                ]),
                _: 2 /* DYNAMIC */
              }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["initialShown", "emojis", "hasChildSection", "customEmojiTree"]))
            }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), (shown.value) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "body"
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(emojis.value, (emoji) => {
              return (_openBlock(), _createElementBlock("button", {
                key: emoji,
                "data-emoji": emoji,
                class: "_button item",
                disabled: __props.disabledEmojis?.value.includes(emoji),
                onPointerenter: computeButtonTitle,
                onClick: _cache[3] || (_cache[3] = ($event: any) => (emit('chosen', emoji, $event)))
              }, [
                (emoji[0] === ':')
                  ? (_openBlock(), _createBlock(_component_MkCustomEmoji, {
                    key: 0,
                    class: "emoji",
                    name: emoji,
                    normal: true
                  }, null, 8 /* PROPS */, ["name", "normal"]))
                  : (_openBlock(), _createBlock(_component_MkEmoji, {
                    key: 1,
                    class: "emoji",
                    emoji: emoji,
                    normal: true
                  }, null, 8 /* PROPS */, ["emoji", "normal"]))
              ], 40 /* PROPS, NEED_HYDRATION */, ["data-emoji", "disabled"]))
            }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true) ]))
}
}

})
