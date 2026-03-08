import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-users" })
import { onActivated, onDeactivated, onMounted, ref } from 'vue'
import * as Misskey from 'misskey-js'
import { useInterval } from '@@/js/use-interval.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { ensureSignin } from '@/i.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkChatHistories',
  setup(__props) {

const $i = ensureSignin();
const history = ref<{
	id: string;
	message: Misskey.entities.ChatMessage;
	other: Misskey.entities.ChatMessage['fromUser'] | Misskey.entities.ChatMessage['toUser'] | null;
	isMe: boolean;
}[]>([]);
const initializing = ref(true);
const fetching = ref(false);
async function fetchHistory() {
	if (fetching.value) return;
	fetching.value = true;
	const [userMessages, roomMessages] = await Promise.all([
		misskeyApi('chat/history', { room: false }),
		misskeyApi('chat/history', { room: true }),
	]);
	history.value = [...userMessages, ...roomMessages]
		.toSorted((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime())
		.map(m => ({
			id: m.id,
			message: m,
			other: (!('room' in m) || m.room == null) ? (m.fromUserId === $i.id ? m.toUser : m.fromUser) : null,
			isMe: m.fromUserId === $i.id,
		}));
	fetching.value = false;
	initializing.value = false;
}
let isActivated = true;
onActivated(() => {
	isActivated = true;
});
onDeactivated(() => {
	isActivated = false;
});
useInterval(() => {
	// TODO: DOM的にバックグラウンドになっていないかどうかも考慮する
	if (!window.document.hidden && isActivated) {
		fetchHistory();
	}
}, 1000 * 10, {
	immediate: false,
	afterMounted: true,
});
onActivated(() => {
	fetchHistory();
});
onMounted(() => {
	fetchHistory();
});

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_MkAcct = _resolveComponent("MkAcct")
  const _component_MkA = _resolveComponent("MkA")
  const _component_MkResult = _resolveComponent("MkResult")
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createElementBlock(_Fragment, null, [ (history.value.length > 0) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "_gaps_s"
        }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(history.value, (item) => {
            return (_openBlock(), _createBlock(_component_MkA, {
              key: item.id,
              class: _normalizeClass(["_panel", [_ctx.$style.message, { [_ctx.$style.isMe]: item.isMe, [_ctx.$style.isRead]: item.message.isRead }]]),
              to: item.message.toRoomId ? `/chat/room/${item.message.toRoomId}` : `/chat/user/${item.other.id}`
            }, {
              default: _withCtx(() => [
                (item.message.toRoomId)
                  ? (_openBlock(), _createBlock(_component_MkAvatar, {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.messageAvatar),
                    user: item.message.fromUser,
                    indicator: "",
                    preview: false
                  }, null, 8 /* PROPS */, ["user", "preview"]))
                  : (item.other)
                    ? (_openBlock(), _createBlock(_component_MkAvatar, {
                      key: 1,
                      class: _normalizeClass(_ctx.$style.messageAvatar),
                      user: item.other,
                      indicator: "",
                      preview: false
                    }, null, 8 /* PROPS */, ["user", "preview"]))
                  : _createCommentVNode("v-if", true),
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.messageBody)
                }, [
                  (item.message.toRoom)
                    ? (_openBlock(), _createElementBlock("header", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.messageHeader)
                    }, [
                      _createElementVNode("span", {
                        class: _normalizeClass(_ctx.$style.messageHeaderName)
                      }, [
                        _hoisted_1,
                        _createTextVNode(" " + _toDisplayString(item.message.toRoom.name), 1 /* TEXT */)
                      ]),
                      _createVNode(_component_MkTime, {
                        time: item.message.createdAt,
                        class: _normalizeClass(_ctx.$style.messageHeaderTime)
                      }, null, 8 /* PROPS */, ["time"])
                    ]))
                    : (_openBlock(), _createElementBlock("header", {
                      key: 1,
                      class: _normalizeClass(_ctx.$style.messageHeader)
                    }, [
                      _createVNode(_component_MkUserName, {
                        class: _normalizeClass(_ctx.$style.messageHeaderName),
                        user: item.other
                      }, null, 8 /* PROPS */, ["user"]),
                      _createVNode(_component_MkAcct, {
                        class: _normalizeClass(_ctx.$style.messageHeaderUsername),
                        user: item.other
                      }, null, 8 /* PROPS */, ["user"]),
                      _createVNode(_component_MkTime, {
                        time: item.message.createdAt,
                        class: _normalizeClass(_ctx.$style.messageHeaderTime)
                      }, null, 8 /* PROPS */, ["time"])
                    ])),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.messageBodyText)
                  }, [
                    (item.isMe)
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.youSaid)
                      }, _toDisplayString(_unref(i18n).ts.you) + ":", 1 /* TEXT */))
                      : _createCommentVNode("v-if", true),
                    _createTextVNode(_toDisplayString(item.message.text), 1 /* TEXT */)
                  ])
                ])
              ]),
              _: 2 /* DYNAMIC */
            }, 1034 /* CLASS, PROPS, DYNAMIC_SLOTS */, ["to"]))
          }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), (!initializing.value && history.value.length == 0) ? (_openBlock(), _createBlock(_component_MkResult, {
          key: 0,
          type: "empty",
          text: _unref(i18n).ts._chat.noHistory
        }, null, 8 /* PROPS */, ["text"])) : _createCommentVNode("v-if", true), (initializing.value) ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 })) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */))
}
}

})
