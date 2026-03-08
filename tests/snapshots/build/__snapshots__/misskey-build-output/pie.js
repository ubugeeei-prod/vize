import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue"

import { computed } from 'vue'
const r = 0.45;

export default /*@__PURE__*/_defineComponent({
  __name: 'pie',
  props: {
    value: { type: Number, required: true }
  },
  setup(__props: any) {

const props = __props
const color = computed(() => `hsl(${180 - (props.value * 180)}, 80%, 70%)`);
const strokeDashoffset = computed(() => (1 - props.value) * (Math.PI * (r * 2)));

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("svg", {
      class: _normalizeClass(_ctx.$style.root),
      viewBox: "0 0 1 1",
      preserveAspectRatio: "none"
    }, [ _createElementVNode("circle", {
        r: r,
        cx: "50%",
        cy: "50%",
        fill: "none",
        "stroke-width": "0.1",
        stroke: "rgba(0, 0, 0, 0.05)",
        class: _normalizeClass(_ctx.$style.circle)
      }, null, 8 /* PROPS */, ["r"]), _createElementVNode("circle", {
        r: r,
        cx: "50%",
        cy: "50%",
        "stroke-dasharray": Math.PI * (r * 2),
        "stroke-dashoffset": strokeDashoffset.value,
        fill: "none",
        "stroke-width": "0.1",
        class: _normalizeClass(_ctx.$style.circle),
        stroke: color.value
      }, null, 8 /* PROPS */, ["r", "stroke-dasharray", "stroke-dashoffset", "stroke"]), _createElementVNode("text", {
        x: "50%",
        y: "50%",
        dy: "0.05",
        "text-anchor": "middle",
        class: _normalizeClass(_ctx.$style.text)
      }, _toDisplayString((__props.value * 100).toFixed(0)) + "%", 1 /* TEXT */) ]))
}
}

})
