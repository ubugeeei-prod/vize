import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { onMounted, ref, computed, useTemplateRef } from 'vue'
import * as Misskey from 'misskey-js'
import { host as currentHost, hostname } from '@@/js/config.js'
import MkInput from '@/components/MkInput.vue'
import FormSplit from '@/components/form/split.vue'
import MkModalWindow from '@/components/MkModalWindow.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { store } from '@/store.js'
import { i18n } from '@/i18n.js'
import { $i } from '@/i.js'
import { instance } from '@/instance.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkUserSelectDialog',
  props: {
    includeSelf: { type: Boolean, required: false, default: false },
    localOnly: { type: Boolean, required: false, default: false }
  },
  emits: ["ok", "cancel", "closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const computedLocalOnly = computed(() => props.localOnly || instance.federation === 'none');
const username = ref('');
const host = ref('');
const users = ref<Misskey.entities.UserLite[]>([]);
const recentUsers = ref<Misskey.entities.UserDetailed[]>([]);
const selected = ref<Misskey.entities.UserLite | null>(null);
const dialogEl = useTemplateRef('dialogEl');
function search() {
	if (username.value === '' && host.value === '') {
		users.value = [];
		return;
	}
	misskeyApi('users/search-by-username-and-host', {
		username: username.value,
		host: computedLocalOnly.value ? '.' : host.value,
		limit: 10,
		detail: false,
	}).then(_users => {
		users.value = _users.filter((u) => {
			if (props.includeSelf) {
				return true;
			} else {
				return u.id !== $i?.id;
			}
		});
	});
}
async function ok() {
	if (selected.value == null) return;
	const user = await misskeyApi('users/show', {
		userId: selected.value.id,
	});
	emit('ok', user);
	dialogEl.value?.close();
	// 最近使ったユーザー更新
	let recents = store.s.recentlyUsedUsers;
	recents = recents.filter(x => x !== selected.value?.id);
	recents.unshift(selected.value.id);
	store.set('recentlyUsedUsers', recents.splice(0, 16));
}
function cancel() {
	emit('cancel');
	dialogEl.value?.close();
}
onMounted(() => {
	misskeyApi('users/show', {
		userIds: store.s.recentlyUsedUsers,
	}).then(foundUsers => {
		let _users = foundUsers;
		_users = _users.filter((u) => {
			if (computedLocalOnly.value) {
				return u.host == null;
			} else {
				return true;
			}
		});
		_users = _users.filter((u) => {
			if (props.includeSelf) {
				return true;
			} else {
				return u.id !== $i?.id;
			}
		});
		recentUsers.value = _users;
	});
});

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_MkAcct = _resolveComponent("MkAcct")

  return (_openBlock(), _createBlock(MkModalWindow, {
      ref_key: "dialogEl", ref: dialogEl,
      withOkButton: true,
      okButtonDisabled: selected.value == null,
      onClick: _cache[0] || (_cache[0] = ($event: any) => (cancel())),
      onClose: _cache[1] || (_cache[1] = ($event: any) => (cancel())),
      onOk: _cache[2] || (_cache[2] = ($event: any) => (ok())),
      onClosed: _cache[3] || (_cache[3] = ($event: any) => (emit('closed')))
    }, {
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(i18n).ts.selectUser), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", null, [
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.form)
          }, [
            (computedLocalOnly.value)
              ? (_openBlock(), _createBlock(MkInput, {
                key: 0,
                autofocus: true,
                "onUpdate:modelValue": search,
                modelValue: username.value
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.username), 1 /* TEXT */)
                ]),
                prefix: _withCtx(() => [
                  _createTextVNode("@")
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["autofocus", "modelValue"]))
              : (_openBlock(), _createBlock(FormSplit, {
                key: 1,
                minWidth: 170
              }, {
                default: _withCtx(() => [
                  _createVNode(MkInput, {
                    autofocus: true,
                    "onUpdate:modelValue": [search, ($event: any) => ((username).value = $event)],
                    modelValue: username.value
                  }, {
                    label: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.username), 1 /* TEXT */)
                    ]),
                    prefix: _withCtx(() => [
                      _createTextVNode("@")
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["autofocus", "modelValue"]),
                  _createVNode(MkInput, {
                    datalist: [_unref(hostname)],
                    "onUpdate:modelValue": [search, ($event: any) => ((host).value = $event)],
                    modelValue: host.value
                  }, {
                    label: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.host), 1 /* TEXT */)
                    ]),
                    prefix: _withCtx(() => [
                      _createTextVNode("@")
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["datalist", "modelValue"])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["minWidth"]))
          ]),
          (username.value != '' || host.value != '')
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass([_ctx.$style.result, { [_ctx.$style.hit]: users.value.length > 0 }])
            }, [
              (users.value.length > 0)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.users)
                }, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(users.value, (user) => {
                    return (_openBlock(), _createElementBlock("div", {
                      key: user.id,
                      class: _normalizeClass(["_button", [_ctx.$style.user, { [_ctx.$style.selected]: selected.value && selected.value.id === user.id }]]),
                      onClick: _cache[4] || (_cache[4] = ($event: any) => (selected.value = user)),
                      onDblclick: _cache[5] || (_cache[5] = ($event: any) => (ok()))
                    }, [
                      _createVNode(_component_MkAvatar, {
                        user: user,
                        class: _normalizeClass(_ctx.$style.avatar),
                        indicator: ""
                      }, null, 8 /* PROPS */, ["user"]),
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.userBody)
                      }, [
                        _createVNode(_component_MkUserName, {
                          user: user,
                          class: _normalizeClass(_ctx.$style.userName)
                        }, null, 8 /* PROPS */, ["user"]),
                        _createVNode(_component_MkAcct, {
                          user: user,
                          class: _normalizeClass(_ctx.$style.userAcct)
                        }, null, 8 /* PROPS */, ["user"])
                      ])
                    ], 2 /* CLASS */))
                  }), 128 /* KEYED_FRAGMENT */))
                ]))
                : (_openBlock(), _createElementBlock("div", {
                  key: 1,
                  class: _normalizeClass(_ctx.$style.empty)
                }, [
                  _createElementVNode("span", null, _toDisplayString(_unref(i18n).ts.noUsers), 1 /* TEXT */)
                ]))
            ]))
            : _createCommentVNode("v-if", true),
          (username.value == '' && host.value == '')
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(_ctx.$style.recent)
            }, [
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.users)
              }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(recentUsers.value, (user) => {
                  return (_openBlock(), _createElementBlock("div", {
                    key: user.id,
                    class: _normalizeClass(["_button", [_ctx.$style.user, { [_ctx.$style.selected]: selected.value && selected.value.id === user.id }]]),
                    onClick: _cache[6] || (_cache[6] = ($event: any) => (selected.value = user)),
                    onDblclick: _cache[7] || (_cache[7] = ($event: any) => (ok()))
                  }, [
                    _createVNode(_component_MkAvatar, {
                      user: user,
                      class: _normalizeClass(_ctx.$style.avatar),
                      indicator: ""
                    }, null, 8 /* PROPS */, ["user"]),
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.userBody)
                    }, [
                      _createVNode(_component_MkUserName, {
                        user: user,
                        class: _normalizeClass(_ctx.$style.userName)
                      }, null, 8 /* PROPS */, ["user"]),
                      _createVNode(_component_MkAcct, {
                        user: user,
                        class: _normalizeClass(_ctx.$style.userAcct)
                      }, null, 8 /* PROPS */, ["user"])
                    ])
                  ], 2 /* CLASS */))
                }), 128 /* KEYED_FRAGMENT */))
              ])
            ]))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["withOkButton", "okButtonDisabled"]))
}
}

})
