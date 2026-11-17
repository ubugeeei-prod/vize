import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode } from "vue";
import { ref } from 'vue';

export const Typed = /* @__PURE__ */ _defineComponent(function Typed(__props) {
	const count = ref<number>(0);
	return (_ctx, _cache) => {
  return (_createElementVNode("button", {
    class: "typed",
    "data-size": __props.size,
    onClick: (event) => count.value += event.type === "click" ? __props.size : 0
  }, [
    __props.label.toUpperCase(),
    _createTextVNode(": "),
    count.value + 1,
    _createElementVNode("i", null, [
      ["a", "b"].map((x) => x + __props.size).join(",")
    ])
  ], 8 /* PROPS */, ["data-size", "onClick"]))
};
}, { props: ["size", "label"] });
