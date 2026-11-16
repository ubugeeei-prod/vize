import { defineComponent as _defineComponent, resolveDynamicComponent as _resolveDynamicComponent, createElementVNode as _createElementVNode, createVNode as _createVNode, openBlock as _openBlock, createBlock as _createBlock, Fragment as _Fragment, withCtx as _withCtx } from "vue";
import { ref } from 'vue';

const Child = /* @__PURE__ */ _defineComponent(function Child(props: { label: string }) {
	return (_ctx, _cache) => {
  return (_createElementVNode("span", { class: "child" }, [
    props.label
  ]))
};
}, { props: ["label"] });

export const Host = /* @__PURE__ */ _defineComponent(function Host(__props) {
	return (_ctx, _cache) => {
  return (_createVNode(_resolveDynamicComponent(__props.tag), {
    class: "host",
    "data-tag": __props.tag
  }, {
    default: _withCtx(() => [
      'dynamic ',
      _createElementVNode("b", null, [
        __props.tag
      ])
    ]),
    _: 2 /* DYNAMIC */
  }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["data-tag"]))
};
}, { props: ["tag"] });

export const Toggle = /* @__PURE__ */ _defineComponent(function Toggle() {
	const tag = ref<'section' | 'article'>('section');
	return (_ctx, _cache) => {
  return (_createElementVNode(_Fragment, null, [
    (_createVNode(_resolveDynamicComponent(tag.value), { class: "toggle" }, {
      default: _withCtx(() => [
        tag.value
      ]),
      _: 2 /* DYNAMIC */
    }, 1024 /* DYNAMIC_SLOTS */)),
    _createElementVNode("button", { onClick: () => {
				tag.value = tag.value === 'section' ? 'article' : 'section';
			} }, "toggle", 8 /* PROPS */, ["onClick"])
  ], 64 /* STABLE_FRAGMENT */))
};
});

export const Pick = /* @__PURE__ */ _defineComponent(function Pick(__props) {
	const component = Child;
	return (_ctx, _cache) => {
  return (_openBlock(), _createBlock(_resolveDynamicComponent(component), { label: __props.label }, null, 8 /* PROPS */, ["label"]))
};
}, { props: ["label"] });
