import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, createVNode as _createVNode, Fragment as _Fragment, renderList as _renderList } from "vue";
export const List = /* @__PURE__ */ _defineComponent(function List(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("ul", null, [
    (__props.items).length !== 0
      ? (_createElementVNode(_Fragment, { key: 0 }, _renderList(__props.items, (item) => {
        return (_createElementVNode("li", { key: item }, [
          item
        ]))
      }), 128 /* KEYED_FRAGMENT */))
      : (_createElementVNode("li", {
        key: 1,
        class: "none"
      }, "none"))
  ]))
};
}, { props: ["items"] });

export const Cards = /* @__PURE__ */ _defineComponent(function Cards(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode(_Fragment, null, [
    _createElementVNode("h1", null, [
      __props.rows.length
    ]),
    (__props.rows).length !== 0
      ? (_createElementVNode(_Fragment, { key: 0 }, _renderList(__props.rows, (row) => {
        return (_createVNode(Card, {
          key: row.id,
          text: row.text
        }, null, 8 /* PROPS */, ["text"]))
      }), 128 /* KEYED_FRAGMENT */))
      : (_createVNode(Card, {
        key: 1,
        text: "nothing"
      }))
  ], 64 /* STABLE_FRAGMENT */))
};
}, { props: ["rows"] });

const Card = /* @__PURE__ */ _defineComponent(function Card(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("p", { class: "card" }, [
    __props.text
  ]))
};
}, { props: ["text"] });
