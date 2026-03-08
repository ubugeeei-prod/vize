import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList } from "vue"

import { onMounted, ref } from 'vue'
import * as Misskey from 'misskey-js'
import { misskeyApi } from '@/utility/misskey-api.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkAvatars',
  props: {
    userIds: { type: Array, required: true },
    limit: { type: Number, required: false, default: Infinity }
  },
  setup(__props: any) {

const props = __props
const users = ref<Misskey.entities.UserLite[]>([]);
onMounted(async () => {
	users.value = await misskeyApi('users/show', {
		userIds: props.userIds,
	});
});

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")

  return (_openBlock(), _createElementBlock("div", null, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(users.value.slice(0, __props.limit), (user) => {
        return (_openBlock(), _createElementBlock("div", {
          key: user.id,
          style: "display:inline-block;width:32px;height:32px;margin-right:8px;"
        }, [
          _createVNode(_component_MkAvatar, {
            user: user,
            style: "width:32px; height:32px;",
            indicator: "",
            link: "",
            preview: ""
          }, null, 8 /* PROPS */, ["user"])
        ]))
      }), 128 /* KEYED_FRAGMENT */)), (users.value.length > __props.limit) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          style: "display: inline-block;"
        }, "...")) : _createCommentVNode("v-if", true) ]))
}
}

})
