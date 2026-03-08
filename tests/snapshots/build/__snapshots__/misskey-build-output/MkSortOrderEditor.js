import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, renderList as _renderList, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "ti ti-plus" })
import { toRefs } from 'vue'
import type { MenuItem } from '@/types/menu.js'
import type { SortOrder } from '@/components/MkSortOrderEditor.define.js'
import MkTagItem from '@/components/MkTagItem.vue'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSortOrderEditor',
  props: {
    baseOrderKeyNames: { type: Array, required: true },
    currentOrders: { type: Array, required: true }
  },
  emits: ["update"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const { currentOrders } = toRefs(props);
function onToggleSortOrderButtonClicked(order: SortOrder<T>) {
	switch (order.direction) {
		case '+':
			order.direction = '-';
			break;
		case '-':
			order.direction = '+';
			break;
	}
	emitOrder(currentOrders.value);
}
function onAddSortOrderButtonClicked(ev: PointerEvent) {
	const menuItems: MenuItem[] = props.baseOrderKeyNames
		.filter(baseKey => !currentOrders.value.map(it => it.key).includes(baseKey))
		.map(it => {
			return {
				text: it,
				action: () => {
					emitOrder([...currentOrders.value, { key: it, direction: '+' }]);
				},
			};
		});
	os.contextMenu(menuItems, ev);
}
function onRemoveSortOrderButtonClicked(order: SortOrder<T>) {
	emitOrder(currentOrders.value.filter(it => it.key !== order.key));
}
function emitOrder(sortOrders: SortOrder<T>[]) {
	emit('update', sortOrders);
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.sortOrderArea)
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.sortOrderAreaTags)
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(currentOrders), (order) => {
          return (_openBlock(), _createBlock(MkTagItem, {
            key: order.key,
            iconClass: order.direction === '+' ? 'ti ti-arrow-up' : 'ti ti-arrow-down',
            exButtonIconClass: 'ti ti-x',
            content: order.key,
            class: _normalizeClass(_ctx.$style.sortOrderTag),
            onClick: _cache[0] || (_cache[0] = ($event: any) => (onToggleSortOrderButtonClicked(order))),
            onExButtonClick: _cache[1] || (_cache[1] = ($event: any) => (onRemoveSortOrderButtonClicked(order)))
          }, null, 8 /* PROPS */, ["iconClass", "exButtonIconClass", "content"]))
        }), 128 /* KEYED_FRAGMENT */)) ]), _createVNode(MkButton, {
        class: _normalizeClass(_ctx.$style.sortOrderAddButton),
        onClick: onAddSortOrderButtonClicked
      }, {
        default: _withCtx(() => [
          _hoisted_1
        ]),
        _: 1 /* STABLE */
      }) ]))
}
}

})
