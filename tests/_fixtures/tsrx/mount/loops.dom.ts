import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, Fragment as _Fragment, createTextVNode as _createTextVNode, renderList as _renderList } from "vue";
export const Rows = /* @__PURE__ */ _defineComponent(function Rows(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode(_Fragment, null, _renderList(__props.rows, (row, i) => {
    return (_createElementVNode("li", {
      key: row.id,
      class: "row"
    }, [
      i,
      _createTextVNode(": "),
      row.text
    ]))
  }), 128 /* KEYED_FRAGMENT */))
};
}, { props: ["rows"] });

export const Pairs = /* @__PURE__ */ _defineComponent(function Pairs(__props) {
	return (_ctx, _cache) => {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.rows, ({ id, text }) => {
    return (_openBlock(), _createBlock(Pair, {
      key: id,
      id: id,
      text: text
    }, null, 8 /* PROPS */, ["id", "text"]))
  }), 128 /* KEYED_FRAGMENT */))
};
}, { props: ["rows"] });

const Pair = /* @__PURE__ */ _defineComponent(function Pair(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("p", { "data-id": __props.id }, [
    __props.text
  ], 8 /* PROPS */, ["data-id"]))
};
}, { props: ["id", "text"] });

export const Nested = /* @__PURE__ */ _defineComponent(function Nested(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("ul", null, [
    (_createElementVNode(_Fragment, null, _renderList(__props.rows, (row) => {
      return (_createElementVNode("li", { key: row.id }, [
        row.text
      ]))
    }), 128 /* KEYED_FRAGMENT */))
  ]))
};
}, { props: ["rows"] });
