'use client';
import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode } from "vue";

import { ref } from 'vue';

export const Client = /* @__PURE__ */ _defineComponent(function Client(__props) {
	const count = ref(0);

	return (_ctx, _cache) => {
  return (_createElementVNode("button", { onClick: () => count.value++ }, [
    __props.label,
    _createTextVNode(": "),
    count.value
  ], 8 /* PROPS */, ["onClick"]))
};
}, { props: ["label"] });
