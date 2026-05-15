import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, renderList as _renderList, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("span", { class: "ti ti-plus" });
import { toRefs } from "vue";
import MkTagItem from "@/components/MkTagItem.vue";
import MkButton from "@/components/MkButton.vue";
import * as os from "@/os.js";
export default {
  __name: "MkSortOrderEditor",
  props: {
    baseOrderKeyNames: {
      type: Array,
      required: true
    },
    currentOrders: {
      type: Array,
      required: true
    }
  },
  emits: ["update"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const { currentOrders } = toRefs(props);
    function onToggleSortOrderButtonClicked(order) {
      switch (order.direction) {
        case "+":
          order.direction = "-";
          break;
        case "-":
          order.direction = "+";
          break;
      }
      emitOrder(currentOrders.value);
    }
    function onAddSortOrderButtonClicked(ev) {
      const menuItems = props.baseOrderKeyNames.filter((baseKey) => !currentOrders.value.map((it) => it.key).includes(baseKey)).map((it) => {
        return {
          text: it,
          action: () => {
            emitOrder([...currentOrders.value, {
              key: it,
              direction: "+"
            }]);
          }
        };
      });
      os.contextMenu(menuItems, ev);
    }
    function onRemoveSortOrderButtonClicked(order) {
      emitOrder(currentOrders.value.filter((it) => it.key !== order.key));
    }
    function emitOrder(sortOrders) {
      emit("update", sortOrders);
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(_ctx.$style.sortOrderArea) },
        [_createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.sortOrderAreaTags) },
          [(_openBlock(true), _createElementBlock(
            _Fragment,
            null,
            _renderList(_unref(currentOrders), (order) => {
              return _openBlock(), _createBlock(MkTagItem, {
                key: order.key,
                iconClass: order.direction === "+" ? "ti ti-arrow-up" : "ti ti-arrow-down",
                exButtonIconClass: "ti ti-x",
                content: order.key,
                class: _normalizeClass(_ctx.$style.sortOrderTag),
                onClick: ($event) => onToggleSortOrderButtonClicked(order),
                onExButtonClick: ($event) => onRemoveSortOrderButtonClicked(order)
              }, null, 10, [
                "iconClass",
                "exButtonIconClass",
                "content",
                "onClick",
                "onExButtonClick"
              ]);
            }),
            128
            /* KEYED_FRAGMENT */
          ))],
          2
          /* CLASS */
        ), _createVNode(
          MkButton,
          {
            class: _normalizeClass(_ctx.$style.sortOrderAddButton),
            onClick: onAddSortOrderButtonClicked
          },
          {
            default: _withCtx(() => [_hoisted_1]),
            _: 1
          },
          2
          /* CLASS */
        )],
        2
        /* CLASS */
      );
    };
  }
};
