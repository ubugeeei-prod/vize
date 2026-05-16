import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode } from "vue"

import TrendsChart from '../Package/TrendsChart.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'LineChart',
  props: {
    packages: { type: Array, required: true }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "font-mono" }, [ _createVNode(TrendsChart, {
        "package-names": __props.packages,
        "in-modal": false,
        "show-facet-selector": ""
      }, null, 8 /* PROPS */, ["package-names", "in-modal"]) ]))
}
}

})
