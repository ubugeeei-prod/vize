import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-info-circle" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-users" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
import { computed, markRaw, reactive, ref } from 'vue'
import * as Misskey from 'misskey-js'
import XEditor from './roles.editor.vue'
import MkFolder from '@/components/MkFolder.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import MkButton from '@/components/MkButton.vue'
import MkUserCardMini from '@/components/MkUserCardMini.vue'
import MkInfo from '@/components/MkInfo.vue'
import MkPagination from '@/components/MkPagination.vue'
import { useRouter } from '@/router.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'roles.role',
  props: {
    id: { type: String, required: true }
  },
  async setup(__props: any) {

let __temp: any, __restore: any

const props = __props
const router = useRouter();
const usersPaginator = markRaw(new Paginator('admin/roles/users', {
	limit: 20,
	computedParams: computed(() => props.id ? ({
		roleId: props.id,
	}) : undefined),
}));
const expandedItemIds = ref<Misskey.entities.AdminRolesUsersResponse[number]['id'][]>([]);
const role = reactive(await misskeyApi('admin/roles/show', {
	roleId: props.id,
}));
function edit() {
	router.push('/admin/roles/:id/edit', {
		params: {
			id: role.id,
		},
	});
}
async function del() {
	const { canceled } = await os.confirm({
		type: 'warning',
		text: i18n.tsx.deleteAreYouSure({ x: role.name }),
	});
	if (canceled) return;
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog('admin/roles/delete', {)),
  await __temp,
  __restore()
)
		roleId: role.id,
	});
	router.push('/admin/roles');
}
async function assign() {
	const user = await os.selectUser({ includeSelf: true });
	const { canceled: canceled2, result: period } = await os.select({
		title: i18n.ts.period + ': ' + role.name,
		items: [{
			value: 'indefinitely', label: i18n.ts.indefinitely,
		}, {
			value: 'oneHour', label: i18n.ts.oneHour,
		}, {
			value: 'oneDay', label: i18n.ts.oneDay,
		}, {
			value: 'oneWeek', label: i18n.ts.oneWeek,
		}, {
			value: 'oneMonth', label: i18n.ts.oneMonth,
		}],
		default: 'indefinitely',
	});
	if (canceled2) return;
	const expiresAt = period === 'indefinitely' ? null
		: period === 'oneHour' ? Date.now() + (1000 * 60 * 60)
		: period === 'oneDay' ? Date.now() + (1000 * 60 * 60 * 24)
		: period === 'oneWeek' ? Date.now() + (1000 * 60 * 60 * 24 * 7)
		: period === 'oneMonth' ? Date.now() + (1000 * 60 * 60 * 24 * 30)
		: null;
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog('admin/roles/assign', { roleId: role.id, userId: user.id, expiresAt }))),
  await __temp,
  __restore()
)
	//role.users.push(user);
}
async function unassign(userId: Misskey.entities.User['id'], ev: PointerEvent) {
	os.popupMenu([{
		text: i18n.ts.unassign,
		icon: 'ti ti-x',
		danger: true,
		action: async () => {
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog('admin/roles/unassign', { roleId: role.id, userId: userId }))),
  await __temp,
  __restore()
)
			//role.users = role.users.filter(u => u.id !== userId);
		},
	}], ev.currentTarget ?? ev.target);
}
async function toggleItem(itemId: string) {
	if (expandedItemIds.value.includes(itemId)) {
		expandedItemIds.value = expandedItemIds.value.filter(x => x !== itemId);
	} else {
		expandedItemIds.value.push(itemId);
	}
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: `${i18n.ts.role}: ${role.name}`,
	icon: 'ti ti-badge',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkResult = _resolveComponent("MkResult")
  const _component_MkA = _resolveComponent("MkA")
  const _component_MkTime = _resolveComponent("MkTime")
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
          _createElementVNode("div", { class: "_gaps" }, [
            _createElementVNode("div", { class: "_buttons" }, [
              _createVNode(MkButton, {
                primary: "",
                rounded: "",
                onClick: edit
              }, {
                default: _withCtx(() => [
                  _hoisted_1,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.edit), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }),
              _createVNode(MkButton, {
                danger: "",
                rounded: "",
                onClick: del
              }, {
                default: _withCtx(() => [
                  _hoisted_2,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.delete), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _createVNode(MkFolder, null, {
              icon: _withCtx(() => [
                _hoisted_3
              ]),
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.info), 1 /* TEXT */)
              ]),
              default: _withCtx(() => [
                _createVNode(XEditor, {
                  modelValue: role,
                  readonly: ""
                }, null, 8 /* PROPS */, ["modelValue"])
              ]),
              _: 1 /* STABLE */
            }),
            (role.target === 'manual')
              ? (_openBlock(), _createBlock(MkFolder, {
                key: 0,
                defaultOpen: ""
              }, {
                icon: _withCtx(() => [
                  _hoisted_4
                ]),
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.users), 1 /* TEXT */)
                ]),
                suffix: _withCtx(() => [
                  _createTextVNode(_toDisplayString(role.usersCount), 1 /* TEXT */)
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkButton, {
                      primary: "",
                      rounded: "",
                      onClick: assign
                    }, {
                      default: _withCtx(() => [
                        _hoisted_5,
                        _createTextVNode(" "),
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.assign), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(MkPagination, { paginator: _unref(usersPaginator) }, {
                      empty: _withCtx(() => [
                        _createVNode(_component_MkResult, {
                          type: "empty",
                          text: _unref(i18n).ts.noUsers
                        }, null, 8 /* PROPS */, ["text"])
                      ]),
                      default: _withCtx(({ items }) => [
                        _createElementVNode("div", { class: "_gaps_s" }, [
                          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
                            return (_openBlock(), _createElementBlock("div", {
                              key: item.user.id,
                              class: _normalizeClass([_ctx.$style.userItem, { [_ctx.$style.userItemOpened]: expandedItemIds.value.includes(item.id) }])
                            }, [
                              _createElementVNode("div", {
                                class: _normalizeClass(_ctx.$style.userItemMain)
                              }, [
                                _createVNode(_component_MkA, {
                                  class: _normalizeClass(_ctx.$style.userItemMainBody),
                                  to: `/admin/user/${item.user.id}`
                                }, {
                                  default: _withCtx(() => [
                                    _createVNode(MkUserCardMini, { user: item.user }, null, 8 /* PROPS */, ["user"])
                                  ]),
                                  _: 2 /* DYNAMIC */
                                }, 8 /* PROPS */, ["to"]),
                                _createElementVNode("button", {
                                  class: _normalizeClass(["_button", _ctx.$style.userToggle]),
                                  onClick: _cache[0] || (_cache[0] = ($event: any) => (toggleItem(item.id)))
                                }, [
                                  _createElementVNode("i", {
                                    class: _normalizeClass(["ti ti-chevron-down", _ctx.$style.chevron])
                                  })
                                ]),
                                _createElementVNode("button", {
                                  class: _normalizeClass(["_button", _ctx.$style.unassign]),
                                  onClick: _cache[1] || (_cache[1] = ($event: any) => (unassign(item.user.id, $event)))
                                }, [
                                  _hoisted_6
                                ])
                              ]),
                              (expandedItemIds.value.includes(item.id))
                                ? (_openBlock(), _createElementBlock("div", {
                                  key: 0,
                                  class: _normalizeClass(_ctx.$style.userItemSub)
                                }, [
                                  _createElementVNode("div", null, [
                                    _createTextVNode("Assigned: "),
                                    _createVNode(_component_MkTime, {
                                      time: item.createdAt,
                                      mode: "detail"
                                    }, null, 8 /* PROPS */, ["time"])
                                  ]),
                                  (item.expiresAt)
                                    ? (_openBlock(), _createElementBlock("div", { key: 0 }, "Period: " + _toDisplayString(new Date(item.expiresAt).toLocaleString()), 1 /* TEXT */))
                                    : (_openBlock(), _createElementBlock("div", { key: 1 }, "Period: " + _toDisplayString(_unref(i18n).ts.indefinitely), 1 /* TEXT */))
                                ]))
                                : _createCommentVNode("v-if", true)
                            ], 2 /* CLASS */))
                          }), 128 /* KEYED_FRAGMENT */))
                        ])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["paginator"])
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : (_openBlock(), _createBlock(MkInfo, { key: 1 }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._role.isConditionalRole), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }))
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
