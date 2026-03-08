import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, withDirectives as _withDirectives, toDisplayString as _toDisplayString, mergeProps as _mergeProps, normalizeClass as _normalizeClass, vModelCheckbox as _vModelCheckbox, withModifiers as _withModifiers } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'CommonCheckbox',
  props: {
    label: { type: String, required: false },
    hover: { type: Boolean, required: false },
    iconChecked: { type: String, required: false },
    iconUnchecked: { type: String, required: false },
    checkedIconColor: { type: String, required: false },
    prependCheckbox: { type: Boolean, required: false },
    "modelValue": {}
  },
  emits: ["update:modelValue"],
  setup(__props: any) {

const modelValue = _useModel(__props, "modelValue")

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("label", _mergeProps(_ctx.$attrs, {
      class: ["common-checkbox flex items-center cursor-pointer py-1 text-md w-full gap-y-1", __props.hover ? 'hover:bg-active ms--2 px-4 py-2' : null],
      onClick: _cache[0] || (_cache[0] = _withModifiers(($event: any) => (modelValue.value = !modelValue.value), ["prevent"]))
    }), [ (__props.label && !__props.prependCheckbox) ? (_openBlock(), _createElementBlock("span", {
          key: 0,
          "flex-1": "",
          "ms-2": "",
          "pointer-events-none": ""
        }, _toDisplayString(__props.label), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createElementVNode("span", {
        class: _normalizeClass([
          modelValue.value ? (__props.iconChecked ?? 'i-ri:checkbox-line') : (__props.iconUnchecked ?? 'i-ri:checkbox-blank-line'),
          modelValue.value && __props.checkedIconColor,
        ]),
        "text-lg": "",
        "aria-hidden": "true"
      }, null, 2 /* CLASS */), _withDirectives(_createElementVNode("input", {
        "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((modelValue).value = $event)),
        type: "checkbox",
        "sr-only": ""
      }, null, 512 /* NEED_PATCH */), [ [_vModelCheckbox, modelValue.value] ]), (__props.label && __props.prependCheckbox) ? (_openBlock(), _createElementBlock("span", {
          key: 0,
          "flex-1": "",
          "ms-2": "",
          "pointer-events-none": ""
        }, _toDisplayString(__props.label), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 16 /* FULL_PROPS */))
}
}

})
