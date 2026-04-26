import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-section" });
import { onMounted, onBeforeUnmount, ref } from "vue";
import XPie from "./pie.vue";
import bytes from "@/filters/bytes.js";
export default {
  __name: "mem",
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
    const total = ref(0);
    const used = ref(0);
    const free = ref(0);
    function onStats(stats) {
      usage.value = stats.mem.active / props.meta.mem.total;
      total.value = props.meta.mem.total;
      used.value = stats.mem.active;
      free.value = total.value - used.value;
    }
    onMounted(() => {
      props.connection.on("stats", onStats);
    });
    onBeforeUnmount(() => {
      props.connection.off("stats", onStats);
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock("div", { class: "zlxnikvl" }, [_createVNode(XPie, {
        class: "pie",
        value: usage.value
      }, null, 8, ["value"]), _createElementVNode("div", null, [
        _createElementVNode("p", null, [_hoisted_1, _createTextVNode("RAM")]),
        _createElementVNode(
          "p",
          null,
          "Total: " + _toDisplayString(bytes(total.value, 1)),
          1
          /* TEXT */
        ),
        _createElementVNode(
          "p",
          null,
          "Used: " + _toDisplayString(bytes(used.value, 1)),
          1
          /* TEXT */
        ),
        _createElementVNode(
          "p",
          null,
          "Free: " + _toDisplayString(bytes(free.value, 1)),
          1
          /* TEXT */
        )
      ])]);
    };
  }
};
