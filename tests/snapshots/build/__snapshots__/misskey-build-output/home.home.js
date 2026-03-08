import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-search" })
import { onActivated, onDeactivated, onMounted, ref } from 'vue'
import * as Misskey from 'misskey-js'
import { useInterval } from '@@/js/use-interval.js'
import XMessage from './XMessage.vue'
import MkButton from '@/components/MkButton.vue'
import { i18n } from '@/i18n.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { ensureSignin } from '@/i.js'
import { useRouter } from '@/router.js'
import * as os from '@/os.js'
import { updateCurrentAccountPartial } from '@/accounts.js'
import MkInput from '@/components/MkInput.vue'
import MkFoldableSection from '@/components/MkFoldableSection.vue'
import MkInfo from '@/components/MkInfo.vue'
import MkChatHistories from '@/components/MkChatHistories.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'home.home',
  setup(__props) {

const $i = ensureSignin();
const router = useRouter();
const searchQuery = ref('');
const searched = ref(false);
const searchResults = ref<Misskey.entities.ChatMessage[]>([]);
function start(ev: PointerEvent) {
	os.popupMenu([{
		text: i18n.ts._chat.individualChat,
		caption: i18n.ts._chat.individualChat_description,
		icon: 'ti ti-user',
		action: () => { startUser(); },
	}, { type: 'divider' }, {
		type: 'parent',
		text: i18n.ts._chat.roomChat,
		caption: i18n.ts._chat.roomChat_description,
		icon: 'ti ti-users-group',
		children: [{
			text: i18n.ts._chat.createRoom,
			icon: 'ti ti-plus',
			action: () => { createRoom(); },
		}],
	}], ev.currentTarget ?? ev.target);
}
async function startUser() {
	// TODO: localOnly は連合に対応したら消す
	os.selectUser({ localOnly: true }).then(user => {
		router.push('/chat/user/:userId', {
			params: {
				userId: user.id,
			},
		});
	});
}
async function createRoom() {
	const { canceled, result } = await os.inputText({
		title: i18n.ts.name,
		minLength: 1,
	});
	if (canceled) return;
	const room = await misskeyApi('chat/rooms/create', {
		name: result,
	});
	router.push('/chat/room/:roomId', {
		params: {
			roomId: room.id,
		},
	});
}
async function search() {
	const res = await misskeyApi('chat/messages/search', {
		query: searchQuery.value,
	});
	searchResults.value = res;
	searched.value = true;
}
onMounted(() => {
	updateCurrentAccountPartial({ hasUnreadChatMessages: false });
});

return (_ctx: any,_cache: any) => {
  const _component_MkAd = _resolveComponent("MkAd")

  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ (_unref($i).policies.chatAvailability === 'available') ? (_openBlock(), _createBlock(MkButton, {
          key: 0,
          primary: "",
          gradate: "",
          rounded: "",
          class: _normalizeClass(_ctx.$style.start),
          onClick: start
        }, {
          default: _withCtx(() => [
            _hoisted_1,
            _createTextVNode(" "),
            _createTextVNode(_toDisplayString(_unref(i18n).ts.startChat), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })) : (_openBlock(), _createBlock(MkInfo, { key: 1 }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref($i).policies.chatAvailability === 'readonly' ? _unref(i18n).ts._chat.chatIsReadOnlyForThisAccountOrServer : _unref(i18n).ts._chat.chatNotAvailableForThisAccountOrServer), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })), _createVNode(_component_MkAd, { preferForms: ['horizontal', 'horizontal-big'] }, null, 8 /* PROPS */, ["preferForms"]), _createVNode(MkInput, {
        placeholder: _unref(i18n).ts._chat.searchMessages,
        type: "search",
        modelValue: searchQuery.value,
        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((searchQuery).value = $event))
      }, {
        prefix: _withCtx(() => [
          _hoisted_2
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["placeholder", "modelValue"]), (searchQuery.value.length > 0) ? (_openBlock(), _createBlock(MkButton, {
          key: 0,
          primary: "",
          rounded: "",
          onClick: search
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.search), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })) : _createCommentVNode("v-if", true), (searched.value) ? (_openBlock(), _createBlock(MkFoldableSection, { key: 0 }, {
          header: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.searchResult), 1 /* TEXT */)
          ]),
          default: _withCtx(() => [
            _createElementVNode("div", { class: "_gaps_s" }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(searchResults.value, (message) => {
                return (_openBlock(), _createElementBlock("div", {
                  key: message.id,
                  class: _normalizeClass(_ctx.$style.searchResultItem)
                }, [
                  _createVNode(XMessage, {
                    message: message,
                    isSearchResult: true
                  }, null, 8 /* PROPS */, ["message", "isSearchResult"])
                ]))
              }), 128 /* KEYED_FRAGMENT */))
            ])
          ]),
          _: 1 /* STABLE */
        })) : _createCommentVNode("v-if", true), _createVNode(MkFoldableSection, null, {
        header: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._chat.history), 1 /* TEXT */)
        ]),
        default: _withCtx(() => [
          _createVNode(MkChatHistories)
        ]),
        _: 1 /* STABLE */
      }) ]))
}
}

})
