import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList } from "vue"

import type { DiffHunk, DiffSkipBlock } from '#shared/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'Table',
  props: {
    hunks: { type: Array, required: true },
    type: { type: String, required: true },
    fileName: { type: String, required: false },
    wordWrap: { type: Boolean, required: false }
  },
  setup(__props: any) {

const props = __props
// provide diff context into child components
provide('diffContext', {
  fileStatus: computed(() => props.type),
  wordWrap: computed(() => props.wordWrap ?? false),
})

return (_ctx: any,_cache: any) => {
  const _component_DiffHunk = _resolveComponent("DiffHunk")
  const _component_DiffSkipBlock = _resolveComponent("DiffSkipBlock")

  return (_openBlock(), _createElementBlock("table", { class: "diff-table shiki font-mono text-sm w-full m-0 border-separate border-0 outline-none overflow-x-auto border-spacing-0" }, [ _createElementVNode("tbody", { class: "w-full box-border" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.hunks, (hunk, index) => {
          return (_openBlock(), _createElementBlock(_Fragment, { key: index }, [
            (hunk.type === 'hunk')
              ? (_openBlock(), _createBlock(_component_DiffHunk, {
                key: 0,
                hunk: hunk
              }, null, 8 /* PROPS */, ["hunk"]))
              : (_openBlock(), _createBlock(_component_DiffSkipBlock, {
                key: 1,
                count: hunk.count,
                content: hunk.content
              }, null, 8 /* PROPS */, ["count", "content"]))
          ], 64 /* STABLE_FRAGMENT */))
        }), 128 /* KEYED_FRAGMENT */)) ]) ]))
}
}

})
