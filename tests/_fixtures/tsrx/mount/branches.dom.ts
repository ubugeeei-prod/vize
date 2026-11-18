import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, openBlock as _openBlock, createElementBlock as _createElementBlock, Fragment as _Fragment, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode } from "vue";
export const Ternary = /* @__PURE__ */ _defineComponent(function Ternary(__props) {
	return (_ctx, _cache) => {
  return (_openBlock(), _createElementBlock("p", null, [
    (__props.a)
      ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
        (__props.b)
          ? (_openBlock(), _createElementBlock("i", { key: 0 }, "b"))
          : (_openBlock(), _createElementBlock("s", { key: 1 }, "not b"))
      ], 64 /* STABLE_FRAGMENT */))
      : (_openBlock(), _createElementBlock("u", { key: 1 }, "not a"))
  ]))
};
}, { props: ["a", "b"] });

export const Blocks = /* @__PURE__ */ _defineComponent(function Blocks(__props) {
	return (_ctx, _cache) => {
  return (_openBlock(), _createElementBlock("p", null, [
    (__props.a)
      ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
        (__props.b)
          ? (_openBlock(), _createElementBlock("i", { key: 0 }, "b"))
          : (_openBlock(), _createElementBlock("s", { key: 1 }, "not b"))
      ], 64 /* STABLE_FRAGMENT */))
      : _createCommentVNode("v-if", true)
  ]))
};
}, { props: ["a", "b"] });

export const Labels = /* @__PURE__ */ _defineComponent(function Labels(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("p", null, [
    (__props.a)
      ? (_createElementVNode(_Fragment, { key: 0 }, [
        (__props.b)
          ? (_createElementVNode("i", { key: 0 }, [
            __props.label
          ]))
          : (_createElementVNode("s", { key: 1 }, [
            __props.label
          ]))
      ], 64 /* STABLE_FRAGMENT */))
      : _createCommentVNode("v-if", true)
  ]))
};
}, { props: ["a", "b", "label"] });

export const Several = /* @__PURE__ */ _defineComponent(function Several(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("p", null, [
    (__props.a)
      ? (_createElementVNode(_Fragment, { key: 0 }, [
        _createElementVNode("i", null, [
          __props.n
        ]),
        _createElementVNode("s", null, "two")
      ], 64 /* STABLE_FRAGMENT */))
      : (_createElementVNode("u", { key: 1 }, "none"))
  ]))
};
}, { props: ["a", "n"] });

export const Texts = /* @__PURE__ */ _defineComponent(function Texts(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("p", null, [
    (__props.a)
      ? (_createElementVNode(_Fragment, { key: 0 }, [
        _createTextVNode("count "),
        __props.n
      ], 64 /* STABLE_FRAGMENT */))
      : _createCommentVNode("v-if", true)
  ]))
};
}, { props: ["a", "n"] });

export const AndIf = /* @__PURE__ */ _defineComponent(function AndIf(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("p", null, [
    (__props.n)
      && (_createElementVNode(_Fragment, { key: 0 }, [
        (__props.a)
          ? (_createElementVNode("i", { key: 0 }, "a"))
          : (_createElementVNode("s", { key: 1 }, "not a"))
      ], 64 /* STABLE_FRAGMENT */))
  ]))
};
}, { props: ["a", "n"] });

export const IfAnd = /* @__PURE__ */ _defineComponent(function IfAnd(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("p", null, [
    (__props.a)
      ? (_createElementVNode(_Fragment, { key: 0 }, [
        (__props.n)
          && (_createElementVNode("i", { key: 0 }, [
            __props.n
          ]))
      ], 64 /* STABLE_FRAGMENT */))
      : (_createElementVNode("u", { key: 1 }, "not a"))
  ]))
};
}, { props: ["a", "n"] });
