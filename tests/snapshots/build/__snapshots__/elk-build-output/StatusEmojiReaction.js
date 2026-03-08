import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, mergeProps as _mergeProps, normalizeClass as _normalizeClass, unref as _unref } from "vue"

import type { mastodon } from 'masto'
import { getEmojiAttributes } from '~~/config/emojis'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusEmojiReaction',
  props: {
    status: { type: null, required: true },
    details: { type: Boolean, required: false }
  },
  setup(__props: any) {

const props = __props
const { status } = useStatusActions(props)
function isCustomEmoji(emoji: mastodon.v1.FedibirdEmojiReaction) {
  return !!emoji.staticUrl
}
function emojiCode(emoji: mastodon.v1.FedibirdEmojiReaction) {
  return `:${emoji.name}:`
}

return (_ctx: any,_cache: any) => {
  const _component_CommonLocalizedNumber = _resolveComponent("CommonLocalizedNumber")

  return (_openBlock(), _createElementBlock("div", {
      flex: "",
      "flex-wrap": "",
      "gap-1": "",
      class: "status-actions"
    }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(status).emojiReactions ?? [], (emoji, i) => {
        return (_openBlock(), _createElementBlock("button", {
          key: i,
          flex: "",
          "gap-1": "",
          p: "block-1 inline-2",
          "text-secondary": "",
          "btn-base": "",
          "rounded-1": "",
          class: _normalizeClass(emoji.me ? 'b-1 b-primary bg-primary-fade' : 'b b-white bg-gray-1 hover:bg-gray-1 hover:b-gray')
        }, [
          (isCustomEmoji(emoji))
            ? (_openBlock(), _createElementBlock("picture", {
              key: 0,
              class: "custom-emoji",
              "data-emoji-id": emoji.name,
              title: emojiCode(emoji)
            }, [
              _createElementVNode("source", {
                srcset: emoji.staticUrl,
                media: "(prefers-reduced-motion: reduce)"
              }, null, 8 /* PROPS */, ["srcset"]),
              _createElementVNode("img", {
                src: emoji.url,
                alt: emojiCode(emoji)
              }, null, 8 /* PROPS */, ["src", "alt"])
            ]))
            : (_openBlock(), _createElementBlock("picture", {
              key: 1,
              class: "custom-emoji",
              "data-emoji-id": emoji.name,
              title: emojiCode(emoji)
            }, [
              _createElementVNode("img", _mergeProps(_unref(getEmojiAttributes)(emoji.name), {
                alt: emojiCode(emoji)
              }), null, 16 /* FULL_PROPS */, ["alt"])
            ])),
          _createVNode(_component_CommonLocalizedNumber, {
            keypath: emoji.count.toString(),
            count: emoji.count
          }, null, 8 /* PROPS */, ["keypath", "count"])
        ], 2 /* CLASS */))
      }), 128 /* KEYED_FRAGMENT */)) ]))
}
}

})
