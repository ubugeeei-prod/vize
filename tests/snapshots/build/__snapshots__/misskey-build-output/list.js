import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
import { computed, markRaw, ref, watch } from 'vue'
import * as Misskey from 'misskey-js'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { definePage } from '@/page.js'
import { i18n } from '@/i18n.js'
import { userPage } from '@/filters/user.js'
import MkUserCardMini from '@/components/MkUserCardMini.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkInput from '@/components/MkInput.vue'
import { userListsCache } from '@/cache.js'
import { ensureSignin } from '@/i.js'
import MkPagination from '@/components/MkPagination.vue'
import { useRouter } from '@/router.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'list',
  props: {
    listId: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const $i = ensureSignin();
const router = useRouter();
const list = ref<Misskey.entities.UserList | null>(null);
const isPublic = ref(false);
const name = ref('');
const membershipsPaginator = markRaw(new Paginator('users/lists/get-memberships', {
	limit: 30,
	computedParams: computed(() => ({
		listId: props.listId,
	})),
}));
function fetchList() {
	misskeyApi('users/lists/show', {
		listId: props.listId,
	}).then(_list => {
		list.value = _list;
		name.value = list.value.name;
		isPublic.value = list.value.isPublic;
	});
}
function addUser() {
	os.selectUser().then(user => {
		if (!list.value) return;
		os.apiWithDialog('users/lists/push', {
			listId: list.value.id,
			userId: user.id,
		}).then(() => {
			membershipsPaginator.reload();
		});
	});
}
async function removeUser(item: Misskey.entities.UsersListsGetMembershipsResponse[number], ev: PointerEvent) {
	os.popupMenu([{
		text: i18n.ts.remove,
		icon: 'ti ti-x',
		danger: true,
		action: async () => {
			if (!list.value) return;
			misskeyApi('users/lists/pull', {
				listId: list.value.id,
				userId: item.userId,
			}).then(() => {
				membershipsPaginator.removeItem(item.id);
			});
		},
	}], ev.currentTarget ?? ev.target);
}
async function showMembershipMenu(item: Misskey.entities.UsersListsGetMembershipsResponse[number], ev: PointerEvent) {
	const withRepliesRef = ref(item.withReplies);
	os.popupMenu([{
		type: 'switch',
		text: i18n.ts.showRepliesToOthersInTimeline,
		icon: 'ti ti-messages',
		ref: withRepliesRef,
	}], ev.currentTarget ?? ev.target);
	watch(withRepliesRef, withReplies => {
		misskeyApi('users/lists/update-membership', {
			listId: list.value!.id,
			userId: item.userId,
			withReplies,
		}).then(() => {
			membershipsPaginator.updateItem(item.id, (old) => ({
				...old,
				withReplies,
			}));
		});
	});
}
async function deleteList() {
	if (!list.value) return;
	const { canceled } = await os.confirm({
		type: 'warning',
		text: i18n.tsx.removeAreYouSure({ x: list.value.name }),
	});
	if (canceled) return;
	await os.apiWithDialog('users/lists/delete', {
		listId: list.value.id,
	});
	userListsCache.delete();
	router.push('/my/lists');
}
async function updateSettings() {
	if (!list.value) return;
	await os.apiWithDialog('users/lists/update', {
		listId: list.value.id,
		name: name.value,
		isPublic: isPublic.value,
	});
	userListsCache.delete();
	list.value.name = name.value;
	list.value.isPublic = isPublic.value;
}
watch(() => props.listId, fetchList, { immediate: true });
const headerActions = computed(() => list.value ? [{
	icon: 'ti ti-timeline',
	text: i18n.ts.timeline,
	handler: () => {
		router.push('/timeline/list/:listId', {
			params: {
				listId: list.value!.id,
			},
		});
	},
}] : []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: list.value ? list.value.name : i18n.ts.lists,
	icon: 'ti ti-list',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 700px;"
        }, [
          (list.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "_gaps"
            }, [
              _createVNode(MkFolder, null, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.settings), 1 /* TEXT */)
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkInput, {
                      modelValue: name.value,
                      "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((name).value = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.name), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["modelValue"]),
                    _createVNode(MkSwitch, {
                      modelValue: isPublic.value,
                      "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((isPublic).value = $event))
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.public), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["modelValue"]),
                    _createElementVNode("div", { class: "_buttons" }, [
                      _createVNode(MkButton, {
                        rounded: "",
                        primary: "",
                        onClick: updateSettings
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }),
                      _createVNode(MkButton, {
                        rounded: "",
                        danger: "",
                        onClick: _cache[2] || (_cache[2] = ($event: any) => (deleteList()))
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.delete), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      })
                    ])
                  ])
                ]),
                _: 1 /* STABLE */
              }),
              _createVNode(MkFolder, { defaultOpen: "" }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.members), 1 /* TEXT */)
                ]),
                caption: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).tsx.nUsers({ n: `${list.value.userIds.length}/${_unref($i).policies["userEachUserListsLimit"]}` })), 1 /* TEXT */)
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkButton, {
                      rounded: "",
                      primary: "",
                      style: "margin: 0 auto;",
                      onClick: _cache[3] || (_cache[3] = ($event: any) => (addUser()))
                    }, {
                      default: _withCtx(() => [
                        _hoisted_1,
                        _createTextVNode(" "),
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.addUser), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(MkPagination, { paginator: _unref(membershipsPaginator) }, {
                      default: _withCtx(({ items }) => [
                        _createElementVNode("div", { class: "_gaps_s" }, [
                          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
                            return (_openBlock(), _createElementBlock("div", { key: item.id }, [
                              _createElementVNode("div", {
                                class: _normalizeClass(_ctx.$style.userItem)
                              }, [
                                _createVNode(_component_MkA, {
                                  class: _normalizeClass(_ctx.$style.userItemBody),
                                  to: `${_unref(userPage)(item.user)}`
                                }, {
                                  default: _withCtx(() => [
                                    _createVNode(MkUserCardMini, { user: item.user }, null, 8 /* PROPS */, ["user"])
                                  ]),
                                  _: 2 /* DYNAMIC */
                                }, 8 /* PROPS */, ["to"]),
                                _createElementVNode("button", {
                                  class: _normalizeClass(["_button", _ctx.$style.menu]),
                                  onClick: _cache[4] || (_cache[4] = ($event: any) => (showMembershipMenu(item, $event)))
                                }, [
                                  _hoisted_2
                                ]),
                                _createElementVNode("button", {
                                  class: _normalizeClass(["_button", _ctx.$style.remove]),
                                  onClick: _cache[5] || (_cache[5] = ($event: any) => (removeUser(item, $event)))
                                }, [
                                  _hoisted_3
                                ])
                              ])
                            ]))
                          }), 128 /* KEYED_FRAGMENT */))
                        ])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["paginator"])
                  ])
                ]),
                _: 1 /* STABLE */
              })
            ]))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
