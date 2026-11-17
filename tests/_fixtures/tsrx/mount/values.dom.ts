import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode } from "vue";
export const Values = /* @__PURE__ */ _defineComponent(function Values(__props) {
	return (_ctx, _cache) => {
  return (_createElementVNode("ul", null, [
    _createElementVNode("li", null, [
      _createTextVNode("Text: "),
      __props.text
    ]),
    _createElementVNode("li", null, [
      __props.count,
      _createTextVNode(" and "),
      __props.count + 1
    ]),
    _createElementVNode("li", null, [
      __props.flag ? 'on' : 'off'
    ]),
    _createElementVNode("li", null, [
      __props.flag ? null : 0
    ]),
    _createElementVNode("li", null, [
      __props.items.join(', ')
    ])
  ]))
};
}, { props: ["text", "count", "flag", "items"] });
