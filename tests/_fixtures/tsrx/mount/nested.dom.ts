import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, createVNode as _createVNode, Fragment as _Fragment, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode } from "vue";
import { ref } from 'vue';

export const Toggle = /* @__PURE__ */ _defineComponent(function Toggle(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode(_Fragment, null, [
    _createElementVNode("h1", null, [
      __props.n
    ]),
    (__props.on)
      ? (_createElementVNode("em", {
        key: 0,
        class: "on"
      }, [
        _createTextVNode("yes "),
        __props.n
      ]))
      : (_createElementVNode("span", { key: 1 }, "no"))
  ], 64 /* STABLE_FRAGMENT */))
};
}, { props: ["on", "n"] });

export const Modes = /* @__PURE__ */ _defineComponent(function Modes(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode(_Fragment, null, [
    (__props.mode === 'a')
      ? (_createVNode(Badge, {
        key: 0,
        text: __props.label
      }, null, 8 /* PROPS */, ["text"]))
      : (__props.mode === 'b')
        ? (_createElementVNode("b", { key: 1 }, [
          __props.label
        ]))
      : _createCommentVNode("v-if", true),
    _createElementVNode("hr")
  ], 64 /* STABLE_FRAGMENT */))
};
}, { props: ["mode", "label"] });

const Badge = /* @__PURE__ */ _defineComponent(function Badge(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("i", { class: "badge" }, [
    __props.text
  ]))
};
}, { props: ["text"] });

export const Guard = /* @__PURE__ */ _defineComponent(function Guard(__props) {
	return (_ctx, _cache) => {
  return (__props.count > 0)
    ? (_createElementVNode("p", {
      key: 0,
      "data-label": __props.label
    }, [
      __props.label
    ], 8 /* PROPS */, ["data-label"]))
    : _createCommentVNode("v-if", true)
};
}, { props: ["count", "label"] });

export const Disclosure = /* @__PURE__ */ _defineComponent(function Disclosure(__props) {
	const open = ref(false);

	return (_ctx, _cache) => {
  return (_createElementVNode(_Fragment, null, [
    _createElementVNode("button", { onClick: () => (open.value = !open.value) }, [
      __props.title
    ], 8 /* PROPS */, ["onClick"]),
    (open.value && __props.title.length > 3)
      ? (_createElementVNode("section", {
        key: 0,
        class: "long"
      }, [
        __props.title
      ]))
      : (open.value)
        ? (_createElementVNode("section", { key: 1 }, "short"))
      : _createCommentVNode("v-if", true)
  ], 64 /* STABLE_FRAGMENT */))
};
}, { props: ["title"] });
