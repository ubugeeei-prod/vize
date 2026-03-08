import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"

import MkTooltip from './MkTooltip.vue'
import MkReactionIcon from '@/components/MkReactionIcon.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkReactionTooltip',
  props: {
    showing: { type: Boolean, required: true },
    reaction: { type: String, required: true },
    anchorElement: { type: null, required: true }
  },
  emits: ["closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit

return (_ctx: any,_cache: any) => {
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
          _createVNode(MkReactionIcon, {
            reaction: __props.reaction,
            class: _normalizeClass(_ctx.$style.icon),
            noStyle: true
          }, null, 8 /* PROPS */, ["reaction", "noStyle"]),
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.name)
          }, _toDisplayString(__props.reaction.replace('@.', '')), 1 /* TEXT */)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["showing", "anchorElement", "maxWidth"]))
}
}

})
