import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("div", { class: "_label" }, "Process");
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("div", { class: "_label" }, "Active");
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("div", { class: "_label" }, "Waiting");
const _hoisted_4 = /* @__PURE__ */ _createElementVNode("div", { class: "_label" }, "Delayed");
const _hoisted_5 = /* @__PURE__ */ _createElementVNode("div", { class: "title" }, "Process");
const _hoisted_6 = /* @__PURE__ */ _createElementVNode("div", { class: "title" }, "Active");
const _hoisted_7 = /* @__PURE__ */ _createElementVNode("div", { class: "title" }, "Delayed");
const _hoisted_8 = /* @__PURE__ */ _createElementVNode("div", { class: "title" }, "Waiting");
import { markRaw, onMounted, onUnmounted, ref, useTemplateRef } from "vue";
import XChart from "./overview.queue.chart.vue";
import number from "@/filters/number.js";
import { useStream } from "@/stream.js";
import { genId } from "@/utility/id.js";
export default {
  __name: "overview.queue",
  props: { domain: {
    type: null,
    required: true
  } },
  setup(__props) {
    const props = __props;
    const connection = markRaw(useStream().useChannel("queueStats"));
    const activeSincePrevTick = ref(0);
    const active = ref(0);
    const delayed = ref(0);
    const waiting = ref(0);
    const chartProcess = useTemplateRef("chartProcess");
    const chartActive = useTemplateRef("chartActive");
    const chartDelayed = useTemplateRef("chartDelayed");
    const chartWaiting = useTemplateRef("chartWaiting");
    function onStats(stats) {
      activeSincePrevTick.value = stats[props.domain].activeSincePrevTick;
      active.value = stats[props.domain].active;
      delayed.value = stats[props.domain].delayed;
      waiting.value = stats[props.domain].waiting;
      chartProcess.value?.pushData(stats[props.domain].activeSincePrevTick);
      chartActive.value?.pushData(stats[props.domain].active);
      chartDelayed.value?.pushData(stats[props.domain].delayed);
      chartWaiting.value?.pushData(stats[props.domain].waiting);
    }
    function onStatsLog(statsLog) {
      const dataProcess = [];
      const dataActive = [];
      const dataDelayed = [];
      const dataWaiting = [];
      for (const stats of [...statsLog].reverse()) {
        dataProcess.push(stats[props.domain].activeSincePrevTick);
        dataActive.push(stats[props.domain].active);
        dataDelayed.push(stats[props.domain].delayed);
        dataWaiting.push(stats[props.domain].waiting);
      }
      chartProcess.value?.setData(dataProcess);
      chartActive.value?.setData(dataActive);
      chartDelayed.value?.setData(dataDelayed);
      chartWaiting.value?.setData(dataWaiting);
    }
    onMounted(() => {
      connection.on("stats", onStats);
      connection.on("statsLog", onStatsLog);
      connection.send("requestLog", {
        id: genId(),
        length: 100
      });
    });
    onUnmounted(() => {
      connection.off("stats", onStats);
      connection.off("statsLog", onStatsLog);
      connection.dispose();
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(_ctx.$style.root) },
        [_createElementVNode("div", { class: "_table status" }, [_createElementVNode("div", { class: "_row" }, [
          _createElementVNode("div", {
            class: "_cell",
            style: "text-align: center;"
          }, [_hoisted_1, _createTextVNode(
            _toDisplayString(number(activeSincePrevTick.value)),
            1
            /* TEXT */
          )]),
          _createElementVNode("div", {
            class: "_cell",
            style: "text-align: center;"
          }, [_hoisted_2, _createTextVNode(
            _toDisplayString(number(active.value)),
            1
            /* TEXT */
          )]),
          _createElementVNode("div", {
            class: "_cell",
            style: "text-align: center;"
          }, [_hoisted_3, _createTextVNode(
            _toDisplayString(number(waiting.value)),
            1
            /* TEXT */
          )]),
          _createElementVNode("div", {
            class: "_cell",
            style: "text-align: center;"
          }, [_hoisted_4, _createTextVNode(
            _toDisplayString(number(delayed.value)),
            1
            /* TEXT */
          )])
        ])]), _createElementVNode("div", { class: "charts" }, [
          _createElementVNode("div", { class: "chart" }, [_hoisted_5, _createVNode(
            XChart,
            {
              ref_key: "chartProcess",
              ref: chartProcess,
              type: "process"
            },
            null,
            512
            /* NEED_PATCH */
          )]),
          _createElementVNode("div", { class: "chart" }, [_hoisted_6, _createVNode(
            XChart,
            {
              ref_key: "chartActive",
              ref: chartActive,
              type: "active"
            },
            null,
            512
            /* NEED_PATCH */
          )]),
          _createElementVNode("div", { class: "chart" }, [_hoisted_7, _createVNode(
            XChart,
            {
              ref_key: "chartDelayed",
              ref: chartDelayed,
              type: "delayed"
            },
            null,
            512
            /* NEED_PATCH */
          )]),
          _createElementVNode("div", { class: "chart" }, [_hoisted_8, _createVNode(
            XChart,
            {
              ref_key: "chartWaiting",
              ref: chartWaiting,
              type: "waiting"
            },
            null,
            512
            /* NEED_PATCH */
          )])
        ])],
        2
        /* CLASS */
      );
    };
  }
};
