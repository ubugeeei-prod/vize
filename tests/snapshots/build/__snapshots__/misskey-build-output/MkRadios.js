import { useSlots as _useSlots } from 'vue'
import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveDirective as _resolveDirective, renderList as _renderList, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span")

type SlotNames = NonNullable<T extends MkRadiosOption<any, infer U> ? U : never>;

import type { StyleValue } from 'vue';
import type { OptionValue } from '@/types/option-value.js';

export type MkRadiosOption<T = OptionValue, S = string> = {
	value: T;
	slotId?: S;
	label?: string;
	labelStyle?: StyleValue;
	icon?: string;
	iconStyle?: StyleValue;
	caption?: string;
	disabled?: boolean;
};

export default /*@__PURE__*/_defineComponent({
  __name: 'MkRadios',
  props: {
    options: { type: Array, required: true },
    vertical: { type: Boolean, required: false },
    "modelValue": { required: true }
  },
  emits: ["update:modelValue"],
  setup(__props: any) {

const model = _useModel(__props, "modelValue")
function getKey(value: OptionValue): PropertyKey {
	if (value === null) return '___null___';
	return value;
}
function toggle(o: MkRadiosOption): void {
	if (o.disabled) return;
	model.value = o.value;
}

return (_ctx: any,_cache: any) => {
  const _directive_adaptive_border = _resolveDirective("adaptive-border")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass({ [_ctx.$style.vertical]: __props.vertical })
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.label)
      }, [ _renderSlot(_ctx.$slots, "label") ]), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.body)
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.options, (option) => {
          return (_openBlock(), _createElementBlock("div", {
            key: getKey(option.value),
            class: _normalizeClass([_ctx.$style.optionRoot, { [_ctx.$style.disabled]: option.disabled, [_ctx.$style.checked]: model.value === option.value }]),
            "aria-checked": model.value === option.value,
            "aria-disabled": option.disabled,
            role: "checkbox",
            onClick: _cache[0] || (_cache[0] = ($event: any) => (toggle(option)))
          }, [
            _createElementVNode("input", {
              type: "radio",
              disabled: option.disabled,
              class: _normalizeClass(_ctx.$style.optionInput)
            }, null, 8 /* PROPS */, ["disabled"]),
            _createElementVNode("span", {
              class: _normalizeClass(_ctx.$style.optionButton)
            }, [
              _hoisted_1
            ]),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.optionContent)
            }, [
              (option.icon)
                ? (_openBlock(), _createElementBlock("i", {
                  key: 0,
                  class: _normalizeClass([_ctx.$style.optionIcon, option.icon]),
                  style: _normalizeStyle(option.iconStyle)
                }))
                : _createCommentVNode("v-if", true),
              _createElementVNode("div", null, [
                (option.slotId != null)
                  ? (_openBlock(), _createElementBlock("slot", {
                    key: 0,
                    name: `option-${option.slotId}`
                  }))
                  : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                    _createElementVNode("div", {
                      style: _normalizeStyle(option.labelStyle)
                    }, _toDisplayString(option.label ?? option.value), 5 /* TEXT, STYLE */),
                    (option.caption)
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.optionCaption)
                      }, _toDisplayString(option.caption), 1 /* TEXT */))
                      : _createCommentVNode("v-if", true)
                  ], 64 /* STABLE_FRAGMENT */))
              ])
            ])
          ], 10 /* CLASS, PROPS */, ["aria-checked", "aria-disabled"]))
        }), 128 /* KEYED_FRAGMENT */)) ]), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.caption)
      }, [ _renderSlot(_ctx.$slots, "caption") ]) ], 2 /* CLASS */))
}
}

})
