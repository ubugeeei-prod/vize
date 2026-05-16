import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-cpu" });
import { onMounted, onBeforeUnmount, ref } from "vue";
import XPie from "./pie.vue";
export default {
  __name: "cpu",
  props: {
    connection: {
      type: null,
      required: true
    },
    meta: {
      type: null,
      required: true
    }
  },
  setup(__props) {
    const props = __props;
    const usage = ref(0);
    function onStats(stats) {
      usage.value = stats.cpu;
    }
    onMounted(() => {
      props.connection.on("stats", onStats);
    });
    onBeforeUnmount(() => {
      props.connection.off("stats", onStats);
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock("div", { class: "vrvdvrys" }, [_createVNode(XPie, {
        class: "pie",
        value: usage.value
      }, null, 8, ["value"]), _createElementVNode("div", null, [
        _createElementVNode("p", null, [_hoisted_1, _createTextVNode("CPU")]),
        _createElementVNode(
          "p",
          null,
          _toDisplayString(__props.meta.cpu.cores) + " Logical cores",
          1
          /* TEXT */
        ),
        _createElementVNode(
          "p",
          null,
          _toDisplayString(__props.meta.cpu.model),
          1
          /* TEXT */
        )
      ])]);
    };
  }
};
