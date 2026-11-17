import { defineComponent as _defineComponent, computed as _computed, useAttrs as _useAttrs, createElementVNode as _createElementVNode, normalizeProps as _normalizeProps, guardReactiveProps as _guardReactiveProps, mergeProps as _mergeProps, openBlock as _openBlock, createElementBlock as _createElementBlock, createTextVNode as _createTextVNode } from "vue";
const _useFullProps = (props) => {
	const attrs = _useAttrs();
	const full = _computed(() => ({ ...props, ...attrs }));
	return new Proxy({}, {
		get: (_, key, receiver) => Reflect.get(full.value, key, receiver),
		set: (_, key, value, receiver) => Reflect.set(full.value, key, value, receiver),
		deleteProperty: (_, key) => Reflect.deleteProperty(full.value, key),
		has: (_, key) => Reflect.has(full.value, key),
		ownKeys: () => Object.keys(full.value),
		getOwnPropertyDescriptor: () => ({ enumerable: true, configurable: true }),
	});
};
export const Spread = /* @__PURE__ */ _defineComponent(function Spread(__props) { const props = _useAttrs();
	return (_ctx, _cache) => {
  return (_openBlock(), _createElementBlock("div", _normalizeProps(_guardReactiveProps(props)), null, 16 /* FULL_PROPS */))
};
});

export const Merged = /* @__PURE__ */ _defineComponent(function Merged(__props) { const props = _useFullProps(__props);
	return (_ctx, _cache) => {
  return (_createElementVNode("p", _mergeProps({
    class: props.kind
  }, props), [
    props.kind
  ], 16 /* FULL_PROPS */))
};
}, { props: ["kind"] });

export const Alias = /* @__PURE__ */ _defineComponent(function Alias(__props) { const props = _useAttrs();
	const p = props;
	return (_ctx, _cache) => {
  return (_createElementVNode("p", null, [
    p.title,
    _createTextVNode(": "),
    JSON.stringify(p)
  ]))
};
});
