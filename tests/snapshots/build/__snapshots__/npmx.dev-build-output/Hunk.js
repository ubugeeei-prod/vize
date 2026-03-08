import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, resolveComponent as _resolveComponent, renderList as _renderList } from "vue"

import type { DiffHunk as DiffHunkType } from '#shared/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'Hunk',
  props: {
    hunk: { type: null, required: true }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  const _component_DiffLine = _resolveComponent("DiffLine")

  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.hunk.lines, (line, index) => {
      return (_openBlock(), _createBlock(_component_DiffLine, {
        key: index,
        line: line
      }, null, 8 /* PROPS */, ["line"]))
    }), 128 /* KEYED_FRAGMENT */))
}
}

})
