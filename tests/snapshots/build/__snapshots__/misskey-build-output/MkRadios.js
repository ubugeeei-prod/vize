import { useModel as _useModel } from "vue";
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("span");
export {};
export default {
  __name: "MkRadios",
  props: {
    options: {
      type: Array,
      required: true
    },
    vertical: {
      type: Boolean,
      required: false
    },
    "modelValue": { required: true }
  },
  emits: ["update:modelValue"],
  setup(__props) {
    const model = _useModel(__props, "modelValue");
    function getKey(value) {
      if (value === null) return "___null___";
      return value;
    }
    function toggle(o) {
      if (o.disabled) return;
      model.value = o.value;
    }
    return (_ctx, _cache) => {
      const _directive_adaptive_border = _resolveDirective("adaptive-border");
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass({ [_ctx.$style.vertical]: __props.vertical }) },
        [
          _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.label) },
            [_renderSlot(_ctx.$slots, "label")],
            2
            /* CLASS */
          ),
          _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.body) },
            [(_openBlock(true), _createElementBlock(
              _Fragment,
              null,
              _renderList(__props.options, (option) => {
                return _withDirectives((_openBlock(), _createElementBlock("div", {
                  key: getKey(option.value),
                  class: _normalizeClass([_ctx.$style.optionRoot, {
                    [_ctx.$style.disabled]: option.disabled,
                    [_ctx.$style.checked]: model.value === option.value
                  }]),
                  "aria-checked": model.value === option.value,
                  "aria-disabled": option.disabled,
                  role: "checkbox",
                  onClick: ($event) => toggle(option)
                }, [
                  _createElementVNode("input", {
                    type: "radio",
                    disabled: option.disabled,
                    class: _normalizeClass(_ctx.$style.optionInput)
                  }, null, 10, ["disabled"]),
                  _createElementVNode(
                    "span",
                    { class: _normalizeClass(_ctx.$style.optionButton) },
                    [_hoisted_1],
                    2
                    /* CLASS */
                  ),
                  _createElementVNode(
                    "div",
                    { class: _normalizeClass(_ctx.$style.optionContent) },
                    [option.icon ? (_openBlock(), _createElementBlock(
                      "i",
                      {
                        key: 0,
                        class: _normalizeClass([_ctx.$style.optionIcon, option.icon]),
                        style: _normalizeStyle(option.iconStyle)
                      },
                      null,
                      6
                      /* CLASS, STYLE */
                    )) : _createCommentVNode("v-if", true), _createElementVNode("div", null, [option.slotId != null ? _renderSlot(_ctx.$slots, `option-${option.slotId}`, { key: 0 }) : (_openBlock(), _createElementBlock(
                      _Fragment,
                      { key: 1 },
                      [_createElementVNode(
                        "div",
                        { style: _normalizeStyle(option.labelStyle) },
                        _toDisplayString(option.label ?? option.value),
                        5
                        /* TEXT, STYLE */
                      ), option.caption ? (_openBlock(), _createElementBlock(
                        "div",
                        {
                          key: 0,
                          class: _normalizeClass(_ctx.$style.optionCaption)
                        },
                        _toDisplayString(option.caption),
                        3
                        /* TEXT, CLASS */
                      )) : _createCommentVNode("v-if", true)],
                      64
                      /* STABLE_FRAGMENT */
                    ))])],
                    2
                    /* CLASS */
                  )
                ], 10, [
                  "aria-checked",
                  "aria-disabled",
                  "onClick"
                ])), [[_directive_adaptive_border]]);
              }),
              128
              /* KEYED_FRAGMENT */
            ))],
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
        ],
        2
        /* CLASS */
      );
    };
  }
};
