import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-http-que", style: "margin-right: 0.5em;" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-http-que" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-player-track-next" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("hr")
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-list-check" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-reload" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-search" })
import { ref, computed, watch } from 'vue'
import * as Misskey from 'misskey-js'
import { debounce } from 'throttle-debounce'
import { useInterval } from '@@/js/use-interval.js'
import XChart from './job-queue.chart.vue'
import XJob from './job-queue.job.vue'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import MkButton from '@/components/MkButton.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import MkTabs from '@/components/MkTabs.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkKeyValue from '@/components/MkKeyValue.vue'
import MkTl from '@/components/MkTl.vue'
import kmg from '@/filters/kmg.js'
import MkInput from '@/components/MkInput.vue'
import bytes from '@/filters/bytes.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'job-queue',
  setup(__props) {

const tab = ref<typeof Misskey.queueTypes[number] | '-'>('-');
const jobState = ref<'all' | 'latest' | 'completed' | 'failed' | 'active' | 'delayed' | 'wait' | 'paused'>('all');
const jobs = ref<Misskey.entities.QueueJob[]>([]);
const jobsFetching = ref(true);
const queueInfos = ref<Misskey.entities.AdminQueueQueuesResponse>([]);
const queueInfo = ref<Misskey.entities.AdminQueueQueueStatsResponse | null>(null);
const searchQuery = ref('');
async function fetchQueues() {
	if (tab.value !== '-') return;
	queueInfos.value = await misskeyApi('admin/queue/queues');
}
async function fetchCurrentQueue() {
	if (tab.value === '-') return;
	queueInfo.value = await misskeyApi('admin/queue/queue-stats', { queue: tab.value });
}
async function fetchJobs() {
	if (tab.value === '-') return;
	jobsFetching.value = true;
	const state = jobState.value;
	jobs.value = await misskeyApi('admin/queue/jobs', {
		queue: tab.value,
		state: state === 'all' ? ['completed', 'failed', 'active', 'delayed', 'wait'] : state === 'latest' ? ['completed', 'failed'] : [state],
		search: searchQuery.value.trim() === '' ? undefined : searchQuery.value,
	}).then((res: Misskey.entities.AdminQueueJobsResponse) => {
		if (state === 'all') {
			res.sort((a, b) => (a.processedOn ?? a.timestamp) > (b.processedOn ?? b.timestamp) ? -1 : 1);
		} else if (state === 'latest') {
			res.sort((a, b) => a.processedOn! > b.processedOn! ? -1 : 1);
		} else if (state === 'delayed') {
			res.sort((a, b) => (a.processedOn ?? a.timestamp) > (b.processedOn ?? b.timestamp) ? -1 : 1);
		}
		return res;
	});
	jobsFetching.value = false;
}
watch([tab], async () => {
	if (tab.value === '-') {
		fetchQueues();
	} else {
		fetchCurrentQueue();
		fetchJobs();
	}
}, { immediate: true });
watch([jobState], () => {
	fetchJobs();
});
const search = debounce(1000, () => {
	fetchJobs();
});
watch([searchQuery], () => {
	search();
});
useInterval(() => {
	if (tab.value === '-') {
		fetchQueues();
	} else {
		fetchCurrentQueue();
	}
}, 1000 * 10, {
	immediate: false,
	afterMounted: true,
});
async function clearQueue() {
	if (tab.value === '-') return;
	const { canceled } = await os.confirm({
		type: 'warning',
		title: i18n.ts.areYouSure,
	});
	if (canceled) return;
	os.apiWithDialog('admin/queue/clear', { queue: tab.value, state: '*' });
	fetchCurrentQueue();
	fetchJobs();
}
async function promoteAllJobs() {
	if (tab.value === '-') return;
	const { canceled } = await os.confirm({
		type: 'warning',
		title: i18n.ts.areYouSure,
	});
	if (canceled) return;
	os.apiWithDialog('admin/queue/promote-jobs', { queue: tab.value });
	fetchCurrentQueue();
	fetchJobs();
}
async function removeJobs() {
	if (tab.value === '-' || jobState.value === 'latest') return;
	const { canceled } = await os.confirm({
		type: 'warning',
		title: i18n.ts.areYouSure,
	});
	if (canceled) return;
	os.apiWithDialog('admin/queue/clear', { queue: tab.value, state: jobState.value === 'all' ? '*' : jobState.value });
	fetchCurrentQueue();
	fetchJobs();
}
async function refreshJob(jobId: string) {
	if (tab.value === '-') return;
	const newJob = await misskeyApi('admin/queue/show-job', { queue: tab.value, jobId });
	const index = jobs.value.findIndex((job) => job.id === jobId);
	if (index !== -1) {
		jobs.value[index] = newJob;
	}
}
const headerActions = computed(() => []);
const headerTabs = computed<{
	key: string;
	title: string;
	icon?: string;
}[]>(() => [{
	key: '-',
	title: i18n.ts.jobQueue,
	icon: 'ti ti-list-check',
}, ...Misskey.queueTypes.map((q) => ({
	key: q,
	title: q,
}))]);
definePage(() => ({
	title: i18n.ts.jobQueue,
	icon: 'ti ti-clock-play',
	needWideArea: true,
}));

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value,
      tab: tab.value,
      "onUpdate:tab": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", { class: "_spacer" }, [
          (tab.value === '-')
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "_gaps"
            }, [
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.queues)
              }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(queueInfos.value, (q) => {
                  return (_openBlock(), _createElementBlock("div", {
                    key: q.name,
                    class: _normalizeClass(_ctx.$style.queue),
                    onClick: _cache[1] || (_cache[1] = ($event: any) => (tab.value = q.name))
                  }, [
                    _createElementVNode("div", { style: "display: flex; align-items: center; font-weight: bold;" }, [
                      _hoisted_1,
                      _createTextVNode(_toDisplayString(q.name), 1 /* TEXT */),
                      (!q.isPaused)
                        ? (_openBlock(), _createElementBlock("i", {
                          key: 0,
                          style: "color: var(--MI_THEME-success); margin-left: auto;",
                          class: "ti ti-player-play"
                        }))
                        : _createCommentVNode("v-if", true)
                    ]),
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.queueCounts)
                    }, [
                      _createVNode(MkKeyValue, null, {
                        key: _withCtx(() => [
                          _createTextVNode("Active")
                        ]),
                        value: _withCtx(() => [
                          _createTextVNode(_toDisplayString(kmg(q.counts.active, 2)), 1 /* TEXT */)
                        ]),
                        _: 2 /* DYNAMIC */
                      }),
                      _createVNode(MkKeyValue, null, {
                        key: _withCtx(() => [
                          _createTextVNode("Delayed")
                        ]),
                        value: _withCtx(() => [
                          _createTextVNode(_toDisplayString(kmg(q.counts.delayed, 2)), 1 /* TEXT */)
                        ]),
                        _: 2 /* DYNAMIC */
                      }),
                      _createVNode(MkKeyValue, null, {
                        key: _withCtx(() => [
                          _createTextVNode("Waiting")
                        ]),
                        value: _withCtx(() => [
                          _createTextVNode(_toDisplayString(kmg(q.counts.waiting, 2)), 1 /* TEXT */)
                        ]),
                        _: 2 /* DYNAMIC */
                      })
                    ]),
                    _createVNode(XChart, { dataSet: { completed: q.metrics.completed.data, failed: q.metrics.failed.data } }, null, 8 /* PROPS */, ["dataSet"])
                  ]))
                }), 128 /* KEYED_FRAGMENT */))
              ])
            ]))
            : (queueInfo.value)
              ? (_openBlock(), _createElementBlock("div", {
                key: 1,
                class: "_gaps"
              }, [
                _createVNode(MkFolder, { defaultOpen: true }, {
                  label: _withCtx(() => [
                    _createTextVNode("Overview: " + _toDisplayString(tab.value), 1 /* TEXT */)
                  ]),
                  icon: _withCtx(() => [
                    _hoisted_2
                  ]),
                  suffix: _withCtx(() => [
                    _createTextVNode("#" + _toDisplayString(queueInfo.value.db.processId) + ":" + _toDisplayString(queueInfo.value.db.port) + " / " + _toDisplayString(queueInfo.value.db.runId), 1 /* TEXT */)
                  ]),
                  caption: _withCtx(() => [
                    _createTextVNode(_toDisplayString(queueInfo.value.qualifiedName), 1 /* TEXT */)
                  ]),
                  footer: _withCtx(() => [
                    _createElementVNode("div", { class: "_buttons" }, [
                      _createVNode(MkButton, {
                        rounded: "",
                        onClick: promoteAllJobs
                      }, {
                        default: _withCtx(() => [
                          _hoisted_3,
                          _createTextVNode(" Promote all jobs")
                        ]),
                        _: 1 /* STABLE */
                      }),
                      _createTextVNode("\n\t\t\t\t\t\t" + "\n\t\t\t\t\t\t" + "\n\t\t\t\t\t\t" + "\n\t\t\t\t\t\t"),
                      _createVNode(MkButton, {
                        rounded: "",
                        danger: "",
                        onClick: clearQueue
                      }, {
                        default: _withCtx(() => [
                          _hoisted_4,
                          _createTextVNode(" Empty queue")
                        ]),
                        _: 1 /* STABLE */
                      })
                    ])
                  ]),
                  default: _withCtx(() => [
                    _createElementVNode("div", { class: "_gaps" }, [
                      _createVNode(XChart, {
                        dataSet: { completed: queueInfo.value.metrics.completed.data, failed: queueInfo.value.metrics.failed.data },
                        aspectRatio: 5
                      }, null, 8 /* PROPS */, ["dataSet", "aspectRatio"]),
                      _createElementVNode("div", { style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 12px;" }, [
                        _createVNode(MkKeyValue, null, {
                          key: _withCtx(() => [
                            _createTextVNode("Active")
                          ]),
                          value: _withCtx(() => [
                            _createTextVNode(_toDisplayString(kmg(queueInfo.value.counts.active, 2)), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }),
                        _createVNode(MkKeyValue, null, {
                          key: _withCtx(() => [
                            _createTextVNode("Delayed")
                          ]),
                          value: _withCtx(() => [
                            _createTextVNode(_toDisplayString(kmg(queueInfo.value.counts.delayed, 2)), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }),
                        _createVNode(MkKeyValue, null, {
                          key: _withCtx(() => [
                            _createTextVNode("Waiting")
                          ]),
                          value: _withCtx(() => [
                            _createTextVNode(_toDisplayString(kmg(queueInfo.value.counts.waiting, 2)), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _hoisted_5,
                      _createElementVNode("div", { style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 12px;" }, [
                        _createVNode(MkKeyValue, null, {
                          key: _withCtx(() => [
                            _createTextVNode("Clients: Connected")
                          ]),
                          value: _withCtx(() => [
                            _createTextVNode(_toDisplayString(queueInfo.value.db.clients.connected), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }),
                        _createVNode(MkKeyValue, null, {
                          key: _withCtx(() => [
                            _createTextVNode("Clients: Blocked")
                          ]),
                          value: _withCtx(() => [
                            _createTextVNode(_toDisplayString(queueInfo.value.db.clients.blocked), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }),
                        _createVNode(MkKeyValue, null, {
                          key: _withCtx(() => [
                            _createTextVNode("Memory: Peak")
                          ]),
                          value: _withCtx(() => [
                            _createTextVNode(_toDisplayString(bytes(queueInfo.value.db.memory.peak, 1)), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }),
                        _createVNode(MkKeyValue, null, {
                          key: _withCtx(() => [
                            _createTextVNode("Memory: Total")
                          ]),
                          value: _withCtx(() => [
                            _createTextVNode(_toDisplayString(bytes(queueInfo.value.db.memory.total, 1)), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }),
                        _createVNode(MkKeyValue, null, {
                          key: _withCtx(() => [
                            _createTextVNode("Memory: Used")
                          ]),
                          value: _withCtx(() => [
                            _createTextVNode(_toDisplayString(bytes(queueInfo.value.db.memory.used, 1)), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }),
                        _createVNode(MkKeyValue, null, {
                          key: _withCtx(() => [
                            _createTextVNode("Uptime")
                          ]),
                          value: _withCtx(() => [
                            _createTextVNode(_toDisplayString(queueInfo.value.db.uptime), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ])
                    ])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["defaultOpen"]),
                _createVNode(MkFolder, {
                  defaultOpen: true,
                  withSpacer: false
                }, {
                  label: _withCtx(() => [
                    _createTextVNode("Jobs: " + _toDisplayString(tab.value), 1 /* TEXT */)
                  ]),
                  icon: _withCtx(() => [
                    _hoisted_6
                  ]),
                  suffix: _withCtx(() => [
                    _createTextVNode("&lt;A:" + _toDisplayString(kmg(queueInfo.value.counts.active, 2)) + "&gt; &lt;D:" + _toDisplayString(kmg(queueInfo.value.counts.delayed, 2)) + "&gt; &lt;W:" + _toDisplayString(kmg(queueInfo.value.counts.waiting, 2)) + "&gt;", 1 /* TEXT */)
                  ]),
                  header: _withCtx(() => [
                    _createVNode(MkTabs, {
                      tabs: [{
  							key: 'all',
  							title: 'All',
  							icon: 'ti ti-code-asterisk',
  						}, {
  							key: 'latest',
  							title: 'Latest',
  							icon: 'ti ti-logs',
  						}, {
  							key: 'completed',
  							title: 'Completed',
  							icon: 'ti ti-check',
  						}, {
  							key: 'failed',
  							title: 'Failed',
  							icon: 'ti ti-circle-x',
  						}, {
  							key: 'active',
  							title: 'Active',
  							icon: 'ti ti-player-play',
  						}, {
  							key: 'delayed',
  							title: 'Delayed',
  							icon: 'ti ti-clock',
  						}, {
  							key: 'wait',
  							title: 'Waiting',
  							icon: 'ti ti-hourglass-high',
  						}, {
  							key: 'paused',
  							title: 'Paused',
  							icon: 'ti ti-player-pause',
  						}],
                      tab: jobState.value,
                      "onUpdate:tab": _cache[2] || (_cache[2] = ($event: any) => ((jobState).value = $event))
                    }, null, 8 /* PROPS */, ["tabs", "tab"])
                  ]),
                  footer: _withCtx(() => [
                    _createElementVNode("div", { class: "_buttons" }, [
                      _createVNode(MkButton, {
                        rounded: "",
                        onClick: _cache[3] || (_cache[3] = ($event: any) => (fetchJobs()))
                      }, {
                        default: _withCtx(() => [
                          _hoisted_7,
                          _createTextVNode(" Refresh view")
                        ]),
                        _: 1 /* STABLE */
                      }),
                      _createVNode(MkButton, {
                        rounded: "",
                        danger: "",
                        style: "margin-left: auto;",
                        onClick: removeJobs
                      }, {
                        default: _withCtx(() => [
                          _hoisted_8,
                          _createTextVNode(" Remove jobs")
                        ]),
                        _: 1 /* STABLE */
                      })
                    ])
                  ]),
                  default: _withCtx(() => [
                    _createElementVNode("div", { class: "_spacer" }, [
                      _createVNode(MkInput, {
                        placeholder: _unref(i18n).ts.search,
                        type: "search",
                        style: "margin-bottom: 16px;",
                        modelValue: searchQuery.value,
                        "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((searchQuery).value = $event))
                      }, {
                        prefix: _withCtx(() => [
                          _hoisted_9
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["placeholder", "modelValue"]),
                      (jobsFetching.value)
                        ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 }))
                        : (_openBlock(), _createBlock(MkTl, {
                          key: 1,
                          events: jobs.value.map((job) => ({
  	id: job.id,
  	timestamp: job.finishedOn ?? job.processedOn ?? job.timestamp,
  	data: job
  })),
                          groupBy: "h",
                          class: "_monospace"
                        }, {
                          right: _withCtx(({ event: job }) => [
                            _createVNode(XJob, {
                              job: job,
                              queueType: tab.value,
                              style: "margin: 4px 0;",
                              onNeedRefresh: _cache[5] || (_cache[5] = ($event: any) => (refreshJob(_ctx.job.id)))
                            }, null, 8 /* PROPS */, ["job", "queueType"])
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["events"]))
                    ])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["defaultOpen", "withSpacer"])
              ]))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs", "tab"]))
}
}

})
