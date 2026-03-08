import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"

import * as Misskey from 'misskey-js'
import { getEmojiName } from '@@/js/emojilist.js'
import MkTooltip from './MkTooltip.vue'
import MkReactionIcon from '@/components/MkReactionIcon.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkReactionsViewer.details',
  props: {
    showing: { type: Boolean, required: true },
    reaction: { type: String, required: true },
    users: { type: Array, required: true },
    count: { type: Number, required: true },
    anchorElement: { type: null, required: true }
  },
  emits: ["closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
function getReactionName(reaction: string): string {
	const trimLocal = reaction.replace('@.', '');
	if (trimLocal.startsWith(':')) {
		return trimLocal;
	}
	return getEmojiName(reaction);
}

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")

  return (_openBlock(), _createBlock(MkTooltip, {
      ref: "tooltip",
      showing: __props.showing,
      anchorElement: __props.anchorElement,
      maxWidth: 340,
      onClosed: _cache[0] || (_cache[0] = ($event: any) => (emit('closed')))
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.root)
        }, [
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.reaction)
          }, [
            _createVNode(MkReactionIcon, {
              reaction: __props.reaction,
              class: _normalizeClass(_ctx.$style.reactionIcon),
              noStyle: true
            }, null, 8 /* PROPS */, ["reaction", "noStyle"]),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.reactionName)
            }, _toDisplayString(getReactionName(__props.reaction)), 1 /* TEXT */)
          ]),
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.users)
          }, [
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.users, (u) => {
              return (_openBlock(), _createElementBlock("div", {
                key: u.id,
                class: _normalizeClass(_ctx.$style.user)
              }, [
                _createVNode(_component_MkAvatar, {
                  class: _normalizeClass(_ctx.$style.avatar),
                  user: u
                }, null, 8 /* PROPS */, ["user"]),
                _createVNode(_component_MkUserName, {
                  user: u,
                  nowrap: true
                }, null, 8 /* PROPS */, ["user", "nowrap"])
              ]))
            }), 128 /* KEYED_FRAGMENT */)),
            (__props.count > 10)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: _normalizeClass(_ctx.$style.more)
              }, "+" + _toDisplayString(__props.count - 10), 1 /* TEXT */))
              : _createCommentVNode("v-if", true)
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["showing", "anchorElement", "maxWidth"]))
}
}

})
