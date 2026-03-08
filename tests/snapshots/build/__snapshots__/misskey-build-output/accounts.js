import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
import { ref, computed } from 'vue'
import * as Misskey from 'misskey-js'
import type { MenuItem } from '@/types/menu.js'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { $i } from '@/i.js'
import { switchAccount, removeAccount, login, getAccountWithSigninDialog, getAccountWithSignupDialog, getAccounts } from '@/accounts.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import MkUserCardMini from '@/components/MkUserCardMini.vue'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'accounts',
  async setup(__props) {

let __temp: any, __restore: any

const accounts = await getAccounts();
function refreshAllAccounts() {
	// TODO
}
function showMenu(host: string, id: string, ev: PointerEvent) {
	let menu: MenuItem[];
	menu = [{
		text: i18n.ts.switch,
		icon: 'ti ti-switch-horizontal',
		action: () => switchAccount(host, id),
	}, {
		text: i18n.ts.remove,
		icon: 'ti ti-trash',
		action: () => removeAccount(host, id),
	}];
	os.popupMenu(menu, ev.currentTarget ?? ev.target);
}
function addAccount(ev: PointerEvent) {
	os.popupMenu([{
		text: i18n.ts.existingAccount,
		action: () => { addExistingAccount(); },
	}, {
		text: i18n.ts.createAccount,
		action: () => { createAccount(); },
	}], ev.currentTarget ?? ev.target);
}
function addExistingAccount() {
	getAccountWithSigninDialog().then((res) => {
		if (res != null) {
			os.success();
		}
	});
}
function createAccount() {
	getAccountWithSignupDialog().then((res) => {
		if (res != null) {
			login(res.token);
		}
	});
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.accounts,
	icon: 'ti ti-users',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchMarker = _resolveComponent("SearchMarker")

  return (_openBlock(), _createBlock(_component_SearchMarker, {
      path: "/settings/accounts",
      label: _unref(i18n).ts.accounts,
      keywords: ['accounts'],
      icon: "ti ti-users"
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", { class: "_gaps" }, [
          _createElementVNode("div", { class: "_buttons" }, [
            _createVNode(MkButton, {
              primary: "",
              onClick: addAccount
            }, {
              default: _withCtx(() => [
                _hoisted_1,
                _createTextVNode(" "),
                _createTextVNode(_toDisplayString(_unref(i18n).ts.addAccount), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            })
          ]),
          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(accounts), (x) => {
            return (_openBlock(), _createElementBlock(_Fragment, { key: x.host + x.id }, [
              (x.user)
                ? (_openBlock(), _createBlock(MkUserCardMini, {
                  key: 0,
                  user: x.user,
                  class: _normalizeClass(_ctx.$style.user),
                  onClick: _cache[0] || (_cache[0] = _withModifiers(($event: any) => (showMenu(x.host, x.id, $event)), ["prevent"]))
                }, null, 8 /* PROPS */, ["user"]))
                : _createCommentVNode("v-if", true)
            ], 64 /* STABLE_FRAGMENT */))
          }), 128 /* KEYED_FRAGMENT */))
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["label", "keywords"]))
}
}

})
