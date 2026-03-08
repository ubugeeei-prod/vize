import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle" })
const _hoisted_2 = { style: "margin-left: 8px; opacity: 0.7;" }
import { markRaw, onMounted, onUnmounted, ref, useTemplateRef } from 'vue'
import * as Misskey from 'misskey-js'
import XChart from './federation-job-queue.chart.chart.vue'
import type { ApQueueDomain } from '@/pages/admin/federation-job-queue.vue'
import number from '@/filters/number.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { useStream } from '@/stream.js'
import { i18n } from '@/i18n.js'
import MkFolder from '@/components/MkFolder.vue'
import { genId } from '@/utility/id.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'federation-job-queue.chart',
  props: {
    domain: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const connection = markRaw(useStream().useChannel('queueStats'));
const activeSincePrevTick = ref(0);
const active = ref(0);
const delayed = ref(0);
const waiting = ref(0);
const jobs = ref<Misskey.Endpoints[`admin/queue/${ApQueueDomain}-delayed`]['res']>([]);
const chartProcess = useTemplateRef('chartProcess');
const chartActive = useTemplateRef('chartActive');
const chartDelayed = useTemplateRef('chartDelayed');
const chartWaiting = useTemplateRef('chartWaiting');
function onStats(stats: Misskey.entities.QueueStats) {
	activeSincePrevTick.value = stats[props.domain].activeSincePrevTick;
	active.value = stats[props.domain].active;
	delayed.value = stats[props.domain].delayed;
	waiting.value = stats[props.domain].waiting;
	if (chartProcess.value != null) chartProcess.value.pushData(stats[props.domain].activeSincePrevTick);
	if (chartActive.value != null) chartActive.value.pushData(stats[props.domain].active);
	if (chartDelayed.value != null) chartDelayed.value.pushData(stats[props.domain].delayed);
	if (chartWaiting.value != null) chartWaiting.value.pushData(stats[props.domain].waiting);
}
function onStatsLog(statsLog: Misskey.entities.QueueStatsLog) {
	const dataProcess: Misskey.entities.QueueStats[ApQueueDomain]['activeSincePrevTick'][] = [];
	const dataActive: Misskey.entities.QueueStats[ApQueueDomain]['active'][] = [];
	const dataDelayed: Misskey.entities.QueueStats[ApQueueDomain]['delayed'][] = [];
	const dataWaiting: Misskey.entities.QueueStats[ApQueueDomain]['waiting'][] = [];
	for (const stats of [...statsLog].reverse()) {
		dataProcess.push(stats[props.domain].activeSincePrevTick);
		dataActive.push(stats[props.domain].active);
		dataDelayed.push(stats[props.domain].delayed);
		dataWaiting.push(stats[props.domain].waiting);
	}
	if (chartProcess.value != null) chartProcess.value.setData(dataProcess);
	if (chartActive.value != null) chartActive.value.setData(dataActive);
	if (chartDelayed.value != null) chartDelayed.value.setData(dataDelayed);
	if (chartWaiting.value != null) chartWaiting.value.setData(dataWaiting);
}
onMounted(() => {
	misskeyApi(`admin/queue/${props.domain}-delayed`).then(result => {
		jobs.value = result;
	});
	connection.on('stats', onStats);
	connection.on('statsLog', onStatsLog);
	connection.send('requestLog', {
		id: genId(),
		length: 200,
	});
});
onUnmounted(() => {
	connection.off('stats', onStats);
	connection.off('statsLog', onStatsLog);
	connection.dispose();
});

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.status)
      }, [ _createElementVNode("div", {
          class: _normalizeClass(["_panel", _ctx.$style.statusItem])
        }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.statusLabel)
          }, "Process"), _createTextVNode(_toDisplayString(number(activeSincePrevTick.value)), 1 /* TEXT */) ]), _createElementVNode("div", {
          class: _normalizeClass(["_panel", _ctx.$style.statusItem])
        }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.statusLabel)
          }, "Active"), _createTextVNode(_toDisplayString(number(active.value)), 1 /* TEXT */) ]), _createElementVNode("div", {
          class: _normalizeClass(["_panel", _ctx.$style.statusItem])
        }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.statusLabel)
          }, "Waiting"), _createTextVNode(_toDisplayString(number(waiting.value)), 1 /* TEXT */) ]), _createElementVNode("div", {
          class: _normalizeClass(["_panel", _ctx.$style.statusItem])
        }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.statusLabel)
          }, "Delayed"), _createTextVNode(_toDisplayString(number(delayed.value)), 1 /* TEXT */) ]) ]), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.charts)
      }, [ _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.chart)
        }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.chartTitle)
          }, "Process"), _createVNode(XChart, {
            ref_key: "chartProcess", ref: chartProcess,
            type: "process"
          }, null, 512 /* NEED_PATCH */) ]), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.chart)
        }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.chartTitle)
          }, "Active"), _createVNode(XChart, {
            ref_key: "chartActive", ref: chartActive,
            type: "active"
          }, null, 512 /* NEED_PATCH */) ]), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.chart)
        }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.chartTitle)
          }, "Delayed"), _createVNode(XChart, {
            ref_key: "chartDelayed", ref: chartDelayed,
            type: "delayed"
          }, null, 512 /* NEED_PATCH */) ]), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.chart)
        }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.chartTitle)
          }, "Waiting"), _createVNode(XChart, {
            ref_key: "chartWaiting", ref: chartWaiting,
            type: "waiting"
          }, null, 512 /* NEED_PATCH */) ]) ]), _createVNode(MkFolder, {
        defaultOpen: true,
        "max-height": 250
      }, {
        icon: _withCtx(() => [
          _hoisted_1
        ]),
        label: _withCtx(() => [
          _createTextVNode("Errored instances")
        ]),
        suffix: _withCtx(() => [
          _createTextVNode("(" + _toDisplayString(number(jobs.value.reduce((a, b) => a + b[1], 0))) + " jobs)", 1 /* TEXT */)
        ]),
        default: _withCtx(() => [
          _createElementVNode("div", null, [
            (jobs.value.length > 0)
              ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(jobs.value, (job) => {
                  return (_openBlock(), _createElementBlock("div", { key: job[0] }, [
                    _createVNode(_component_MkA, {
                      to: `/instance-info/${job[0]}`,
                      behavior: "window"
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(job[0]), 1 /* TEXT */)
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 8 /* PROPS */, ["to"]),
                    _createElementVNode("span", _hoisted_2, "(" + _toDisplayString(number(job[1])) + " jobs)", 1 /* TEXT */)
                  ]))
                }), 128 /* KEYED_FRAGMENT */))
              ]))
              : (_openBlock(), _createElementBlock("span", {
                key: 1,
                style: "opacity: 0.5;"
              }, _toDisplayString(_unref(i18n).ts.noJobs), 1 /* TEXT */))
          ])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["defaultOpen", "max-height"]) ]))
}
}

})
