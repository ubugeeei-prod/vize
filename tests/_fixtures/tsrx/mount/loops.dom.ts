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

export const Values = /* @__PURE__ */ _defineComponent(function Values(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode(_Fragment, null, _renderList(__props.items, (item) => {
    return (_createElementVNode("li", {
      key: item,
      class: "value"
    }, [
      item
    ]))
  }), 128 /* KEYED_FRAGMENT */))
};
}, { props: ["items"] });

export const Indexed = /* @__PURE__ */ _defineComponent(function Indexed(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode(_Fragment, null, _renderList(__props.items, (item, i) => {
    return (_createElementVNode("li", { key: i }, [
      i,
      _createTextVNode(": "),
      item
    ]))
  }), 128 /* KEYED_FRAGMENT */))
};
}, { props: ["items"] });

export const Owned = /* @__PURE__ */ _defineComponent(function Owned(__props) {
	return (_ctx, _cache) => {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.rows, (row) => {
    return (_openBlock(), _createBlock(Pair, {
      key: row.id,
      id: row.id,
      text: row.text
    }, null, 8 /* PROPS */, ["id", "text"]))
  }), 128 /* KEYED_FRAGMENT */))
};
}, { props: ["rows"] });

export const Terms = /* @__PURE__ */ _defineComponent(function Terms(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode(_Fragment, null, _renderList(__props.rows, (row) => {
    return (_createElementVNode(_Fragment, { key: row.id }, [
      _createElementVNode("dt", null, [
        row.id
      ]),
      _createElementVNode("dd", { class: "term" }, [
        row.text
      ])
    ], 64 /* STABLE_FRAGMENT */))
  }), 128 /* KEYED_FRAGMENT */))
};
}, { props: ["rows"] });

export const Labels = /* @__PURE__ */ _defineComponent(function Labels(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode(_Fragment, null, _renderList(__props.items, (item, i) => {
    return (_createElementVNode(_Fragment, { key: i }, [
      i,
      _createElementVNode("b", null, [
        item
      ])
    ], 64 /* STABLE_FRAGMENT */))
  }), 128 /* KEYED_FRAGMENT */))
};
}, { props: ["items"] });
