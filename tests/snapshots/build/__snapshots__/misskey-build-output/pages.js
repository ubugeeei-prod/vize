import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, renderList as _renderList, withCtx as _withCtx, unref as _unref } from "vue"

import { computed, markRaw } from 'vue'
import * as Misskey from 'misskey-js'
import MkPagePreview from '@/components/MkPagePreview.vue'
import MkPagination from '@/components/MkPagination.vue'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'pages',
  props: {
    user: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const paginator = markRaw(new Paginator('users/pages', {
	limit: 20,
	computedParams: computed(() => ({
		userId: props.user.id,
	})),
}));

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: "_spacer",
      style: "--MI_SPACER-w: 700px;"
    }, [ _createVNode(MkPagination, {
        paginator: _unref(paginator),
        withControl: ""
      }, {
        default: _withCtx(({items}) => [
          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (page) => {
            return (_openBlock(), _createBlock(MkPagePreview, {
              key: page.id,
              page: page,
              class: "_margin"
            }, null, 8 /* PROPS */, ["page"]))
          }), 128 /* KEYED_FRAGMENT */))
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["paginator"]) ]))
}
}

})
