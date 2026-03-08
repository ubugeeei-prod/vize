import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"

import type { SearchResult } from '~/composables/masto/search'

export default /*@__PURE__*/_defineComponent({
  __name: 'AccountSearchResult',
  props: {
    result: { type: null, required: true },
    active: { type: Boolean, required: true }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  const _component_AccountInfo = _resolveComponent("AccountInfo")
  const _component_CommonScrollIntoView = _resolveComponent("CommonScrollIntoView")

  return (_openBlock(), _createBlock(_component_CommonScrollIntoView, {
      as: "div",
      active: __props.active,
      py2: "",
      block: "",
      px2: "",
      "aria-selected": __props.active,
      class: _normalizeClass({ 'bg-active': __props.active })
    }, {
      default: _withCtx(() => [
        (__props.result.type === 'account')
          ? (_openBlock(), _createBlock(_component_AccountInfo, {
            key: 0,
            account: __props.result.data
          }, null, 8 /* PROPS */, ["account"]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 10 /* CLASS, PROPS */, ["active", "aria-selected"]))
}
}

})
