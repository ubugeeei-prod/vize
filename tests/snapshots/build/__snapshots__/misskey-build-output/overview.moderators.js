import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { onMounted, ref } from 'vue'
import * as Misskey from 'misskey-js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'overview.moderators',
  setup(__props) {

const moderators = ref<Misskey.entities.UserDetailed[] | null>(null);
const fetching = ref(true);
onMounted(async () => {
	moderators.value = await misskeyApi('admin/show-users', {
		sort: '+lastActiveDate',
		state: 'adminOrModerator',
		limit: 30,
	});
	fetching.value = false;
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(), _createElementBlock("div", null, [ _createVNode(_Transition, {
        name: _unref(prefer).s.animation ? '_transition_zoom' : '',
        mode: "out-in"
      }, {
        default: _withCtx(() => [
          (fetching.value)
            ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 }))
            : (_openBlock(), _createElementBlock("div", {
              key: 1,
              class: _normalizeClass(["_panel", _ctx.$style.root])
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(moderators.value, (user) => {
                return (_openBlock(), _createBlock(_component_MkA, {
                  key: user.id,
                  class: "user",
                  to: `/admin/user/${user.id}`
                }, {
                  default: _withCtx(() => [
                    _createVNode(_component_MkAvatar, {
                      user: user,
                      class: "avatar",
                      indicator: ""
                    }, null, 8 /* PROPS */, ["user"])
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
