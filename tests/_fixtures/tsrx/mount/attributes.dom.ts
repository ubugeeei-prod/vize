import { defineComponent as _defineComponent, createElementVNode as _createElementVNode, normalizeClass as _normalizeClass } from "vue";
export const Field = /* @__PURE__ */ _defineComponent(function Field(__props) {
	const id = `field-${__props.name}`;

	return (_ctx, _cache) => {
  return (_createElementVNode("div", {
    class: _normalizeClass(__props.tone),
    "data-name": __props.name
  }, [
    _createElementVNode("label", { for: id }, [
      __props.name
    ], 8 /* PROPS */, ["for"]),
    _createElementVNode("input", {
      id: id,
      type: "text",
      value: __props.value,
      disabled: __props.disabled
    }, null, 8 /* PROPS */, ["id", "value", "disabled"]),
    _createElementVNode("output", {
      "aria-live": "polite",
      title: __props.value.length > 3 ? 'long' : undefined
    }, [
      __props.value
    ], 8 /* PROPS */, ["title"])
  ], 10 /* CLASS, PROPS */, ["data-name"]))
};
}, { props: ["name", "value", "disabled", "tone"] });
