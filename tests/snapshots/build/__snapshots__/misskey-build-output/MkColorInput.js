import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue";
import { ref, useTemplateRef, toRefs } from "vue";
export default {
  __name: "MkColorInput",
  props: {
    modelValue: {
      type: [String, null],
      required: true
    },
    required: {
      type: Boolean,
      required: false
    },
    readonly: {
      type: Boolean,
      required: false
    },
    disabled: {
      type: Boolean,
      required: false
    }
  },
  emits: ["update:modelValue"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const { modelValue } = toRefs(props);
    const v = ref(modelValue.value);
    const inputEl = useTemplateRef("inputEl");
    const onInput = () => {
      emit("update:modelValue", v.value ?? "");
    };
    return (_ctx, _cache) => {
      const _directive_adaptive_border = _resolveDirective("adaptive-border");
      return _openBlock(), _createElementBlock("div", null, [
        _createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.label) },
          [_renderSlot(_ctx.$slots, "label")],
          2
          /* CLASS */
        ),
        _createElementVNode(
          "div",
          { class: _normalizeClass([_ctx.$style.input, { disabled: __props.disabled }]) },
          [_withDirectives(_createElementVNode("input", {
            ref_key: "inputEl",
            ref: inputEl,
            "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => v.value = $event),
            class: _normalizeClass(_ctx.$style.inputCore),
            type: "color",
            disabled: __props.disabled,
            required: __props.required,
            readonly: __props.readonly,
            onInput
          }, null, 42, [
            "disabled",
            "required",
            "readonly"
          ]), [[_directive_adaptive_border]])],
          2
          /* CLASS */
        ),
        _createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.caption) },
          [_renderSlot(_ctx.$slots, "caption")],
          2
          /* CLASS */
        )
      ]);
    };
  }
};
