import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, normalizeClass as _normalizeClass, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, Fragment as _Fragment, createTextVNode as _createTextVNode, renderList as _renderList } from "vue";
import { ref } from 'vue';

export const Items = /* @__PURE__ */ _defineComponent(function Items(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode(_Fragment, null, [
    _createElementVNode("h1", null, [
      __props.items.length
    ]),
    (_createElementVNode(_Fragment, null, _renderList(__props.items, (item, i) => {
      return (_createElementVNode("li", { class: _normalizeClass(i % 2 ? 'odd' : 'even') }, [
        i,
        _createTextVNode(": "),
        item
      ], 2 /* CLASS */))
    }), 256 /* UNKEYED_FRAGMENT */))
  ], 64 /* STABLE_FRAGMENT */))
};
}, { props: ["items"] });

export const Rows = /* @__PURE__ */ _defineComponent(function Rows(__props) {
	const picked = ref(0);

	return (_ctx, _cache) => {
  return (_openBlock(), _createElementBlock(_Fragment, null, [
    _createElementVNode("button", { onClick: () => (picked.value += 1) }, "next", 8 /* PROPS */, ["onClick"]),
    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.rows, (row) => {
      return (_openBlock(), _createBlock(Row, {
        key: row.id,
        text: row.text,
        on: row.id === picked.value
      }, null, 8 /* PROPS */, ["text", "on"]))
    }), 128 /* KEYED_FRAGMENT */))
  ], 64 /* STABLE_FRAGMENT */))
};
}, { props: ["rows"] });

const Row = /* @__PURE__ */ _defineComponent(function Row(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("p", {
    class: _normalizeClass(__props.on ? 'on' : 'off')
  }, [
    __props.text
  ], 2 /* CLASS */))
};
}, { props: ["text", "on"] });
