import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-search" })
import * as Misskey from 'misskey-js'
import { computed, markRaw, ref } from 'vue'
import MkInput from '@/components/MkInput.vue'
import MkSelect from '@/components/MkSelect.vue'
import MkPagination from '@/components/MkPagination.vue'
import MkInstanceCardMini from '@/components/MkInstanceCardMini.vue'
import FormSplit from '@/components/form/split.vue'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { useMkSelect } from '@/composables/use-mkselect.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'federation',
  setup(__props) {

const host = ref('');
const {
	model: state,
	def: stateDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts.all, value: 'all' },
		{ label: i18n.ts.federating, value: 'federating' },
		{ label: i18n.ts.subscribing, value: 'subscribing' },
		{ label: i18n.ts.publishing, value: 'publishing' },
		{ label: i18n.ts.suspended, value: 'suspended' },
		{ label: i18n.ts.blocked, value: 'blocked' },
		{ label: i18n.ts.silence, value: 'silenced' },
		{ label: i18n.ts.notResponding, value: 'notResponding' },
	],
	initialValue: 'federating',
});
const {
	model: sort,
	def: sortDef,
} = useMkSelect({
	items: [
		{ label: `${i18n.ts.pubSub} (${i18n.ts.descendingOrder})`, value: '+pubSub' },
		{ label: `${i18n.ts.pubSub} (${i18n.ts.ascendingOrder})`, value: '-pubSub' },
		{ label: `${i18n.ts.notes} (${i18n.ts.descendingOrder})`, value: '+notes' },
		{ label: `${i18n.ts.notes} (${i18n.ts.ascendingOrder})`, value: '-notes' },
		{ label: `${i18n.ts.users} (${i18n.ts.descendingOrder})`, value: '+users' },
		{ label: `${i18n.ts.users} (${i18n.ts.ascendingOrder})`, value: '-users' },
		{ label: `${i18n.ts.following} (${i18n.ts.descendingOrder})`, value: '+following' },
		{ label: `${i18n.ts.following} (${i18n.ts.ascendingOrder})`, value: '-following' },
		{ label: `${i18n.ts.followers} (${i18n.ts.descendingOrder})`, value: '+followers' },
		{ label: `${i18n.ts.followers} (${i18n.ts.ascendingOrder})`, value: '-followers' },
		{ label: `${i18n.ts.registeredAt} (${i18n.ts.descendingOrder})`, value: '+firstRetrievedAt' },
		{ label: `${i18n.ts.registeredAt} (${i18n.ts.ascendingOrder})`, value: '-firstRetrievedAt' },
	],
	initialValue: '+pubSub',
});
const paginator = markRaw(new Paginator('federation/instances', {
	limit: 10,
	offsetMode: true,
	computedParams: computed(() => ({
		sort: sort.value,
		host: host.value !== '' ? host.value : null,
		...(
			state.value === 'federating' ? { federating: true, suspended: false, blocked: false } :
			state.value === 'subscribing' ? { subscribing: true, suspended: false, blocked: false } :
			state.value === 'publishing' ? { publishing: true, suspended: false, blocked: false } :
			state.value === 'suspended' ? { suspended: true } :
			state.value === 'blocked' ? { blocked: true } :
			state.value === 'silenced' ? { silenced: true } :
			state.value === 'notResponding' ? { notResponding: true } :
			{}),
	})),
}));
function getStatus(instance: Misskey.entities.FederationInstance) {
	switch (instance.suspensionState) {
		case 'manuallySuspended':
			return 'Manually Suspended';
		case 'goneSuspended':
			return 'Automatically Suspended (Gone)';
		case 'autoSuspendedForNotResponding':
			return 'Automatically Suspended (Not Responding)';
		case 'none':
			break;
	}
	if (instance.isBlocked) return 'Blocked';
	if (instance.isSilenced) return 'Silenced';
	if (instance.isNotResponding) return 'Error';
	return 'Alive';
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.federation,
	icon: 'ti ti-whirl',
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
            _createElementVNode("div", null, [
              _createVNode(MkInput, {
                debounce: true,
                class: "",
                modelValue: host.value,
                "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((host).value = $event))
              }, {
                prefix: _withCtx(() => [
                  _hoisted_1
                ]),
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.host), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["debounce", "modelValue"]),
              _createVNode(FormSplit, { style: "margin-top: var(--MI-margin);" }, {
                default: _withCtx(() => [
                  _createVNode(MkSelect, {
                    items: _unref(stateDef),
                    modelValue: _unref(state),
                    "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((state).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.state), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["items", "modelValue"]),
                  _createVNode(MkSelect, {
                    items: _unref(sortDef),
                    modelValue: _unref(sort),
                    "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((sort).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.sort), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["items", "modelValue"])
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _createVNode(MkPagination, {
              key: host.value + _unref(state),
              paginator: _unref(paginator)
            }, {
              default: _withCtx(({items}) => [
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.instances)
                }, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (instance) => {
                    return (_openBlock(), _createBlock(_component_MkA, {
                      key: instance.id,
                      class: _normalizeClass(_ctx.$style.instance),
                      to: `/instance-info/${instance.host}`
                    }, {
                      default: _withCtx(() => [
                        _createVNode(MkInstanceCardMini, { instance: instance }, null, 8 /* PROPS */, ["instance"])
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
