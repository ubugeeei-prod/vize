import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, renderList as _renderList, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import * as Misskey from 'misskey-js'
import type { IPaginator, ExtractorFunction } from '@/utility/paginator.js'
import MkUserInfo from '@/components/MkUserInfo.vue'
import MkPagination from '@/components/MkPagination.vue'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkUserList',
  props: {
    paginator: { type: null, required: true },
    noGap: { type: Boolean, required: false },
    extractor: { type: null, required: false, default: (item: any) => item as Misskey.entities.UserDetailed }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  const _component_MkResult = _resolveComponent("MkResult")

  return (_openBlock(), _createBlock(MkPagination, { paginator: __props.paginator }, {
      empty: _withCtx(() => [
        _createVNode(_component_MkResult, {
          type: "empty",
          text: _unref(i18n).ts.noUsers
        }, null, 8 /* PROPS */, ["text"])
      ]),
      default: _withCtx(({ items }) => [
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.root)
        }, [
          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
            return (_openBlock(), _createBlock(MkUserInfo, {
              key: item.id,
              user: __props.extractor(item)
            }, null, 8 /* PROPS */, ["user"]))
          }), 128 /* KEYED_FRAGMENT */))
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["paginator"]))
}
}

})
