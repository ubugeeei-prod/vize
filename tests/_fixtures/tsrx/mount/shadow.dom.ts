import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, Fragment as _Fragment, createTextVNode as _createTextVNode, renderList as _renderList } from "vue";
export const Shadow = /* @__PURE__ */ _defineComponent(function Shadow(__props) {
	const upper = __props.label.toUpperCase();
	const pick = (label: string) => label + upper;
	function twice(label: string) {
		return label + label;
	}
	let total = 0;
	for (const label of __props.items) {
		total += label.length;
	}
	{
		const label = 'block';
		total += label.length;
	}

	return (_ctx, _cache) => {
  return (_createElementVNode(_Fragment, null, [
    _createElementVNode("h1", { title: __props.label }, [
      pick('x'),
      _createTextVNode(),
      twice('y'),
      _createTextVNode(),
      total
    ], 8 /* PROPS */, ["title"]),
    _createElementVNode("p", null, [
      __props.items.map((label) => label.toUpperCase()).join(',')
    ]),
    (_createElementVNode(_Fragment, null, _renderList(__props.items, (label) => {
      return (_createElementVNode("li", { key: label }, [
        label
      ]))
    }), 128 /* KEYED_FRAGMENT */))
  ], 64 /* STABLE_FRAGMENT */))
};
}, { props: ["label", "items"] });
