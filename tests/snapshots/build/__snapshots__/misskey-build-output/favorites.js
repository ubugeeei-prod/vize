import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, renderList as _renderList, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { markRaw } from 'vue'
import MkPagination from '@/components/MkPagination.vue'
import MkNote from '@/components/MkNote.vue'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'favorites',
  setup(__props) {

const paginator = markRaw(new Paginator('i/favorites', {
	limit: 10,
}));
definePage(() => ({
	title: i18n.ts.favorites,
	icon: 'ti ti-star',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkResult = _resolveComponent("MkResult")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, null, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 800px;"
        }, [
          _createVNode(MkPagination, { paginator: _unref(paginator) }, {
            empty: _withCtx(() => [
              _createVNode(_component_MkResult, {
                type: "empty",
                text: _unref(i18n).ts.noNotes
              }, null, 8 /* PROPS */, ["text"])
            ]),
            default: _withCtx(({ items }) => [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
                return (_openBlock(), _createBlock(MkNote, {
                  key: item.id,
                  note: item.note,
                  class: _normalizeClass(_ctx.$style.note)
                }, null, 8 /* PROPS */, ["note"]))
              }), 128 /* KEYED_FRAGMENT */))
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["paginator"])
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
