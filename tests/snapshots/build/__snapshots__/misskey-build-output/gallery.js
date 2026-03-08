import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, renderList as _renderList, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { computed, markRaw } from 'vue'
import * as Misskey from 'misskey-js'
import MkGalleryPostPreview from '@/components/MkGalleryPostPreview.vue'
import MkPagination from '@/components/MkPagination.vue'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'gallery',
  props: {
    user: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const paginator = markRaw(new Paginator('users/gallery/posts', {
	limit: 6,
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
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.root)
          }, [
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (post) => {
              return (_openBlock(), _createBlock(MkGalleryPostPreview, {
                key: post.id,
                post: post,
                class: "post"
              }, null, 8 /* PROPS */, ["post"]))
            }), 128 /* KEYED_FRAGMENT */))
          ])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["paginator"]) ]))
}
}

})
