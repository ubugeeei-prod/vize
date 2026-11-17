import { defineComponent as _defineComponent, useAttrs as _useAttrs, createElementVNode as _createElementVNode, normalizeProps as _normalizeProps, guardReactiveProps as _guardReactiveProps, createTextVNode as _createTextVNode } from "vue";
const FALLBACK = 'fallback';

export const Defaults = /* @__PURE__ */ _defineComponent(function Defaults(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("p", { "data-flag": String(__props.flag) }, [
    __props.label,
    _createTextVNode(": "),
    __props.format(__props.count),
    _createTextVNode(),
    __props.tags.join(','),
    _createTextVNode(),
    __props.fallback
  ], 8 /* PROPS */, ["data-flag"]))
};
}, { props: { "label": { type: String, default: 'none' }, "count": { type: Number, default: 1 }, "flag": { type: Boolean, default: false }, "tags": { type: Array, default: () => (['a']) }, "format": { type: Function, default: (n: number) => `#${n}` }, "fallback": { default: FALLBACK, skipFactory: true } } });

export const Renamed = /* @__PURE__ */ _defineComponent(function Renamed(__props) {
	const doubled = __props.count * 2;
	return (_ctx, _cache) => {
  return (_createElementVNode("p", { title: __props.label }, [
    __props.label,
    _createTextVNode(": "),
    __props.count,
    _createTextVNode(),
    doubled,
    _createTextVNode(),
    String({ text: __props.label }.text)
  ], 8 /* PROPS */, ["title"]))
};
}, { props: { "label": null, "count": { type: Number, default: 2 } } });

export const Rest = /* @__PURE__ */ _defineComponent(function Rest(__props) { const rest = _useAttrs();
	return (_ctx, _cache) => {
  return (_createElementVNode("p", _normalizeProps(_guardReactiveProps(rest)), [
    __props.label
  ], 16 /* FULL_PROPS */))
};
}, { inheritAttrs: false, props: ["label"] });
