import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
import * as Misskey from 'misskey-js'
import { computed, markRaw, ref, useTemplateRef } from 'vue'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import MkButton from '@/components/MkButton.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkSelect from '@/components/MkSelect.vue'
import MkInput from '@/components/MkInput.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkPagination from '@/components/MkPagination.vue'
import MkInviteCode from '@/components/MkInviteCode.vue'
import { definePage } from '@/page.js'
import { useMkSelect } from '@/composables/use-mkselect.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'invites',
  setup(__props) {

const {
	model: type,
	def: typeDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts.all, value: 'all' },
		{ label: i18n.ts.unused, value: 'unused' },
		{ label: i18n.ts.used, value: 'used' },
		{ label: i18n.ts.expired, value: 'expired' },
	],
	initialValue: 'all',
});
const {
	model: sort,
	def: sortDef,
} = useMkSelect({
	items: [
		{ label: `${i18n.ts.createdAt} (${i18n.ts.ascendingOrder})`, value: '+createdAt' },
		{ label: `${i18n.ts.createdAt} (${i18n.ts.descendingOrder})`, value: '-createdAt' },
		{ label: `${i18n.ts.usedAt} (${i18n.ts.ascendingOrder})`, value: '+usedAt' },
		{ label: `${i18n.ts.usedAt} (${i18n.ts.descendingOrder})`, value: '-usedAt' },
	],
	initialValue: '+createdAt',
});
const paginator = markRaw(new Paginator('admin/invite/list', {
	limit: 10,
	computedParams: computed(() => ({
		type: type.value,
		sort: sort.value,
	})),
	offsetMode: true,
}));
const expiresAt = ref('');
const noExpirationDate = ref(true);
const createCount = ref(1);
async function createWithOptions() {
	const options = {
		expiresAt: noExpirationDate.value ? null : expiresAt.value,
		count: createCount.value,
	};
	const tickets = await misskeyApi('admin/invite/create', options);
	os.alert({
		type: 'success',
		title: i18n.ts.inviteCodeCreated,
		text: tickets.map(x => x.code).join('\n'),
	});
	tickets.forEach(ticket => paginator.prepend(ticket));
}
function deleted(id: string) {
	paginator.removeItem(id);
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.invite,
	icon: 'ti ti-user-plus',
}));

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 800px;"
        }, [
          _createElementVNode("div", { class: "_gaps_m" }, [
            _createVNode(MkFolder, { expanded: false }, {
              icon: _withCtx(() => [
                _hoisted_1
              ]),
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.createInviteCode), 1 /* TEXT */)
              ]),
              default: _withCtx(() => [
                _createElementVNode("div", { class: "_gaps_m" }, [
                  _createVNode(MkSwitch, {
                    modelValue: noExpirationDate.value,
                    "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((noExpirationDate).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.noExpirationDate), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["modelValue"]),
                  (!noExpirationDate.value)
                    ? (_openBlock(), _createBlock(MkInput, {
                      key: 0,
                      type: "datetime-local",
                      modelValue: expiresAt.value,
                      "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((expiresAt).value = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.expirationDate), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["modelValue"]))
                    : _createCommentVNode("v-if", true),
                  _createVNode(MkInput, {
                    type: "number",
                    min: 1,
                    modelValue: createCount.value,
                    "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((createCount).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.createCount), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["min", "modelValue"]),
                  _createVNode(MkButton, {
                    primary: "",
                    rounded: "",
                    onClick: createWithOptions
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.create), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["expanded"]),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.inputs)
            }, [
              _createVNode(MkSelect, {
                items: _unref(typeDef),
                class: _normalizeClass(_ctx.$style.input),
                modelValue: _unref(type),
                "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((type).value = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.state), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["items", "modelValue"]),
              _createVNode(MkSelect, {
                items: _unref(sortDef),
                class: _normalizeClass(_ctx.$style.input),
                modelValue: _unref(sort),
                "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((sort).value = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.sort), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["items", "modelValue"])
            ]),
            _createVNode(MkPagination, { paginator: _unref(paginator) }, {
              default: _withCtx(({ items }) => [
                _createElementVNode("div", { class: "_gaps_s" }, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
                    return (_openBlock(), _createBlock(MkInviteCode, {
                      key: item.id,
                      invite: item,
                      onDeleted: deleted,
                      moderator: ""
                    }, null, 8 /* PROPS */, ["invite", "onDeleted"]))
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
