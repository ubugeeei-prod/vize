import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, unref as _unref } from "vue"

import { onMounted, ref } from 'vue'
import * as Misskey from 'misskey-js'
import XRoom from './XRoom.vue'
import { i18n } from '@/i18n.js'
import { misskeyApi } from '@/utility/misskey-api.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'home.ownedRooms',
  setup(__props) {

const fetching = ref(true);
const rooms = ref<Misskey.entities.ChatRoom[]>([]);
async function fetchRooms() {
	fetching.value = true;
	const res = await misskeyApi('chat/rooms/owned', {
	});
	rooms.value = res;
	fetching.value = false;
}
onMounted(() => {
	fetchRooms();
});

return (_ctx: any,_cache: any) => {
  const _component_MkResult = _resolveComponent("MkResult")
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ (rooms.value.length > 0) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "_gaps_s"
        }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(rooms.value, (room) => {
            return (_openBlock(), _createBlock(XRoom, {
              key: room.id,
              room: room
            }, null, 8 /* PROPS */, ["room"]))
          }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), (!fetching.value && rooms.value.length == 0) ? (_openBlock(), _createBlock(_component_MkResult, {
          key: 0,
          type: "empty",
          text: _unref(i18n).ts._chat.noRooms
        }, null, 8 /* PROPS */, ["text"])) : _createCommentVNode("v-if", true), (fetching.value) ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 })) : _createCommentVNode("v-if", true) ]))
}
}

})
