import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, normalizeClass as _normalizeClass, vShow as _vShow } from "vue";
import { onMounted, useTemplateRef, ref } from "vue";
import { Chart } from "chart.js";
import gradient from "chartjs-plugin-gradient";
import { misskeyApi } from "@/utility/misskey-api.js";
import { store } from "@/store.js";
import { useChartTooltip } from "@/composables/use-chart-tooltip.js";
import { chartVLine } from "@/utility/chart-vline.js";
import { initChart } from "@/utility/init-chart.js";
import { chartLegend } from "@/utility/chart-legend.js";
import MkChartLegend from "@/components/MkChartLegend.vue";
const chartLimit = 50;
export default {
  __name: "activity.notes",
  props: { user: {
    type: null,
    required: true
  } },
  setup(__props) {
    const props = __props;
    initChart();
    const chartEl = useTemplateRef("chartEl");
    const legendEl = useTemplateRef("legendEl");
    const now = new Date();
    let chartInstance = null;
    const fetching = ref(true);
    const { handler: externalTooltipHandler } = useChartTooltip();
    async function renderChart() {
      if (chartEl.value == null) return;
      if (chartInstance) {
        chartInstance.destroy();
      }
      const getDate = (ago) => {
        const y = now.getFullYear();
        const m = now.getMonth();
        const d = now.getDate();
        return new Date(y, m, d - ago);
      };
      const format = (arr) => {
        return arr.map((v, i) => ({
          x: getDate(i).getTime(),
          y: v
        }));
      };
      const raw = await misskeyApi("charts/user/notes", {
        userId: props.user.id,
        limit: chartLimit,
        span: "day"
      });
      const vLineColor = store.s.darkMode ? "rgba(255, 255, 255, 0.2)" : "rgba(0, 0, 0, 0.2)";
      const colorNormal = "#008FFB";
      const colorReply = "#FEB019";
      const colorRenote = "#00E396";
      const colorFile = "#e300db";
      function makeDataset(label, data, extra = {}) {
        return Object.assign({
          label,
          data,
          parsing: false,
          pointRadius: 0,
          borderWidth: 0,
          borderJoinStyle: "round",
          borderRadius: 4,
          barPercentage: .9,
          fill: true
        }, extra);
      }
      chartInstance = new Chart(chartEl.value, {
        type: "bar",
        data: { datasets: [
          makeDataset("File", format(raw.diffs.withFile).slice().reverse(), { backgroundColor: colorFile }),
          makeDataset("Renote", format(raw.diffs.renote).slice().reverse(), { backgroundColor: colorRenote }),
          makeDataset("Reply", format(raw.diffs.reply).slice().reverse(), { backgroundColor: colorReply }),
          makeDataset("Normal", format(raw.diffs.normal).slice().reverse(), { backgroundColor: colorNormal })
        ] },
        options: {
          aspectRatio: 3,
          layout: { padding: {
            left: 0,
            right: 8,
            top: 0,
            bottom: 0
          } },
          scales: {
            x: {
              type: "time",
              offset: true,
              stacked: true,
              time: {
                unit: "day",
                displayFormats: {
                  day: "M/d",
                  month: "Y/M"
                }
              },
              grid: { display: false },
              ticks: {
                display: true,
                maxRotation: 0,
                autoSkipPadding: 8
              }
            },
            y: {
              position: "left",
              stacked: true,
              suggestedMax: 10,
              grid: { display: true },
              ticks: { display: true }
            }
          },
          interaction: {
            intersect: false,
            mode: "index"
          },
          plugins: {
            legend: { display: false },
            tooltip: {
              enabled: false,
              mode: "index",
              animation: { duration: 0 },
              external: externalTooltipHandler
            },
            ...{ gradient }
          }
        },
        plugins: [chartVLine(vLineColor), chartLegend(legendEl.value)]
      });
      fetching.value = false;
    }
    onMounted(async () => {
      renderChart();
    });
    return (_ctx, _cache) => {
      const _component_MkLoading = _resolveComponent("MkLoading");
      return _openBlock(), _createElementBlock("div", null, [fetching.value ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 })) : _createCommentVNode("v-if", true), _withDirectives(_createElementVNode(
        "div",
        { class: _normalizeClass(["_panel", _ctx.$style.root]) },
        [_createElementVNode(
          "canvas",
          {
            ref_key: "chartEl",
            ref: chartEl
          },
          null,
          512
          /* NEED_PATCH */
        ), _createVNode(
          MkChartLegend,
          {
            ref_key: "legendEl",
            ref: legendEl,
            style: "margin-top: 8px;"
          },
          null,
          512
          /* NEED_PATCH */
        )],
        2
        /* CLASS */
      ), [[_vShow, !fetching.value]])]);
    };
  }
};
