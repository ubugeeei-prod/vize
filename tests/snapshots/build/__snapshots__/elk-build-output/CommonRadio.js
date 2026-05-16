import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, vModelRadio as _vModelRadio, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = { "flex-1": "true", "ms-2": "true", "pointer-events-none": "true" }

export default /*@__PURE__*/_defineComponent({
  __name: 'CommonRadio',
  props: {
    label: { type: String, required: true },
    value: { type: null, required: true },
    hover: { type: Boolean, required: false },
    "modelValue": {}
  },
  emits: ["update:modelValue"],
  setup(__props: any) {

const modelValue = _useModel(__props, "modelValue")

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("label", {
      class: _normalizeClass(["common-radio flex items-center cursor-pointer py-1 text-md w-full gap-y-1", __props.hover ? 'hover:bg-active ms--2 px-4 py-2' : null]),
      onClick: _cache[0] || (_cache[0] = _withModifiers(($event: any) => (modelValue.value = __props.value), ["prevent"]))
    }, [ _createElementVNode("span", _hoisted_1, _toDisplayString(__props.label), 1 /* TEXT */), _createElementVNode("span", {
        class: _normalizeClass(modelValue.value === __props.value ? 'i-ri:radio-button-line' : 'i-ri:checkbox-blank-circle-line'),
        "aria-hidden": "true"
      }, null, 2 /* CLASS */), _withDirectives(_createElementVNode("input", {
        "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((modelValue).value = $event)),
        type: "radio",
        value: __props.value,
        "sr-only": ""
      }, null, 8 /* PROPS */, ["value"]), [ [_vModelRadio, modelValue.value] ]) ], 2 /* CLASS */))
}
}

})
