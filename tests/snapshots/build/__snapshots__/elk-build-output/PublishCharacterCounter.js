import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { "text-secondary-light": "true" }, "/")
const _hoisted_2 = { "text-secondary-light": "true" }

export default /*@__PURE__*/_defineComponent({
  __name: 'PublishCharacterCounter',
  props: {
    max: { type: Number, required: true },
    length: { type: Number, required: true }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      dir: "ltr",
      "pointer-events-none": "",
      "pe-1": "",
      "pt-2": "",
      "text-sm": "",
      "tabular-nums": "",
      "text-secondary": "",
      flex: "",
      gap: "0.5",
      class: _normalizeClass({ 'text-rose-500': __props.length > __props.max })
    }, [ _createTextVNode(_toDisplayString(__props.length ?? 0), 1 /* TEXT */), _hoisted_1, _createElementVNode("span", _hoisted_2, _toDisplayString(__props.max), 1 /* TEXT */) ], 2 /* CLASS */))
}
}

})
