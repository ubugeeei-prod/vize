import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("hr")
import { computed, onMounted, ref } from 'vue'
import * as Misskey from 'misskey-js'
import MkButton from '@/components/MkButton.vue'
import { i18n } from '@/i18n.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import MkUserCardMini from '@/components/MkUserCardMini.vue'
import { userPage } from '@/filters/user.js'
import { ensureSignin } from '@/i.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'room.members',
  props: {
    room: { type: null, required: true }
  },
  emits: ["inviteUser"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const $i = ensureSignin();
const isOwner = computed(() => {
	return props.room.ownerId === $i.id;
});
const memberships = ref<Misskey.entities.ChatRoomMembership[]>([]);
const invitations = ref<Misskey.entities.ChatRoomInvitation[]>([]);
onMounted(async () => {
	memberships.value = await misskeyApi('chat/rooms/members', {
		roomId: props.room.id,
		limit: 50,
	});
	if (isOwner.value) {
		invitations.value = await misskeyApi('chat/rooms/invitations/outbox', {
			roomId: props.room.id,
			limit: 50,
		});
	}
});

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ (isOwner.value) ? (_openBlock(), _createBlock(MkButton, {
          key: 0,
          primary: "",
          rounded: "",
          style: "margin: 0 auto;",
          onClick: _cache[0] || (_cache[0] = ($event: any) => (emit('inviteUser')))
        }, {
          default: _withCtx(() => [
            _hoisted_1,
            _createTextVNode(" "),
            _createTextVNode(_toDisplayString(_unref(i18n).ts._chat.inviteUser), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })) : _createCommentVNode("v-if", true), _createVNode(_component_MkA, {
        class: _normalizeClass(_ctx.$style.membershipBody),
        to: `${_unref(userPage)(__props.room.owner)}`
      }, {
        default: _withCtx(() => [
          _createVNode(MkUserCardMini, { user: __props.room.owner }, null, 8 /* PROPS */, ["user"])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["to"]), (memberships.value.length > 0) ? (_openBlock(), _createElementBlock("hr", { key: 0 })) : _createCommentVNode("v-if", true), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(memberships.value, (membership) => {
        return (_openBlock(), _createElementBlock("div", {
          key: membership.id,
          class: _normalizeClass(_ctx.$style.membership)
        }, [
          _createVNode(_component_MkA, {
            class: _normalizeClass(_ctx.$style.membershipBody),
            to: `${_unref(userPage)(membership.user)}`
          }, {
            default: _withCtx(() => [
              _createVNode(MkUserCardMini, { user: membership.user }, null, 8 /* PROPS */, ["user"])
            ]),
            _: 2 /* DYNAMIC */
          }, 8 /* PROPS */, ["to"])
        ]))
      }), 128 /* KEYED_FRAGMENT */)), (isOwner.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _hoisted_2, _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._chat.sentInvitations), 1 /* TEXT */), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(invitations.value, (invitation) => {
            return (_openBlock(), _createElementBlock("div", {
              key: invitation.id,
              class: _normalizeClass(_ctx.$style.invitation)
            }, [
              _createVNode(_component_MkA, {
                class: _normalizeClass(_ctx.$style.invitationBody),
                to: `${_unref(userPage)(invitation.user)}`
              }, {
                default: _withCtx(() => [
                  _createVNode(MkUserCardMini, { user: invitation.user }, null, 8 /* PROPS */, ["user"])
                ]),
                _: 2 /* DYNAMIC */
              }, 8 /* PROPS */, ["to"])
            ]))
          }), 128 /* KEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ]))
}
}

})
