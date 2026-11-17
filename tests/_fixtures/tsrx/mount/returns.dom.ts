import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, createVNode as _createVNode, createTextVNode as _createTextVNode } from "vue";
import { ref } from 'vue';

const Child = /* @__PURE__ */ _defineComponent(function Child(props: { text: string }) {
	return (_ctx, _cache) => {
  return (_createElementVNode("b", { class: "child" }, [
    props.text
  ]))
};
}, { props: ["text"] });

export const Plain = /* @__PURE__ */ _defineComponent(function Plain(__props) {
	const upper = __props.label.toUpperCase();
	return (
		(_ctx, _cache) => {
  return (_createElementVNode("p", {
    class: "plain",
    "data-count": __props.count
  }, [
    upper,
    _createTextVNode(": "),
    _createVNode(Child, { text: __props.label }, null, 8 /* PROPS */, ["text"])
  ], 8 /* PROPS */, ["data-count"]))
}
	);
}, { props: ["label", "count"] });

export const Arrow = /* @__PURE__ */ _defineComponent((__props) => (_ctx, _cache) => {
  return (_createElementVNode("span", { "data-count": __props.count }, [
    __props.count * 2
  ], 8 /* PROPS */, ["data-count"]))
}, { props: ["count"] });

export const Grow = /* @__PURE__ */ _defineComponent((props: { items: string[] }) => {
	const size = ref(1);
	return (
		(_ctx, _cache) => {
  return (_createElementVNode("div", null, [
    _createElementVNode("button", { onClick: () => (size.value += 1) }, "grow", 8 /* PROPS */, ["onClick"]),
    _createElementVNode("p", null, [
      props.items.slice(0, size.value).join(',')
    ])
  ]))
}
	);
}, { props: ["items"] });
