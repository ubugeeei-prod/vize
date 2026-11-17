import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, createVNode as _createVNode } from "vue";
const Badge = /* @__PURE__ */ _defineComponent(function Badge(props: { text: string }) {
	return (_ctx, _cache) => {
  return (_createElementVNode("b", { class: "badge" }, [
    props.text
  ]))
};
}, { props: ["text"] });

export default /* @__PURE__ */ _defineComponent(function App(__props) {
	const heading = __props.title.toUpperCase();
	return (_ctx, _cache) => {
  return (_createElementVNode("section", { class: "app" }, [
    _createElementVNode("h1", null, [
      heading
    ]),
    _createVNode(Badge, { text: __props.title }, null, 8 /* PROPS */, ["text"]),
    _createElementVNode("p", null, [
      __props.count
    ])
  ]))
};
}, { props: ["title", "count"] });
