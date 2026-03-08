import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createTextVNode as _createTextVNode, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue"

import { computed } from 'vue'
import number from '@/filters/number.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkNumberDiff',
  props: {
    value: { type: Number, required: true }
  },
  setup(__props: any) {

const props = __props
const isPlus = computed(() => props.value > 0);
const isMinus = computed(() => props.value < 0);
const isZero = computed(() => props.value === 0);

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("span", {
      class: _normalizeClass(["ceaaebcd", { [_ctx.$style.isPlus]: isPlus.value, [_ctx.$style.isMinus]: isMinus.value, [_ctx.$style.isZero]: isZero.value }])
    }, [ _renderSlot(_ctx.$slots, "before"), _createTextVNode(_toDisplayString(isPlus.value ? '+' : '') + _toDisplayString(number(__props.value)), 1 /* TEXT */), _renderSlot(_ctx.$slots, "after") ], 2 /* CLASS */))
}
}

})
