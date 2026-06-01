import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderSlot as _renderSlot, normalizeClass as _normalizeClass, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-help-circle" });
import { toRefs } from "vue";
import XButton from "@/components/MkSwitch.button.vue";
import { haptic } from "@/utility/haptic.js";
export default {
  __name: "MkSwitch",
  props: {
    modelValue: {
      type: [Boolean, Object],
      required: true
    },
    disabled: {
      type: Boolean,
      required: false
    },
    helpText: {
      type: String,
      required: false
    },
    noBody: {
      type: Boolean,
      required: false
    }
  },
  emits: ["update:modelValue", "change"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const checked = toRefs(props).modelValue;
    const toggle = () => {
      if (props.disabled) return;
      emit("update:modelValue", !checked.value);
      emit("change", !checked.value);
      haptic();
    };
    return (_ctx, _cache) => {
      const _directive_tooltip = _resolveDirective("tooltip");
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.disabled]: __props.disabled }]) },
        [
          _createElementVNode("input", {
            ref: "input",
            type: "checkbox",
            disabled: __props.disabled,
            class: _normalizeClass(_ctx.$style.input),
            onClick: toggle
          }, null, 10, ["disabled"]),
          _createVNode(XButton, {
            class: _normalizeClass(_ctx.$style.toggle),
            checked: _unref(checked),
            disabled: __props.disabled,
            onToggle: toggle
          }, null, 10, ["checked", "disabled"]),
          !__props.noBody ? (_openBlock(), _createElementBlock(
            "span",
            {
              key: 0,
              class: _normalizeClass(_ctx.$style.body)
            },
            [_createElementVNode(
              "span",
              { class: _normalizeClass(_ctx.$style.label) },
              [_createElementVNode("span", { onClick: toggle }, [_renderSlot(_ctx.$slots, "label"), _renderSlot(_ctx.$slots, "default")]), __props.helpText ? _withDirectives((_openBlock(), _createElementBlock(
                "span",
                {
                  key: 0,
                  class: _normalizeClass(["_button _help", _ctx.$style.help])
                },
                [_hoisted_1],
                2
                /* CLASS */
              )), [[
                _directive_tooltip,
                __props.helpText,
                "dialog"
              ]]) : _createCommentVNode("v-if", true)],
              2
              /* CLASS */
            ), _createElementVNode(
              "p",
              { class: _normalizeClass(_ctx.$style.caption) },
              [_renderSlot(_ctx.$slots, "caption")],
              2
              /* CLASS */
            )],
            2
            /* CLASS */
          )) : _createCommentVNode("v-if", true)
        ],
        2
        /* CLASS */
      );
    };
  }
};
