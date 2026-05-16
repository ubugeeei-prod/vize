import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("div", { class: "title" }, "Sub");
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("div", { class: "title" }, "Pub");
import { onMounted, computed, useTemplateRef } from "vue";
import { Chart } from "chart.js";
import MkSelect from "@/components/MkSelect.vue";
import MkChart from "@/components/MkChart.vue";
import { useChartTooltip } from "@/composables/use-chart-tooltip.js";
import { $i } from "@/i.js";
import * as os from "@/os.js";
import { misskeyApiGet } from "@/utility/misskey-api.js";
import { instance } from "@/instance.js";
import { i18n } from "@/i18n.js";
import MkHeatmap from "@/components/MkHeatmap.vue";
import MkFoldableSection from "@/components/MkFoldableSection.vue";
import MkRetentionHeatmap from "@/components/MkRetentionHeatmap.vue";
import MkRetentionLineChart from "@/components/MkRetentionLineChart.vue";
import { initChart } from "@/utility/init-chart.js";
import { useMkSelect } from "@/composables/use-mkselect.js";
const chartLimit = 500;
export default {
  __name: "MkInstanceStats",
  setup(__props) {
    initChart();
    const shouldShowFederation = computed(() => instance.federation !== "none" || $i?.isModerator);
    const { model: chartSpan, def: chartSpanDef } = useMkSelect({
      items: [{
        value: "hour",
        label: i18n.ts.perHour
      }, {
        value: "day",
        label: i18n.ts.perDay
      }],
      initialValue: "hour"
    });
    const { model: chartSrc, def: chartSrcDef } = useMkSelect({
      items: computed(() => {
        const items = [];
        if (shouldShowFederation.value) {
          items.push({
            type: "group",
            label: i18n.ts.federation,
            items: [{
              value: "federation",
              label: i18n.ts._charts.federation
            }, {
              value: "ap-request",
              label: i18n.ts._charts.apRequest
            }]
          });
        }
        items.push({
          type: "group",
          label: i18n.ts.users,
          items: [
            {
              value: "users",
              label: i18n.ts._charts.usersIncDec
            },
            {
              value: "users-total",
              label: i18n.ts._charts.usersTotal
            },
            {
              value: "active-users",
              label: i18n.ts._charts.activeUsers
            }
          ]
        });
        const notesItems = [{
          value: "notes",
          label: i18n.ts._charts.notesIncDec
        }, {
          value: "local-notes",
          label: i18n.ts._charts.localNotesIncDec
        }];
        if (shouldShowFederation.value) notesItems.push({
          value: "remote-notes",
          label: i18n.ts._charts.remoteNotesIncDec
        });
        notesItems.push({
          value: "notes-total",
          label: i18n.ts._charts.notesTotal
        });
        items.push({
          type: "group",
          label: i18n.ts.notes,
          items: notesItems
        });
        items.push({
          type: "group",
          label: i18n.ts.drive,
          items: [{
            value: "drive-files",
            label: i18n.ts._charts.filesIncDec
          }, {
            value: "drive",
            label: i18n.ts._charts.storageUsageIncDec
          }]
        });
        return items;
      }),
      initialValue: "active-users"
    });
    const { model: heatmapSrc, def: heatmapSrcDef } = useMkSelect({
      items: computed(() => [
        {
          value: "active-users",
          label: "Active Users"
        },
        {
          value: "notes",
          label: "Notes"
        },
        ...shouldShowFederation.value ? [
          {
            value: "ap-requests-inbox-received",
            label: "AP Requests: inboxReceived"
          },
          {
            value: "ap-requests-deliver-succeeded",
            label: "AP Requests: deliverSucceeded"
          },
          {
            value: "ap-requests-deliver-failed",
            label: "AP Requests: deliverFailed"
          }
        ] : []
      ]),
      initialValue: "active-users"
    });
    const subDoughnutEl = useTemplateRef("subDoughnutEl");
    const pubDoughnutEl = useTemplateRef("pubDoughnutEl");
    const { handler: externalTooltipHandler1 } = useChartTooltip({ position: "middle" });
    const { handler: externalTooltipHandler2 } = useChartTooltip({ position: "middle" });
    function createDoughnut(chartEl, tooltip, data) {
      const chartInstance = new Chart(chartEl, {
        type: "doughnut",
        data: {
          labels: data.map((x) => x.name),
          datasets: [{
            backgroundColor: data.map((x) => x.color),
            borderColor: getComputedStyle(window.document.documentElement).getPropertyValue("--MI_THEME-panel"),
            borderWidth: 2,
            hoverOffset: 0,
            data: data.map((x) => x.value)
          }]
        },
        options: {
          maintainAspectRatio: false,
          layout: { padding: {
            left: 16,
            right: 16,
            top: 16,
            bottom: 16
          } },
          onClick: (ev) => {
            if (ev.native == null) return;
            const hit = chartInstance.getElementsAtEventForMode(ev.native, "nearest", { intersect: true }, false)[0];
            if (hit != null) {
              data[hit.index].onClick?.();
            }
          },
          plugins: {
            legend: { display: false },
            tooltip: {
              enabled: false,
              mode: "index",
              animation: { duration: 0 },
              external: tooltip
            }
          }
        }
      });
      return chartInstance;
    }
    onMounted(() => {
      misskeyApiGet("federation/stats", { limit: 30 }).then((fedStats) => {
        const subs = fedStats.topSubInstances.map((x) => ({
          name: x.host,
          color: x.themeColor ?? "#888888",
          value: x.followersCount,
          onClick: () => {
            os.pageWindow(`/instance-info/${x.host}`);
          }
        }));
        subs.push({
          name: "(other)",
          color: "#80808080",
          value: fedStats.otherFollowersCount
        });
        if (subDoughnutEl.value != null) createDoughnut(subDoughnutEl.value, externalTooltipHandler1, subs);
        const pubs = fedStats.topPubInstances.map((x) => ({
          name: x.host,
          color: x.themeColor ?? "#888888",
          value: x.followingCount,
          onClick: () => {
            os.pageWindow(`/instance-info/${x.host}`);
          }
        }));
        pubs.push({
          name: "(other)",
          color: "#80808080",
          value: fedStats.otherFollowingCount
        });
        if (pubDoughnutEl.value != null) createDoughnut(pubDoughnutEl.value, externalTooltipHandler2, pubs);
      });
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(_ctx.$style.root) },
        [
          _createVNode(MkFoldableSection, { class: "item" }, {
            header: _withCtx(() => [_createTextVNode("Chart")]),
            default: _withCtx(() => [_createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.chart) },
              [_createElementVNode("div", { class: "selects" }, [_createVNode(MkSelect, {
                items: _unref(chartSrcDef),
                style: "margin: 0; flex: 1;",
                modelValue: _unref(chartSrc),
                "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => chartSrc.value = $event)
              }, null, 8, ["items", "modelValue"]), _createVNode(MkSelect, {
                items: _unref(chartSpanDef),
                style: "margin: 0 0 0 10px;",
                modelValue: _unref(chartSpan),
                "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event) => chartSpan.value = $event)
              }, null, 8, ["items", "modelValue"])]), _createElementVNode("div", { class: "chart _panel" }, [_createVNode(MkChart, {
                src: _unref(chartSrc),
                span: _unref(chartSpan),
                limit: chartLimit,
                detailed: true
              }, null, 8, [
                "src",
                "span",
                "limit",
                "detailed"
              ])])],
              2
              /* CLASS */
            )]),
            _: 1
          }),
          _createVNode(MkFoldableSection, { class: "item" }, {
            header: _withCtx(() => [_createTextVNode("Active users heatmap")]),
            default: _withCtx(() => [_createVNode(MkSelect, {
              items: _unref(heatmapSrcDef),
              style: "margin: 0 0 12px 0;",
              modelValue: _unref(heatmapSrc),
              "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event) => heatmapSrc.value = $event)
            }, null, 8, ["items", "modelValue"]), _createElementVNode(
              "div",
              { class: _normalizeClass(["_panel", _ctx.$style.heatmap]) },
              [_createVNode(MkHeatmap, {
                src: _unref(heatmapSrc),
                label: "Read & Write"
              }, null, 8, ["src", "label"])],
              2
              /* CLASS */
            )]),
            _: 1
          }),
          _createVNode(MkFoldableSection, { class: "item" }, {
            header: _withCtx(() => [_createTextVNode("Retention rate")]),
            default: _withCtx(() => [_createElementVNode(
              "div",
              { class: _normalizeClass(["_panel", _ctx.$style.retentionHeatmap]) },
              [_createVNode(MkRetentionHeatmap)],
              2
              /* CLASS */
            ), _createElementVNode(
              "div",
              { class: _normalizeClass(["_panel", _ctx.$style.retentionLine]) },
              [_createVNode(MkRetentionLineChart)],
              2
              /* CLASS */
            )]),
            _: 1
          }),
          shouldShowFederation.value ? (_openBlock(), _createBlock(MkFoldableSection, {
            key: 0,
            class: "item"
          }, {
            header: _withCtx(() => [_createTextVNode("Federation")]),
            default: _withCtx(() => [_createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.federation) },
              [_createElementVNode("div", { class: "pies" }, [_createElementVNode("div", { class: "sub" }, [_hoisted_1, _createElementVNode(
                "canvas",
                {
                  ref_key: "subDoughnutEl",
                  ref: subDoughnutEl
                },
                null,
                512
                /* NEED_PATCH */
              )]), _createElementVNode("div", { class: "pub" }, [_hoisted_2, _createElementVNode(
                "canvas",
                {
                  ref_key: "pubDoughnutEl",
                  ref: pubDoughnutEl
                },
                null,
                512
                /* NEED_PATCH */
              )])])],
              2
              /* CLASS */
            )]),
            _: 1
          })) : _createCommentVNode("v-if", true)
        ],
        2
        /* CLASS */
      );
    };
  }
};
