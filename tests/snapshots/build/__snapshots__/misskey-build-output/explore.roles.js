import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, unref as _unref } from "vue"

import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import MkRolePreview from '@/components/MkRolePreview.vue'
import { i18n } from '@/i18n.js'
import { misskeyApi } from '@/utility/misskey-api.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'explore.roles',
  setup(__props) {

const roles = ref<Misskey.entities.Role[] | null>(null);
const loading = ref(true);
misskeyApi('roles/list').then(res => {
	roles.value = res.filter(x => x.target === 'manual').sort((a, b) => b.displayOrder - a.displayOrder);
}).finally(() => {
	loading.value = false;
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_MkResult = _resolveComponent("MkResult")

  return (_openBlock(), _createElementBlock("div", {
      class: "_spacer",
      style: "--MI_SPACER-w: 700px;"
    }, [ (roles.value != null && roles.value.length > 0) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "_gaps_s"
        }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(roles.value, (role) => {
            return (_openBlock(), _createBlock(MkRolePreview, {
              key: role.id,
              role: role,
              forModeration: false
            }, null, 8 /* PROPS */, ["role", "forModeration"]))
          }), 128 /* KEYED_FRAGMENT */)) ])) : (loading.value) ? (_openBlock(), _createBlock(_component_MkLoading, { key: 1 })) : (_openBlock(), _createBlock(_component_MkResult, {
          key: 2,
          type: "empty",
          text: _unref(i18n).ts.noRole
        }, null, 8 /* PROPS */, ["text"])) ]))
}
}

})
