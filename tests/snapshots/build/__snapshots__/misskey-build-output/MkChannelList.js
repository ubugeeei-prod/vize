import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, resolveComponent as _resolveComponent, renderList as _renderList, withCtx as _withCtx } from "vue"

import * as Misskey from 'misskey-js'
import type { IPaginator, ExtractorFunction } from '@/utility/paginator.js'
import MkChannelPreview from '@/components/MkChannelPreview.vue'
import MkPagination from '@/components/MkPagination.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkChannelList',
  props: {
    paginator: { type: null, required: true },
    noGap: { type: Boolean, required: false },
    extractor: { type: null, required: false, default: (item: any) => item as Misskey.entities.Channel }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  const _component_MkResult = _resolveComponent("MkResult")

  return (_openBlock(), _createBlock(MkPagination, { paginator: __props.paginator }, {
      empty: _withCtx(() => [
        _createVNode(_component_MkResult, { type: "empty" })
      ]),
      default: _withCtx(({ items }) => [
        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
          return (_openBlock(), _createBlock(MkChannelPreview, {
            key: item.id,
            class: "_margin",
            channel: __props.extractor(item)
          }, null, 8 /* PROPS */, ["channel"]))
        }), 128 /* KEYED_FRAGMENT */))
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["paginator"]))
}
}

})
