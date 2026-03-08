import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, vModelText as _vModelText } from "vue"

import { computed } from 'vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkMediaRange',
  props: {
    buffer: { type: Number, required: false, default: undefined },
    sliderBgWhite: { type: Boolean, required: false, default: false },
    "modelValue": { required: true }
  },
  emits: ["dragEnded", "update:modelValue"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const model = _useModel(__props, "modelValue")
const modelValue = computed({
	get: () => typeof model.value === 'number' ? model.value : parseFloat(model.value),
	set: v => { model.value = v; },
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      style: _normalizeStyle(__props.sliderBgWhite ? '--sliderBg: rgba(255,255,255,.25);' : '--sliderBg: var(--MI_THEME-scrollbarHandle);')
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.controlsSeekbar)
      }, [ (__props.buffer !== undefined) ? (_openBlock(), _createElementBlock("progress", {
            key: 0,
            class: _normalizeClass(_ctx.$style.buffer),
            value: isNaN(__props.buffer) ? 0 : __props.buffer,
            min: "0",
            max: "1"
          }, _toDisplayString(Math.round(__props.buffer * 100)) + "% buffered", 1 /* TEXT */)) : _createCommentVNode("v-if", true), _withDirectives(_createElementVNode("input", {
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((model).value = $event)),
          class: _normalizeClass(_ctx.$style.seek),
          style: _normalizeStyle(`--value: ${modelValue.value * 100}%;`),
          type: "range",
          min: "0",
          max: "1",
          step: "any",
          onChange: _cache[1] || (_cache[1] = ($event: any) => (emit('dragEnded', modelValue.value)))
        }, null, 36 /* STYLE, NEED_HYDRATION */), [ [_vModelText, model.value] ]) ]) ], 4 /* STYLE */))
}
}

})
