import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, renderList as _renderList, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { computed, markRaw } from 'vue'
import * as Misskey from 'misskey-js'
import MkPagination from '@/components/MkPagination.vue'
import MkNote from '@/components/MkNote.vue'
import MkReactionIcon from '@/components/MkReactionIcon.vue'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'reactions',
  props: {
    user: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const paginator = markRaw(new Paginator('users/reactions', {
	limit: 20,
	computedParams: computed(() => ({
		userId: props.user.id,
	})),
}));

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkTime = _resolveComponent("MkTime")

  return (_openBlock(), _createElementBlock("div", {
      class: "_spacer",
      style: "--MI_SPACER-w: 700px;"
    }, [ _createVNode(MkPagination, { paginator: _unref(paginator) }, {
        default: _withCtx(({items}) => [
          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
            return (_openBlock(), _createElementBlock("div", {
              key: item.id,
              to: `/clips/${item.id}`,
              class: "_panel _margin"
            }, [
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.header)
              }, [
                _createVNode(_component_MkAvatar, {
                  class: _normalizeClass(_ctx.$style.avatar),
                  user: __props.user
                }, null, 8 /* PROPS */, ["user"]),
                _createVNode(MkReactionIcon, {
                  class: _normalizeClass(_ctx.$style.reaction),
                  reaction: item.type,
                  noStyle: true
                }, null, 8 /* PROPS */, ["reaction", "noStyle"]),
                _createVNode(_component_MkTime, {
                  time: item.createdAt,
                  class: _normalizeClass(_ctx.$style.createdAt)
                }, null, 8 /* PROPS */, ["time"])
              ]),
              _createVNode(MkNote, {
                key: item.id,
                note: item.note
              }, null, 8 /* PROPS */, ["note"])
            ], 8 /* PROPS */, ["to"]))
          }), 128 /* KEYED_FRAGMENT */))
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["paginator"]) ]))
}
}

})
