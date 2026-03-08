import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, mergeProps as _mergeProps, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"

import type { SelectBaseProps } from './types'

export interface SelectFieldProps extends SelectBaseProps {
  items: { label: string; value: string; disabled?: boolean }[]
  size?: keyof typeof SELECT_FIELD_SIZES
  selectAttrs?: Omit<SelectBaseProps, 'size' | 'id'> &
    Record<string, string | number | boolean | undefined>
  label?: string
  labelAttrs?: Record<string, string | number | boolean | undefined>
  /** Visually hide label */
  hiddenLabel?: boolean
  id: string
  /** Render select full width */
  block?: boolean
}

export default /*@__PURE__*/_defineComponent({
  __name: 'Field',
  props: {
    items: { type: Array, required: true },
    size: { type: null, required: false, default: 'md' },
    selectAttrs: { type: null, required: false },
    label: { type: String, required: false },
    labelAttrs: { type: null, required: false },
    hiddenLabel: { type: Boolean, required: false },
    id: { type: String, required: true },
    block: { type: Boolean, required: false },
    "modelValue": { default: undefined }
  },
  emits: ["update:modelValue"],
  setup(__props: any) {

const props = __props
const model = _useModel(__props, "modelValue")
const SELECT_FIELD_SIZES = {
  sm: 'text-xs py-1.75 ps-2 pe-6 rounded-md',
  md: 'text-sm py-2.25 ps-3 pe-9 rounded-lg',
  lg: 'text-base py-4 ps-6 pe-15 rounded-xl',
}
const SELECT_FIELD_ICON_SIZES = {
  sm: 'inset-ie-2 size-[0.75rem]',
  md: 'inset-ie-3 size-[1rem]',
  lg: 'inset-ie-5 size-[1.5rem]',
}
const SELECT_FIELD_LABEL_SIZES = {
  sm: 'text-2xs',
  md: 'text-xs',
  lg: 'text-sm',
}

return (_ctx: any,_cache: any) => {
  const _component_SelectBase = _resolveComponent("SelectBase")

  return (_openBlock(), _createElementBlock("div", { class: "group/select" }, [ (__props.label) ? (_openBlock(), _createElementBlock("label", _mergeProps(__props.labelAttrs, {
          key: 0,
          for: __props.id,
          class: _normalizeClass(["block mb-1 font-mono text-fg-subtle tracking-wide uppercase", [__props.hiddenLabel ? 'sr-only' : '', SELECT_FIELD_LABEL_SIZES[__props.size]]])
        }), _toDisplayString(__props.label), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createElementVNode("div", {
        class: _normalizeClass(["relative", [__props.block ? 'w-full' : 'w-fit']])
      }, [ _createVNode(_component_SelectBase, _mergeProps(__props.selectAttrs, {
          disabled: _ctx.disabled,
          size: "none",
          class: ["appearance-none group-hover/select:border-fg-muted", [SELECT_FIELD_SIZES[__props.size], __props.block ? 'w-full' : 'w-fit']],
          id: __props.id,
          modelValue: model.value,
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((model).value = $event))
        }), {
          default: _withCtx(() => [
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.items, (item) => {
              return (_openBlock(), _createElementBlock("option", {
                key: item.value,
                value: item.value,
                disabled: item.disabled
              }, _toDisplayString(item.label), 9 /* TEXT, PROPS */, ["value", "disabled"]))
            }), 128 /* KEYED_FRAGMENT */))
          ]),
          _: 1 /* STABLE */
        }, 16 /* FULL_PROPS */, ["disabled", "id", "modelValue"]), _createElementVNode("span", {
          "aria-hidden": "true",
          class: _normalizeClass(["block i-lucide:chevron-down absolute top-1/2 -translate-y-1/2 text-fg-subtle pointer-events-none group-hover/select:text-fg group-focus-within/select:text-fg", [SELECT_FIELD_ICON_SIZES[__props.size]]])
        }, null, 2 /* CLASS */) ], 2 /* CLASS */) ]))
}
}

})
