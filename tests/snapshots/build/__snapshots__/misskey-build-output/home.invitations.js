import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-users-group" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("hr")
import { onMounted, ref } from 'vue'
import * as Misskey from 'misskey-js'
import MkButton from '@/components/MkButton.vue'
import { i18n } from '@/i18n.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { useRouter } from '@/router.js'
import MkFolder from '@/components/MkFolder.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'home.invitations',
  setup(__props) {

const router = useRouter();
const fetching = ref(true);
const invitations = ref<Misskey.entities.ChatRoomInvitation[]>([]);
async function fetchInvitations() {
	fetching.value = true;
	const res = await misskeyApi('chat/rooms/invitations/inbox');
	invitations.value = res;
	fetching.value = false;
}
async function join(invitation: Misskey.entities.ChatRoomInvitation) {
	await misskeyApi('chat/rooms/join', {
		roomId: invitation.room.id,
	});
	router.push('/chat/room/:roomId', {
		params: {
			roomId: invitation.room.id,
		},
	});
}
async function ignore(invitation: Misskey.entities.ChatRoomInvitation) {
	await misskeyApi('chat/rooms/invitations/ignore', {
		roomId: invitation.room.id,
	});
	invitations.value = invitations.value.filter(i => i.id !== invitation.id);
}
onMounted(() => {
	fetchInvitations();
});

return (_ctx: any,_cache: any) => {
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_MkResult = _resolveComponent("MkResult")
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ (invitations.value.length > 0) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "_gaps_s"
        }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(invitations.value, (invitation) => {
            return (_openBlock(), _createBlock(MkFolder, {
              key: invitation.id,
              defaultOpen: true
            }, {
              icon: _withCtx(() => [
                _hoisted_1
              ]),
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(invitation.room.name), 1 /* TEXT */)
              ]),
              suffix: _withCtx(() => [
                _createVNode(_component_MkTime, { time: invitation.createdAt }, null, 8 /* PROPS */, ["time"])
              ]),
              footer: _withCtx(() => [
                _createElementVNode("div", { class: "_buttons" }, [
                  _createVNode(MkButton, {
                    primary: "",
                    onClick: _cache[0] || (_cache[0] = ($event: any) => (join(invitation)))
                  }, {
                    default: _withCtx(() => [
                      _hoisted_2,
                      _createTextVNode(" "),
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._chat.join), 1 /* TEXT */)
                    ]),
                    _: 2 /* DYNAMIC */
                  }),
                  _createVNode(MkButton, {
                    danger: "",
                    onClick: _cache[1] || (_cache[1] = ($event: any) => (ignore(invitation)))
                  }, {
                    default: _withCtx(() => [
                      _hoisted_3,
                      _createTextVNode(" "),
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._chat.ignore), 1 /* TEXT */)
                    ]),
                    _: 2 /* DYNAMIC */
                  })
                ])
              ]),
              default: _withCtx(() => [
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.invitationBody)
                }, [
                  _createVNode(_component_MkAvatar, {
                    user: invitation.room.owner,
                    class: _normalizeClass(_ctx.$style.invitationBodyAvatar),
                    link: ""
                  }, null, 8 /* PROPS */, ["user"]),
                  _createElementVNode("div", {
                    style: "flex: 1;",
                    class: "_gaps_s"
                  }, [
                    _createVNode(_component_MkUserName, { user: invitation.room.owner }, null, 8 /* PROPS */, ["user"]),
                    _hoisted_4,
                    _createElementVNode("div", null, _toDisplayString(invitation.room.description === '' ? _unref(i18n).ts.noDescription : invitation.room.description), 1 /* TEXT */)
                  ])
                ])
              ]),
              _: 2 /* DYNAMIC */
            }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["defaultOpen"]))
          }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), (!fetching.value && invitations.value.length == 0) ? (_openBlock(), _createBlock(_component_MkResult, {
          key: 0,
          type: "empty",
          text: _unref(i18n).ts._chat.noInvitations
        }, null, 8 /* PROPS */, ["text"])) : _createCommentVNode("v-if", true), (fetching.value) ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 })) : _createCommentVNode("v-if", true) ]))
}
}

})
