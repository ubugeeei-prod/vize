import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:loader-2-line": "true", "animate-spin": "true" })
import type { CommandHandler } from '~/composables/command'
import type { CustomEmoji, Emoji } from '~/composables/tiptap/suggestion'
import { getEmojiMatchesInText } from '@iconify/utils/lib/emoji/replace/find'
import { emojiFilename, emojiPrefix, emojiRegEx } from '~~/config/emojis'
import { isCustomEmoji } from '~/composables/tiptap/suggestion'

export default /*@__PURE__*/_defineComponent({
  __name: 'TiptapEmojiList',
  props: {
    items: { type: Array, required: true },
    command: { type: null, required: true },
    isPending: { type: Boolean, required: false }
  },
  setup(__props: any, { expose: __expose }) {

const emojis = computed(() => {
  if (import.meta.server)
    return []

  return __props.items.map((item: CustomEmoji | Emoji) => {
    if (isCustomEmoji(item)) {
      return {
        title: item.shortcode,
        src: item.url,
        emoji: item,
      }
    }

    const skin = item.skins.find(skin => skin.native !== undefined)
    const match = getEmojiMatchesInText(emojiRegEx, skin!.native)[0]
    const file = emojiFilename(match)

    return {
      title: item.id,
      src: `/emojis/${emojiPrefix}/${file.filename}`,
      emoji: item,
    }
  })
})
const selectedIndex = ref(0)
watch(() => __props.items, () => {
  selectedIndex.value = 0
})
function onKeyDown(event: KeyboardEvent) {
  if (__props.items.length === 0)
    return false
  if (event.key === 'ArrowUp') {
    selectedIndex.value = ((selectedIndex.value + __props.items.length) - 1) % __props.items.length
    return true
  }
  else if (event.key === 'ArrowDown') {
    selectedIndex.value = (selectedIndex.value + 1) % __props.items.length
    return true
  }
  else if (event.key === 'Enter') {
    selectItem(selectedIndex.value)
    return true
  }
  return false
}
function selectItem(index: number) {
  const emoji = emojis.value[index]
  if (emoji)
    __props.command(emoji)
}
__expose({
  onKeyDown,
})

return (_ctx: any,_cache: any) => {
  const _component_SearchEmojiInfo = _resolveComponent("SearchEmojiInfo")
  const _component_CommonScrollIntoView = _resolveComponent("CommonScrollIntoView")

  return (__props.isPending || __props.items.length)
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        relative: "",
        "bg-base": "",
        "text-base": "",
        shadow: "",
        border: "~ base rounded",
        "text-sm": "",
        "py-2": "",
        "overflow-x-hidden": "",
        "overflow-y-auto": "",
        "max-h-100": "",
        "min-w-40": "",
        "max-w-50": ""
      }, [ (__props.isPending) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            flex: "",
            "gap-1": "",
            "items-center": "",
            p2: "",
            "animate-pulse": ""
          }, [ _hoisted_1, _createElementVNode("span", null, _toDisplayString(_ctx.$t('common.fetching')), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true), (__props.items.length) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(emojis.value, (item, index) => {
              return (_openBlock(), _createBlock(_component_CommonScrollIntoView, {
                key: index,
                active: index === selectedIndex.value,
                as: "button",
                class: _normalizeClass(index === selectedIndex.value ? 'bg-active' : 'text-secondary'),
                block: "",
                m0: "",
                "w-full": "",
                "text-left": "",
                px2: "",
                py1: "",
                onClick: _cache[0] || (_cache[0] = ($event: any) => (selectItem(index)))
              }, {
                default: _withCtx(() => [
                  _createVNode(_component_SearchEmojiInfo, { emoji: item }, null, 8 /* PROPS */, ["emoji"])
                ]),
                _: 2 /* DYNAMIC */
              }, 1034 /* CLASS, PROPS, DYNAMIC_SLOTS */, ["active"]))
            }), 128 /* KEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ]))
      : (_openBlock(), _createElementBlock("div", { key: 1 }))
}
}

})
