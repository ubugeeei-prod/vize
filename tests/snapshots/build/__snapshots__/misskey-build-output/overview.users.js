import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import { useInterval } from '@@/js/use-interval.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import MkUserCardMini from '@/components/MkUserCardMini.vue'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'overview.users',
  setup(__props) {

const newUsers = ref<Misskey.entities.UserDetailed[] | null>(null);
const fetching = ref(true);
const fetch = async () => {
	const _newUsers = await misskeyApi('admin/show-users', {
		limit: 5,
		sort: '+createdAt',
		origin: 'local',
	});
	newUsers.value = _newUsers;
	fetching.value = false;
};
useInterval(fetch, 1000 * 60, {
	immediate: true,
	afterMounted: true,
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ _createVNode(_Transition, {
        name: _unref(prefer).s.animation ? '_transition_zoom' : '',
        mode: "out-in"
      }, {
        default: _withCtx(() => [
          (fetching.value)
            ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 }))
            : (_openBlock(), _createElementBlock("div", {
              key: 1,
              class: "users"
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(newUsers.value, (user, i) => {
                return (_openBlock(), _createBlock(_component_MkA, {
                  key: user.id,
                  to: `/admin/user/${user.id}`,
                  class: "user"
                }, {
                  default: _withCtx(() => [
                    _createVNode(MkUserCardMini, { user: user }, null, 8 /* PROPS */, ["user"])
                  ]),
                  _: 2 /* DYNAMIC */
                }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["to"]))
              }), 128 /* KEYED_FRAGMENT */))
            ]))
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["name"]) ]))
}
}

})
