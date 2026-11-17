import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, createVNode as _createVNode, createTextVNode as _createTextVNode, withCtx as _withCtx } from "vue";
import { useSlots } from 'vue';

const Card = /* @__PURE__ */ _defineComponent(function Card() {
	const slots = useSlots();
	return (_ctx, _cache) => {
  return slots.default?.()
};
});

export const Panel = /* @__PURE__ */ _defineComponent(function Panel(__props) {
	return (_ctx, _cache) => {
  return (_createVNode(Card, null, {
    default: _withCtx(() => [
      _createElementVNode("h2", { class: "title" }, [
        __props.title
      ]),
      _createElementVNode("p", null, [
        _createTextVNode("count: "),
        __props.count
      ])
    ]),
    _: 2 /* DYNAMIC */
  }, 1024 /* DYNAMIC_SLOTS */))
};
}, { props: ["title", "count"] });
