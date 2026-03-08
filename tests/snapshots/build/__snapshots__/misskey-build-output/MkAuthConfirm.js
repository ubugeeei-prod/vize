import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vModelRadio as _vModelRadio } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-user" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-user-plus" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-apps" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
import { ref, computed } from 'vue'
import * as Misskey from 'misskey-js'
import MkButton from '@/components/MkButton.vue'
import { $i } from '@/i.js'
import { getAccounts, getAccountWithSigninDialog, getAccountWithSignupDialog } from '@/accounts.js'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'
import { getProxiedImageUrl } from '@/utility/media-proxy.js'
import { misskeyApi } from '@/utility/misskey-api.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkAuthConfirm',
  props: {
    name: { type: String, required: false },
    icon: { type: String, required: false },
    permissions: { type: Array, required: false },
    manualWaiting: { type: Boolean, required: false },
    waitOnDeny: { type: Boolean, required: false }
  },
  emits: ["accept", "deny"],
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const waiting = ref(true);
const _waiting = computed(() => waiting.value || props.manualWaiting);
const phase = ref<'accountSelect' | 'consent' | 'success' | 'denied' | 'failed'>('accountSelect');
const selectedUser = ref<string | null>(null);
const users = ref(new Map<string, Misskey.entities.UserDetailed & { token: string; }>());
async function init() {
	waiting.value = true;
	users.value.clear();
	if ($i) {
		users.value.set($i.id, $i);
	}
	const accounts = await getAccounts();
	const accountIdsToFetch = accounts.map(a => a.id).filter(id => !users.value.has(id));
	if (accountIdsToFetch.length > 0) {
		const usersRes = await misskeyApi('users/show', {
			userIds: accountIdsToFetch,
		});
		for (const user of usersRes) {
			if (users.value.has(user.id)) continue;
			const account = accounts.find(a => a.id === user.id);
			if (!account || account.token == null) continue;
			users.value.set(user.id, {
				...user,
				token: account.token,
			});
		}
	}
	waiting.value = false;
}
init();
function clickAddAccount(ev: PointerEvent) {
	selectedUser.value = null;
	os.popupMenu([{
		text: i18n.ts.existingAccount,
		action: () => {
			getAccountWithSigninDialog().then(async (res) => {
				if (res != null) {
					os.success();
					await init();
					if (users.value.has(res.id)) {
						selectedUser.value = res.id;
					}
				}
			});
		},
	}, {
		text: i18n.ts.createAccount,
		action: () => {
			getAccountWithSignupDialog().then(async (res) => {
				if (res != null) {
					os.success();
					await init();
					if (users.value.has(res.id)) {
						selectedUser.value = res.id;
					}
				}
			});
		},
	}], ev.currentTarget ?? ev.target);
}
function clickChooseAccount() {
	if (selectedUser.value === null) return;
	phase.value = 'consent';
}
function clickBackToAccountSelect() {
	selectedUser.value = null;
	phase.value = 'accountSelect';
}
function clickCancel() {
	if (selectedUser.value === null) return;
	const user = users.value.get(selectedUser.value)!;
	const token = user.token;
	if (props.waitOnDeny) {
		waiting.value = true;
	}
	emit('deny', token);
}
async function clickAccept() {
	if (selectedUser.value === null) return;
	const user = users.value.get(selectedUser.value)!;
	const token = user.token;
	waiting.value = true;
	emit('accept', token);
}
function showUI(state: 'success' | 'denied' | 'failed') {
	phase.value = state;
	waiting.value = false;
}
__expose({
	showUI,
})

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_MkAcct = _resolveComponent("MkAcct")
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.wrapper)
    }, [ _createVNode(_Transition, {
        mode: "out-in",
        enterActiveClass: _ctx.$style.transition_enterActive,
        leaveActiveClass: _ctx.$style.transition_leaveActive,
        enterFromClass: _ctx.$style.transition_enterFrom,
        leaveToClass: _ctx.$style.transition_leaveTo,
        inert: _waiting.value
      }, {
        default: _withCtx(() => [
          (phase.value === 'accountSelect')
            ? (_openBlock(), _createElementBlock("div", {
              key: "accountSelect",
              class: _normalizeClass(["_gaps", _ctx.$style.root])
            }, [
              _createElementVNode("div", {
                class: _normalizeClass(["_gaps_s", _ctx.$style.header])
              }, [
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.iconFallback)
                }, [
                  _hoisted_1
                ]),
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.headerText)
                }, _toDisplayString(_unref(i18n).ts.pleaseSelectAccount), 1 /* TEXT */)
              ]),
              _createElementVNode("div", null, [
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.accountSelectorLabel)
                }, _toDisplayString(_unref(i18n).ts.selectAccount), 1 /* TEXT */),
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.accountSelectorList)
                }, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(users.value, ([id, user]) => {
                    return (_openBlock(), _createElementBlock(_Fragment, null, [
                      _withDirectives(_createElementVNode("input", {
                        id: 'account-' + id,
                        "onUpdate:modelValue": [($event: any) => ((selectedUser).value = $event), ($event: any) => ((selectedUser).value = $event)],
                        type: "radio",
                        name: "accountSelector",
                        value: id,
                        class: _normalizeClass(_ctx.$style.accountSelectorRadio)
                      }, null, 8 /* PROPS */, ["id", "value"]), [
                        [_vModelRadio, selectedUser.value]
                      ]),
                      _createElementVNode("label", {
                        for: 'account-' + id,
                        class: _normalizeClass(_ctx.$style.accountSelectorItem)
                      }, [
                        _createVNode(_component_MkAvatar, {
                          user: user,
                          class: _normalizeClass(_ctx.$style.accountSelectorAvatar)
                        }, null, 8 /* PROPS */, ["user"]),
                        _createElementVNode("div", {
                          class: _normalizeClass(_ctx.$style.accountSelectorBody)
                        }, [
                          _createVNode(_component_MkUserName, {
                            user: user,
                            class: _normalizeClass(_ctx.$style.accountSelectorName)
                          }, null, 8 /* PROPS */, ["user"]),
                          _createVNode(_component_MkAcct, {
                            user: user,
                            class: _normalizeClass(_ctx.$style.accountSelectorAcct)
                          }, null, 8 /* PROPS */, ["user"])
                        ])
                      ], 8 /* PROPS */, ["for"])
                    ], 64 /* STABLE_FRAGMENT */))
                  }), 256 /* UNKEYED_FRAGMENT */)),
                  _createElementVNode("button", {
                    class: _normalizeClass(["_button", [_ctx.$style.accountSelectorItem, _ctx.$style.accountSelectorAddAccountRoot]]),
                    onClick: clickAddAccount
                  }, [
                    _createElementVNode("div", {
                      class: _normalizeClass([_ctx.$style.accountSelectorAvatar, _ctx.$style.accountSelectorAddAccountAvatar])
                    }, [
                      _hoisted_2
                    ]),
                    _createElementVNode("div", {
                      class: _normalizeClass([_ctx.$style.accountSelectorBody, _ctx.$style.accountSelectorName])
                    }, _toDisplayString(_unref(i18n).ts.addAccount), 1 /* TEXT */)
                  ])
                ])
              ]),
              _createElementVNode("div", { class: "_buttonsCenter" }, [
                _createVNode(MkButton, {
                  rounded: "",
                  gradate: "",
                  disabled: selectedUser.value === null,
                  onClick: clickChooseAccount
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.continue), 1 /* TEXT */),
                    _createTextVNode(" "),
                    _hoisted_3
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["disabled"])
              ])
            ]))
            : (phase.value === 'consent')
              ? (_openBlock(), _createElementBlock("div", {
                key: "consent",
                class: _normalizeClass(["_gaps", _ctx.$style.root])
              }, [
                _createElementVNode("div", {
                  class: _normalizeClass(["_gaps_s", _ctx.$style.header])
                }, [
                  (__props.icon)
                    ? (_openBlock(), _createElementBlock("img", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.icon),
                      src: _unref(getProxiedImageUrl)(__props.icon, 'preview')
                    }))
                    : (_openBlock(), _createElementBlock("div", {
                      key: 1,
                      class: _normalizeClass(_ctx.$style.iconFallback)
                    }, [
                      _hoisted_4
                    ])),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.headerText)
                  }, _toDisplayString(__props.name ? _unref(i18n).tsx._auth.shareAccess({ name: __props.name }) : _unref(i18n).ts._auth.shareAccessAsk), 1 /* TEXT */)
                ]),
                (__props.permissions && __props.permissions.length > 0)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: _normalizeClass(["_gaps_s", _ctx.$style.permissionRoot])
                  }, [
                    _createElementVNode("div", null, _toDisplayString(__props.name ? _unref(i18n).tsx._auth.permission({ name: __props.name }) : _unref(i18n).ts._auth.permissionAsk), 1 /* TEXT */),
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.permissionListWrapper)
                    }, [
                      _createElementVNode("ul", {
                        class: _normalizeClass(_ctx.$style.permissionList)
                      }, [
                        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.permissions, (p) => {
                          return (_openBlock(), _createElementBlock("li", { key: p }, _toDisplayString(_unref(i18n).ts._permissions[p]), 1 /* TEXT */))
                        }), 128 /* KEYED_FRAGMENT */))
                      ])
                    ])
                  ]))
                  : _createCommentVNode("v-if", true),
                _renderSlot(_ctx.$slots, "consentAdditionalInfo"),
                _createElementVNode("div", null, [
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.accountSelectorLabel)
                  }, [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._auth.scopeUser) + " ", 1 /* TEXT */),
                    _createElementVNode("button", {
                      class: "_textButton",
                      onClick: clickBackToAccountSelect
                    }, _toDisplayString(_unref(i18n).ts.switchAccount), 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.accountSelectorList)
                  }, [
                    _createElementVNode("div", {
                      class: _normalizeClass([_ctx.$style.accountSelectorItem, _ctx.$style.static])
                    }, [
                      _createVNode(_component_MkAvatar, {
                        user: users.value.get(selectedUser.value),
                        class: _normalizeClass(_ctx.$style.accountSelectorAvatar)
                      }, null, 8 /* PROPS */, ["user"]),
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.accountSelectorBody)
                      }, [
                        _createVNode(_component_MkUserName, {
                          user: users.value.get(selectedUser.value),
                          class: _normalizeClass(_ctx.$style.accountSelectorName)
                        }, null, 8 /* PROPS */, ["user"]),
                        _createVNode(_component_MkAcct, {
                          user: users.value.get(selectedUser.value),
                          class: _normalizeClass(_ctx.$style.accountSelectorAcct)
                        }, null, 8 /* PROPS */, ["user"])
                      ])
                    ])
                  ])
                ]),
                _createElementVNode("div", { class: "_buttonsCenter" }, [
                  _createVNode(MkButton, {
                    rounded: "",
                    onClick: clickCancel
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.reject), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }),
                  _createVNode(MkButton, {
                    rounded: "",
                    gradate: "",
                    onClick: clickAccept
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.accept), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ])
              ]))
            : (phase.value === 'success')
              ? (_openBlock(), _createElementBlock("div", {
                key: "success",
                class: _normalizeClass(["_gaps_s", _ctx.$style.root])
              }, [
                _createElementVNode("div", {
                  class: _normalizeClass(["_gaps_s", _ctx.$style.header])
                }, [
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.iconFallback)
                  }, [
                    _hoisted_5
                  ]),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.headerText)
                  }, _toDisplayString(_unref(i18n).ts._auth.accepted), 1 /* TEXT */),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.headerTextSub)
                  }, _toDisplayString(_unref(i18n).ts._auth.pleaseGoBack), 1 /* TEXT */)
                ])
              ]))
            : (phase.value === 'denied')
              ? (_openBlock(), _createElementBlock("div", {
                key: "denied",
                class: _normalizeClass(["_gaps_s", _ctx.$style.root])
              }, [
                _createElementVNode("div", {
                  class: _normalizeClass(["_gaps_s", _ctx.$style.header])
                }, [
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.iconFallback)
                  }, [
                    _hoisted_6
                  ]),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.headerText)
                  }, _toDisplayString(_unref(i18n).ts._auth.denied), 1 /* TEXT */)
                ])
              ]))
            : (phase.value === 'failed')
              ? (_openBlock(), _createElementBlock("div", {
                key: "failed",
                class: _normalizeClass(["_gaps_s", _ctx.$style.root])
              }, [
                _createElementVNode("div", {
                  class: _normalizeClass(["_gaps_s", _ctx.$style.header])
                }, [
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.iconFallback)
                  }, [
                    _hoisted_7
                  ]),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.headerText)
                  }, _toDisplayString(_unref(i18n).ts.somethingHappened), 1 /* TEXT */)
                ])
              ]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass", "inert"]), (_waiting.value) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: _normalizeClass(_ctx.$style.waitingRoot)
        }, [ _createVNode(_component_MkLoading) ])) : _createCommentVNode("v-if", true) ]))
}
}

})
