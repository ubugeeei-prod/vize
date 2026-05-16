import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, vModelCheckbox as _vModelCheckbox } from "vue"


const _hoisted_1 = { class: "text-sm text-fg font-medium text-start", style: "grid-area: label-text" }
const _hoisted_2 = { class: "text-sm text-fg font-medium text-start", style: "grid-area: label-text" }
import TooltipApp from '~/components/Tooltip/App.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'Toggle.client',
  props: {
    label: { type: String, required: true },
    description: { type: String, required: false },
    justify: { type: String, required: false, default: 'between' },
    tooltip: { type: String, required: false },
    tooltipPosition: { type: String, required: false },
    tooltipTo: { type: String, required: false },
    tooltipOffset: { type: Number, required: false },
    reverseOrder: { type: Boolean, required: false, default: false },
    "modelValue": {
  required: true,
}
  },
  emits: ["update:modelValue"],
  setup(__props: any) {

const props = __props
const checked = _useModel(__props, "modelValue")
const id = useId()

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createElementVNode("label", {
        for: _unref(id),
        class: _normalizeClass(["grid items-center gap-1.5 py-1 -my-1 grid-cols-[auto_1fr_auto]", [__props.justify === 'start' ? 'justify-start' : '']]),
        style: _normalizeStyle(
        props.reverseOrder
          ? 'grid-template-areas: \'toggle . label-text\''
          : 'grid-template-areas: \'label-text . toggle\''
      )
      }, [ (props.reverseOrder) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _withDirectives(_createElementVNode("input", {
              role: "switch",
              type: "checkbox",
              id: _unref(id),
              "onUpdate:modelValue": [($event: any) => ((checked).value = $event), ($event: any) => ((checked).value = $event), ($event: any) => ((checked).value = $event)],
              class: "toggle appearance-none h-6 w-11 rounded-full border border-fg relative shrink-0 bg-fg-subtle checked:bg-fg checked:border-fg focus-visible:(outline-2 outline-fg outline-offset-2) before:content-[''] before:absolute before:h-5 before:w-5 before:top-1px before:rounded-full before:bg-bg",
              style: "grid-area: toggle"
            }, null, 8 /* PROPS */, ["id"]), [ [_vModelCheckbox, checked.value] ]), (__props.tooltip && __props.label) ? (_openBlock(), _createBlock(TooltipApp, {
                key: 0,
                text: __props.tooltip,
                position: __props.tooltipPosition ?? 'top',
                to: __props.tooltipTo,
                offset: __props.tooltipOffset
              }, {
                default: _withCtx(() => [
                  _createElementVNode("span", _hoisted_1, _toDisplayString(__props.label), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["text", "position", "to", "offset"])) : (__props.label) ? (_openBlock(), _createElementBlock("span", {
                  key: 1,
                  class: "text-sm text-fg font-medium text-start",
                  style: "grid-area: label-text"
                }, _toDisplayString(__props.label), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ (__props.tooltip && __props.label) ? (_openBlock(), _createBlock(TooltipApp, {
                key: 0,
                text: __props.tooltip,
                position: __props.tooltipPosition ?? 'top',
                to: __props.tooltipTo,
                offset: __props.tooltipOffset
              }, {
                default: _withCtx(() => [
                  _createElementVNode("span", _hoisted_2, _toDisplayString(__props.label), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["text", "position", "to", "offset"])) : (__props.label) ? (_openBlock(), _createElementBlock("span", {
                  key: 1,
                  class: "text-sm text-fg font-medium text-start",
                  style: "grid-area: label-text"
                }, _toDisplayString(__props.label), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _withDirectives(_createElementVNode("input", {
              role: "switch",
              type: "checkbox",
              id: _unref(id),
              "onUpdate:modelValue": [($event: any) => ((checked).value = $event), ($event: any) => ((checked).value = $event)],
              class: "toggle appearance-none h-6 w-11 rounded-full border border-fg relative shrink-0 bg-fg-subtle checked:bg-fg checked:border-fg focus-visible:(outline-2 outline-fg outline-offset-2) before:content-[''] before:absolute before:h-5 before:w-5 before:top-1px before:rounded-full before:bg-bg",
              style: "grid-area: toggle; justify-self: end"
            }, null, 8 /* PROPS */, ["id"]), [ [_vModelCheckbox, checked.value] ]) ], 64 /* STABLE_FRAGMENT */)) ], 14 /* CLASS, STYLE, PROPS */, ["for"]), (__props.description) ? (_openBlock(), _createElementBlock("p", {
          key: 0,
          class: "text-sm text-fg-muted mt-2"
        }, _toDisplayString(__props.description), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */))
}
}

})
