import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderSlot as _renderSlot, normalizeClass as _normalizeClass, vModelText as _vModelText } from "vue"

import { ref, useTemplateRef, toRefs } from 'vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkColorInput',
  props: {
    modelValue: { type: String, required: true },
    required: { type: Boolean, required: false },
    readonly: { type: Boolean, required: false },
    disabled: { type: Boolean, required: false }
  },
  emits: ["update:modelValue"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const { modelValue } = toRefs(props);
const v = ref(modelValue.value);
const inputEl = useTemplateRef('inputEl');
const onInput = () => {
	emit('update:modelValue', v.value ?? '');
};

return (_ctx: any,_cache: any) => {
  const _directive_adaptive_border = _resolveDirective("adaptive-border")

  return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.label)
      }, [ _renderSlot(_ctx.$slots, "label") ]), _createElementVNode("div", {
        class: _normalizeClass([_ctx.$style.input, { disabled: __props.disabled }])
      }, [ _withDirectives(_createElementVNode("input", {
          ref_key: "inputEl", ref: inputEl,
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((v).value = $event)),
          class: _normalizeClass(_ctx.$style.inputCore),
          type: "color",
          disabled: __props.disabled,
          required: __props.required,
          readonly: __props.readonly,
          onInput: onInput
        }, null, 40 /* PROPS, NEED_HYDRATION */, ["disabled", "required", "readonly"]), [ [_vModelText, v.value] ]) ], 2 /* CLASS */), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.caption)
      }, [ _renderSlot(_ctx.$slots, "caption") ]) ]))
}
}

})
