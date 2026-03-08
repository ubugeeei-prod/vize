import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { markRaw } from 'vue'
import * as Misskey from 'misskey-js'
import MkPagination from '@/components/MkPagination.vue'
import MkStickyContainer from '@/components/global/MkStickyContainer.vue'
import MkAvatars from '@/components/MkAvatars.vue'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'lists',
  props: {
    user: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const paginator = markRaw(new Paginator('users/lists/list', {
	noPaging: true,
	limit: 10,
	params: {
		userId: props.user.id,
	},
}));

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(), _createBlock(MkStickyContainer, null, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 700px;"
        }, [
          _createElementVNode("div", null, [
            _createVNode(MkPagination, {
              paginator: _unref(paginator),
              withControl: ""
            }, {
              default: _withCtx(({items}) => [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (list) => {
                  return (_openBlock(), _createBlock(_component_MkA, {
                    key: list.id,
                    class: _normalizeClass(["_panel", _ctx.$style.list]),
                    to: `/list/${ list.id }`
                  }, {
                    default: _withCtx(() => [
                      _createElementVNode("div", null, _toDisplayString(list.name), 1 /* TEXT */),
                      (list.userIds != null)
                        ? (_openBlock(), _createBlock(MkAvatars, {
                          key: 0,
                          userIds: list.userIds
                        }, null, 8 /* PROPS */, ["userIds"]))
                        : _createCommentVNode("v-if", true)
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["to"]))
                }), 128 /* KEYED_FRAGMENT */))
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["paginator"])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
