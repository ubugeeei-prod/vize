import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, createVNode as _createVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode } from "vue";
import { ref } from 'vue';

const Label = /* @__PURE__ */ _defineComponent(function Label(props: { text: string; count: number }) {
	return (_ctx, _cache) => {
  return (_createElementVNode("span", { class: "label" }, [
    props.text,
    _createTextVNode(": "),
    props.count
  ]))
};
}, { props: ["text", "count"] });

export const Counter = /* @__PURE__ */ _defineComponent(function Counter(__props) {
	const count = ref(0);
	const heading = __props.title.toUpperCase();

	return (_ctx, _cache) => {
  return (_createElementVNode("section", null, [
    _createElementVNode("h1", null, [
      heading
    ]),
    _createVNode(Label, {
      text: __props.title,
      count: count.value
    }, null, 8 /* PROPS */, ["text", "count"]),
    _createElementVNode("button", { onClick: () => (count.value += __props.step) }, "add", 8 /* PROPS */, ["onClick"]),
    (count.value > 2)
      ? (_createElementVNode("p", {
        key: 0,
        class: "many"
      }, "many"))
      : (count.value > 0)
        ? (_createElementVNode("p", {
          key: 1,
          class: "some"
        }, "some"))
      : (_createElementVNode("p", {
        key: 2,
        class: "none"
      }, "none"))
  ]))
};
}, { props: ["title", "step"] });
