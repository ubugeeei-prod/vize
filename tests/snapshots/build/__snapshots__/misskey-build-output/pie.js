import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue";
import { computed } from "vue";
const r = .45;
export default {
  __name: "pie",
  props: { value: {
    type: Number,
    required: true
  } },
  setup(__props) {
    const props = __props;
    const color = computed(() => `hsl(${180 - props.value * 180}, 80%, 70%)`);
    const strokeDashoffset = computed(() => (1 - props.value) * (Math.PI * (r * 2)));
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "svg",
        {
          class: _normalizeClass(_ctx.$style.root),
          viewBox: "0 0 1 1",
          preserveAspectRatio: "none"
        },
        [
          _createElementVNode("circle", {
            r,
            cx: "50%",
            cy: "50%",
            fill: "none",
            "stroke-width": "0.1",
            stroke: "rgba(0, 0, 0, 0.05)",
            class: _normalizeClass(_ctx.$style.circle)
          }, null, 10, ["r"]),
          _createElementVNode("circle", {
            r,
            cx: "50%",
            cy: "50%",
            "stroke-dasharray": Math.PI * (r * 2),
            "stroke-dashoffset": strokeDashoffset.value,
            fill: "none",
            "stroke-width": "0.1",
            class: _normalizeClass(_ctx.$style.circle),
            stroke: color.value
          }, null, 10, [
            "r",
            "stroke-dasharray",
            "stroke-dashoffset",
            "stroke"
          ]),
          _createElementVNode(
            "text",
            {
              x: "50%",
              y: "50%",
              dy: "0.05",
              "text-anchor": "middle",
              class: _normalizeClass(_ctx.$style.text)
            },
            _toDisplayString((__props.value * 100).toFixed(0)) + "%",
            3
            /* TEXT, CLASS */
          )
        ],
        2
        /* CLASS */
      );
    };
  }
};
