import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { computed, markRaw, ref, watchEffect } from 'vue'
import * as Misskey from 'misskey-js'
import { defaultMemoryStorage } from '@/memory-storage'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import MkSelect from '@/components/MkSelect.vue'
import MkPagination from '@/components/MkPagination.vue'
import * as os from '@/os.js'
import { lookupUser } from '@/utility/admin-lookup.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { useMkSelect } from '@/composables/use-mkselect.js'
import MkUserCardMini from '@/components/MkUserCardMini.vue'
import { dateString } from '@/filters/date.js'
import { Paginator } from '@/utility/paginator.js'

type SearchQuery = {
	sort?: '-createdAt' | '+createdAt' | '-updatedAt' | '+updatedAt';
	state?: 'all' | 'available' | 'admin' | 'moderator' | 'suspended';
	origin?: 'combined' | 'local' | 'remote';
	username?: string;
	hostname?: string;
};

export default /*@__PURE__*/_defineComponent({
  __name: 'users',
  setup(__props) {

const storedQuery = JSON.parse(defaultMemoryStorage.getItem('admin-users-query') ?? '{}') as SearchQuery;
const {
	model: sort,
	def: sortDef,
} = useMkSelect({
	items: [
		{ label: `${i18n.ts.registeredDate} (${i18n.ts.ascendingOrder})`, value: '-createdAt' },
		{ label: `${i18n.ts.registeredDate} (${i18n.ts.descendingOrder})`, value: '+createdAt' },
		{ label: `${i18n.ts.lastUsed} (${i18n.ts.ascendingOrder})`, value: '-updatedAt' },
		{ label: `${i18n.ts.lastUsed} (${i18n.ts.descendingOrder})`, value: '+updatedAt' },
	],
	initialValue: storedQuery.sort ?? '+createdAt',
});
const {
	model: state,
	def: stateDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts.all, value: 'all' },
		{ label: i18n.ts.normal, value: 'available' },
		{ label: i18n.ts.administrator, value: 'admin' },
		{ label: i18n.ts.moderator, value: 'moderator' },
		{ label: i18n.ts.suspend, value: 'suspended' },
	],
	initialValue: storedQuery.state ?? 'all',
});
const {
	model: origin,
	def: originDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts.all, value: 'combined' },
		{ label: i18n.ts.local, value: 'local' },
		{ label: i18n.ts.remote, value: 'remote' },
	],
	initialValue: storedQuery.origin ?? 'local',
});
const searchUsername = ref(storedQuery.username ?? '');
const searchHost = ref(storedQuery.hostname ?? '');
const paginator = markRaw(new Paginator('admin/show-users', {
	limit: 10,
	computedParams: computed(() => ({
		sort: sort.value,
		state: state.value,
		origin: origin.value,
		username: searchUsername.value,
		hostname: searchHost.value,
	})),
	offsetMode: true,
}));
function searchUser() {
	os.selectUser({ includeSelf: true }).then(user => {
		show(user);
	});
}
async function addUser() {
	const { canceled: canceled1, result: username } = await os.inputText({
		title: i18n.ts.username,
	});
	if (canceled1 || username == null) return;
	const { canceled: canceled2, result: password } = await os.inputText({
		title: i18n.ts.password,
		type: 'password',
	});
	if (canceled2 || password == null) return;
	os.apiWithDialog('admin/accounts/create', {
		username: username,
		password: password,
	}).then(res => {
		paginator.reload();
	});
}
function show(user: Misskey.entities.UserDetailed) {
	os.pageWindow(`/admin/user/${user.id}`);
}
function resetQuery() {
	sort.value = '+createdAt';
	state.value = 'all';
	origin.value = 'local';
	searchUsername.value = '';
	searchHost.value = '';
}
const headerActions = computed(() => [{
	icon: 'ti ti-search',
	text: i18n.ts.search,
	handler: searchUser,
}, {
	asFullButton: true,
	icon: 'ti ti-plus',
	text: i18n.ts.addUser,
	handler: addUser,
}, {
	asFullButton: true,
	icon: 'ti ti-search',
	text: i18n.ts.lookup,
	handler: lookupUser,
}]);
const headerTabs = computed(() => []);
watchEffect(() => {
	defaultMemoryStorage.setItem('admin-users-query', JSON.stringify({
		sort: sort.value,
		state: state.value,
		origin: origin.value,
		username: searchUsername.value,
		hostname: searchHost.value,
	}));
});
definePage(() => ({
	title: i18n.ts.users,
	icon: 'ti ti-users',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 900px;"
        }, [
          _createElementVNode("div", { class: "_gaps" }, [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.inputs)
            }, [
              _createVNode(MkButton, {
                style: "margin-left: auto",
                onClick: resetQuery
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.reset), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.inputs)
            }, [
              _createVNode(MkSelect, {
                items: _unref(sortDef),
                style: "flex: 1;",
                modelValue: _unref(sort),
                "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((sort).value = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.sort), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["items", "modelValue"]),
              _createVNode(MkSelect, {
                items: _unref(stateDef),
                style: "flex: 1;",
                modelValue: _unref(state),
                "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((state).value = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.state), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["items", "modelValue"]),
              _createVNode(MkSelect, {
                items: _unref(originDef),
                style: "flex: 1;",
                modelValue: _unref(origin),
                "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((origin).value = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.instance), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["items", "modelValue"])
            ]),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.inputs)
            }, [
              _createVNode(MkInput, {
                style: "flex: 1;",
                type: "text",
                spellcheck: false,
                modelValue: searchUsername.value,
                "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((searchUsername).value = $event))
              }, {
                prefix: _withCtx(() => [
                  _createTextVNode("@")
                ]),
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.username), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["spellcheck", "modelValue"]),
              _createVNode(MkInput, {
                style: "flex: 1;",
                type: "text",
                spellcheck: false,
                disabled: _unref(paginator).computedParams?.value?.origin === 'local',
                modelValue: searchHost.value,
                "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((searchHost).value = $event))
              }, {
                prefix: _withCtx(() => [
                  _createTextVNode("@")
                ]),
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.host), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["spellcheck", "disabled", "modelValue"])
            ]),
            _createVNode(MkPagination, { paginator: _unref(paginator) }, {
              default: _withCtx(({items}) => [
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.users)
                }, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (user) => {
                    return (_openBlock(), _createBlock(_component_MkA, {
                      key: user.id,
                      class: _normalizeClass(_ctx.$style.user),
                      to: `/admin/user/${user.id}`
                    }, {
                      default: _withCtx(() => [
                        _createVNode(MkUserCardMini, { user: user }, null, 8 /* PROPS */, ["user"])
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["to"]))
                  }), 128 /* KEYED_FRAGMENT */))
                ])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["paginator"])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
