import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, createVNode as _createVNode } from "vue";
const values: Record<string, unknown> = {
	zero: 0,
	empty: '',
	nan: NaN,
	none: null,
	missing: undefined,
	no: false,
	big: 0n,
	text: 'x',
	yes: true,
};

export const Count = /* @__PURE__ */ _defineComponent(function Count(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("p", null, [
    (__props.n)
      && (_createElementVNode("b", {
        key: 0,
        class: "count"
      }, [
        __props.label
      ]))
  ]))
};
}, { props: ["n", "label"] });

export const Values = /* @__PURE__ */ _defineComponent(function Values(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("div", null, [
    _cache[0] || (_cache[0] = _createElementVNode("span", null, "before", -1 /* CACHED */)),
    (values[__props.kind])
      && (_createElementVNode("em", { key: 0 }, "set")),
    _cache[1] || (_cache[1] = _createElementVNode("span", null, "after", -1 /* CACHED */))
  ]))
};
}, { props: ["kind"] });

export const Root = /* @__PURE__ */ _defineComponent(function Root(__props) {
	return (_ctx, _cache) => {
  return (__props.n)
    && (_createElementVNode("i", { key: 0 }, [
      __props.n
    ]))
};
}, { props: ["n"] });

export const Pair = /* @__PURE__ */ _defineComponent(function Pair(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("ul", null, [
    (__props.a)
      && (_createElementVNode("li", {
        key: 0,
        class: "a"
      }, [
        __props.a
      ])),
    (__props.b)
      && (_createElementVNode("li", {
        key: 1,
        class: "b"
      }, [
        __props.b
      ]))
  ]))
};
}, { props: ["a", "b"] });

const Badge = /* @__PURE__ */ _defineComponent(function Badge(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("mark", null, [
    __props.text
  ]))
};
}, { props: ["text"] });

export const Tagged = /* @__PURE__ */ _defineComponent(function Tagged(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("section", null, [
    (__props.count)
      && (_createVNode(Badge, {
        key: 0,
        text: __props.title
      }, null, 8 /* PROPS */, ["text"]))
  ]))
};
}, { props: ["count", "title"] });

export const Chained = /* @__PURE__ */ _defineComponent(function Chained(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("p", null, [
    (__props.n && __props.m)
      && (_createElementVNode("b", { key: 0 }, [
        __props.m
      ]))
  ]))
};
}, { props: ["n", "m"] });

export const Drawn = /* @__PURE__ */ _defineComponent(function Drawn(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("svg", null, [
    (__props.n)
      && (_createElementVNode("circle", {
        key: 0,
        r: __props.n
      }, null, 8 /* PROPS */, ["r"]))
  ]))
};
}, { props: ["n"] });
