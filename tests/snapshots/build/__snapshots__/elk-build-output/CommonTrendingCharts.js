import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock } from "vue"

import type { mastodon } from 'masto'
import sparkline from '@fnando/sparkline'

export default /*@__PURE__*/_defineComponent({
  __name: 'CommonTrendingCharts',
  props: {
    history: { type: Array, required: false },
    width: { type: Number, required: false, default: 60 },
    height: { type: Number, required: false, default: 40 }
  },
  setup(__props: any) {

const historyNum = computed(() => {
  if (!__props.history)
    return [1, 1, 1, 1, 1, 1, 1]
  return [...__props.history].reverse().map(item => Number(item.accounts) || 0)
})
const sparklineEl = ref<SVGSVGElement>()
const sparklineFn = typeof sparkline !== 'function' ? (sparkline as any).default : sparkline
watch([historyNum, sparklineEl], ([historyNum, sparklineEl]) => {
  if (!sparklineEl)
    return
  sparklineFn(sparklineEl, historyNum)
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("svg", {
      ref_key: "sparklineEl", ref: sparklineEl,
      class: "sparkline",
      width: __props.width,
      height: __props.height,
      "stroke-width": "3"
    }, null, 8 /* PROPS */, ["width", "height"]))
}
}

})
