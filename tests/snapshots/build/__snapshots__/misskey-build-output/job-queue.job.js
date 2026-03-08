import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-circle-x" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-clock" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-player-play" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-copy" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-reload" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-player-track-next" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { style: "color: var(--MI_THEME-error)", class: "ti ti-alert-triangle" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("b", null, "Finished")
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-circle-x", style: "color: var(--MI_THEME-error);" })
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("b", null, "Finished")
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check", style: "color: var(--MI_THEME-success);" })
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("b", null, "Processed")
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-player-play" })
const _hoisted_17 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle", style: "color: var(--MI_THEME-warn);" })
const _hoisted_18 = /*#__PURE__*/ _createElementVNode("b", null, "Created")
const _hoisted_19 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_20 = /*#__PURE__*/ _createElementVNode("div", null, "at ?")
const _hoisted_21 = { style: "font-size: 90%; opacity: 0.7;" }
const _hoisted_22 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-floppy" })
const _hoisted_23 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-refresh" })
import { ref, computed } from 'vue'
import * as Misskey from 'misskey-js'
import JSON5 from 'json5'
import type { TlEvent } from '@/components/MkTl.vue'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import MkButton from '@/components/MkButton.vue'
import MkTabs from '@/components/MkTabs.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkCode from '@/components/MkCode.vue'
import MkKeyValue from '@/components/MkKeyValue.vue'
import MkCodeEditor from '@/components/MkCodeEditor.vue'
import MkTl from '@/components/MkTl.vue'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'

type TlType = TlEvent<{
	type: 'created' | 'processed' | 'finished';
} | {
	type: 'attempt';
	attempt: number;
}>;
const canEdit = true;

export default /*@__PURE__*/_defineComponent({
  __name: 'job-queue.job',
  props: {
    job: { type: null, required: true },
    queueType: { type: null, required: true }
  },
  emits: ["needRefresh"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
function msSMH(v: number | null) {
	if (v == null) return 'N/A';
	if (v === 0) return '0';
	const suffixes = ['ms', 's', 'm', 'h'];
	const isMinus = v < 0;
	if (isMinus) v = -v;
	const i = Math.floor(Math.log(v) / Math.log(1000));
	const value = v / Math.pow(1000, i);
	const suffix = suffixes[i];
	return `${isMinus ? '-' : ''}${value.toFixed(1)}${suffix}`;
}
const tab = ref('info');
const editData = ref(JSON5.stringify(props.job.data, null, '\t'));
const logs = ref<string[]>([]);
const timeline = computed(() => {
	const events: TlType[] = [{
		id: 'created',
		timestamp: props.job.timestamp,
		data: {
			type: 'created',
		},
	}];

	if (props.job.attempts > 1) {
		for (let i = 1; i < props.job.attempts; i++) {
			events.push({
				id: `attempt-${i}`,
				timestamp: props.job.timestamp + i,
				data: {
					type: 'attempt',
					attempt: i,
				},
			});
		}
	}
	if (props.job.processedOn != null) {
		events.push({
			id: 'processed',
			timestamp: props.job.processedOn,
			data: {
				type: 'processed',
			},
		});
	}
	if (props.job.finishedOn != null) {
		events.push({
			id: 'finished',
			timestamp: props.job.finishedOn,
			data: {
				type: 'finished',
			},
		});
	}
	return events;
});
async function promoteJob() {
	const { canceled } = await os.confirm({
		type: 'warning',
		title: i18n.ts.areYouSure,
	});
	if (canceled) return;
	os.apiWithDialog('admin/queue/retry-job', { queue: props.queueType, jobId: props.job.id });
}
async function removeJob() {
	const { canceled } = await os.confirm({
		type: 'warning',
		title: i18n.ts.areYouSure,
	});
	if (canceled) return;
	os.apiWithDialog('admin/queue/remove-job', { queue: props.queueType, jobId: props.job.id });
}
async function loadLogs() {
	logs.value = await os.apiWithDialog('admin/queue/show-job-logs', { queue: props.queueType, jobId: props.job.id });
}
// TODO
// function moveJob() {
//
// }
function refresh() {
	emit('needRefresh');
}
function copyRaw() {
	const raw = JSON.stringify(props.job, null, '\t');
	copyToClipboard(raw);
}

return (_ctx: any,_cache: any) => {
  const _component_MkTime = _resolveComponent("MkTime")

  return (_openBlock(), _createBlock(MkFolder, null, {
      label: _withCtx(() => [
        (__props.job.opts.repeat != null)
          ? (_openBlock(), _createElementBlock("span", {
            key: 0,
            style: "margin-right: 1em;"
          }, "&lt;repeat&gt;"))
          : (_openBlock(), _createElementBlock("span", {
            key: 1,
            style: "margin-right: 1em;"
          }, "#" + _toDisplayString(__props.job.id), 1 /* TEXT */)),
        _createElementVNode("span", null, _toDisplayString(__props.job.name), 1 /* TEXT */)
      ]),
      suffix: _withCtx(() => [
        _createVNode(_component_MkTime, {
          time: __props.job.finishedOn ?? __props.job.processedOn ?? __props.job.timestamp,
          mode: "relative"
        }, null, 8 /* PROPS */, ["time"]),
        (__props.job.progress != null && typeof __props.job.progress === 'number' && __props.job.progress > 0)
          ? (_openBlock(), _createElementBlock("span", {
            key: 0,
            style: "margin-left: 1em;"
          }, _toDisplayString(Math.floor(__props.job.progress)) + "%", 1 /* TEXT */))
          : _createCommentVNode("v-if", true),
        (__props.job.opts.attempts != null && __props.job.opts.attempts > 0 && __props.job.attempts > 1)
          ? (_openBlock(), _createElementBlock("span", {
            key: 0,
            style: "margin-left: 1em; color: var(--MI_THEME-warn); font-variant-numeric: diagonal-fractions;"
          }, _toDisplayString(__props.job.attempts) + "/" + _toDisplayString(__props.job.opts.attempts), 1 /* TEXT */))
          : _createCommentVNode("v-if", true),
        (__props.job.isFailed && __props.job.finishedOn != null)
          ? (_openBlock(), _createElementBlock("span", {
            key: 0,
            style: "margin-left: 1em; color: var(--MI_THEME-error)"
          }, [
            _hoisted_1
          ]))
          : (__props.job.isFailed)
            ? (_openBlock(), _createElementBlock("span", {
              key: 1,
              style: "margin-left: 1em; color: var(--MI_THEME-warn)"
            }, [
              _hoisted_2
            ]))
          : (__props.job.finishedOn != null)
            ? (_openBlock(), _createElementBlock("span", {
              key: 2,
              style: "margin-left: 1em; color: var(--MI_THEME-success)"
            }, [
              _hoisted_3
            ]))
          : (__props.job.delay != null && __props.job.delay != 0)
            ? (_openBlock(), _createElementBlock("span", {
              key: 3,
              style: "margin-left: 1em;"
            }, [
              _hoisted_4
            ]))
          : (__props.job.processedOn != null)
            ? (_openBlock(), _createElementBlock("span", {
              key: 4,
              style: "margin-left: 1em; color: var(--MI_THEME-success)"
            }, [
              _hoisted_5
            ]))
          : _createCommentVNode("v-if", true)
      ]),
      header: _withCtx(() => [
        _createVNode(MkTabs, {
          tabs: [{
  					key: 'info',
  					title: 'Info',
  					icon: 'ti ti-info-circle',
  				}, {
  					key: 'timeline',
  					title: 'Timeline',
  					icon: 'ti ti-timeline-event',
  				}, {
  					key: 'data',
  					title: 'Data',
  					icon: 'ti ti-package',
  				}, ...(canEdit ? [{
  					key: 'dataEdit',
  					title: 'Data (edit)',
  					icon: 'ti ti-package',
  				}] : []),
  				...(__props.job.returnValue != null ? [{
  					key: 'result',
  					title: 'Result',
  					icon: 'ti ti-check',
  				}] : []),
  				...(__props.job.stacktrace.length > 0 ? [{
  					key: 'error',
  					title: 'Error',
  					icon: 'ti ti-alert-triangle',
  				}] : []), {
  					key: 'logs',
  					title: 'Logs',
  					icon: 'ti ti-logs',
  				}],
          tab: tab.value,
          "onUpdate:tab": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
        }, null, 8 /* PROPS */, ["tabs", "tab"])
      ]),
      footer: _withCtx(() => [
        _createElementVNode("div", { class: "_buttons" }, [
          _createVNode(MkButton, {
            rounded: "",
            onClick: _cache[1] || (_cache[1] = ($event: any) => (copyRaw()))
          }, {
            default: _withCtx(() => [
              _hoisted_6,
              _createTextVNode(" Copy raw")
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(MkButton, {
            rounded: "",
            onClick: _cache[2] || (_cache[2] = ($event: any) => (refresh()))
          }, {
            default: _withCtx(() => [
              _hoisted_7,
              _createTextVNode(" Refresh view")
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(MkButton, {
            rounded: "",
            onClick: _cache[3] || (_cache[3] = ($event: any) => (promoteJob()))
          }, {
            default: _withCtx(() => [
              _hoisted_8,
              _createTextVNode(" Promote")
            ]),
            _: 1 /* STABLE */
          }),
          _createTextVNode("\n\t\t\t" + "\n\t\t\t"),
          _createVNode(MkButton, {
            danger: "",
            rounded: "",
            style: "margin-left: auto;",
            onClick: _cache[4] || (_cache[4] = ($event: any) => (removeJob()))
          }, {
            default: _withCtx(() => [
              _hoisted_9,
              _createTextVNode(" Remove")
            ]),
            _: 1 /* STABLE */
          })
        ])
      ]),
      default: _withCtx(() => [
        (tab.value === 'info')
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "_gaps_s"
          }, [
            _createElementVNode("div", { style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 12px;" }, [
              _createVNode(MkKeyValue, null, {
                key: _withCtx(() => [
                  _createTextVNode("ID")
                ]),
                value: _withCtx(() => [
                  _createTextVNode(_toDisplayString(__props.job.id), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }),
              _createVNode(MkKeyValue, null, {
                key: _withCtx(() => [
                  _createTextVNode("Created at")
                ]),
                value: _withCtx(() => [
                  _createVNode(_component_MkTime, {
                    time: __props.job.timestamp,
                    mode: "detail"
                  }, null, 8 /* PROPS */, ["time"])
                ]),
                _: 1 /* STABLE */
              }),
              (__props.job.processedOn != null)
                ? (_openBlock(), _createBlock(MkKeyValue, { key: 0 }, {
                  key: _withCtx(() => [
                    _createTextVNode("Processed at")
                  ]),
                  value: _withCtx(() => [
                    _createVNode(_component_MkTime, {
                      time: __props.job.processedOn,
                      mode: "detail"
                    }, null, 8 /* PROPS */, ["time"])
                  ]),
                  _: 1 /* STABLE */
                }))
                : _createCommentVNode("v-if", true),
              (__props.job.finishedOn != null)
                ? (_openBlock(), _createBlock(MkKeyValue, { key: 0 }, {
                  key: _withCtx(() => [
                    _createTextVNode("Finished at")
                  ]),
                  value: _withCtx(() => [
                    _createVNode(_component_MkTime, {
                      time: __props.job.finishedOn,
                      mode: "detail"
                    }, null, 8 /* PROPS */, ["time"])
                  ]),
                  _: 1 /* STABLE */
                }))
                : _createCommentVNode("v-if", true),
              (__props.job.processedOn != null && __props.job.finishedOn != null)
                ? (_openBlock(), _createBlock(MkKeyValue, { key: 0 }, {
                  key: _withCtx(() => [
                    _createTextVNode("Spent")
                  ]),
                  value: _withCtx(() => [
                    _createTextVNode(_toDisplayString(__props.job.finishedOn - __props.job.processedOn) + "ms", 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }))
                : _createCommentVNode("v-if", true),
              (__props.job.failedReason != null)
                ? (_openBlock(), _createBlock(MkKeyValue, { key: 0 }, {
                  key: _withCtx(() => [
                    _createTextVNode("Failed reason")
                  ]),
                  value: _withCtx(() => [
                    _hoisted_10,
                    _createTextVNode(" "),
                    _createTextVNode(_toDisplayString(__props.job.failedReason), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }))
                : _createCommentVNode("v-if", true),
              (__props.job.opts.attempts != null && __props.job.opts.attempts > 0)
                ? (_openBlock(), _createBlock(MkKeyValue, { key: 0 }, {
                  key: _withCtx(() => [
                    _createTextVNode("Attempts")
                  ]),
                  value: _withCtx(() => [
                    _createTextVNode(_toDisplayString(__props.job.attempts) + " of " + _toDisplayString(__props.job.opts.attempts), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }))
                : _createCommentVNode("v-if", true),
              (__props.job.progress != null && typeof __props.job.progress === 'number' && __props.job.progress > 0)
                ? (_openBlock(), _createBlock(MkKeyValue, { key: 0 }, {
                  key: _withCtx(() => [
                    _createTextVNode("Progress")
                  ]),
                  value: _withCtx(() => [
                    _createTextVNode(_toDisplayString(Math.floor(__props.job.progress)) + "%", 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }))
                : _createCommentVNode("v-if", true)
            ]),
            _createVNode(MkFolder, { withSpacer: false }, {
              label: _withCtx(() => [
                _createTextVNode("Options")
              ]),
              default: _withCtx(() => [
                _createVNode(MkCode, {
                  code: JSON5.stringify(__props.job.opts, null, '\t'),
                  lang: "js"
                }, null, 8 /* PROPS */, ["code"])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["withSpacer"])
          ]))
          : (tab.value === 'timeline')
            ? (_openBlock(), _createElementBlock("div", { key: 1 }, [
              _createVNode(MkTl, {
                events: timeline.value,
                groupBy: "h"
              }, {
                left: _withCtx(({ event }) => [
                  _createElementVNode("div", null, [
                    (event.type === 'finished')
                      ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                        (__props.job.isFailed)
                          ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                            _hoisted_11,
                            _createTextVNode(),
                            _hoisted_12
                          ], 64 /* STABLE_FRAGMENT */))
                          : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                            _hoisted_13,
                            _createTextVNode(),
                            _hoisted_14
                          ], 64 /* STABLE_FRAGMENT */))
                      ], 64 /* STABLE_FRAGMENT */))
                      : (event.type === 'processed')
                        ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                          _hoisted_15,
                          _createTextVNode(),
                          _hoisted_16
                        ], 64 /* STABLE_FRAGMENT */))
                      : (event.type === 'attempt')
                        ? (_openBlock(), _createElementBlock(_Fragment, { key: 2 }, [
                          _createElementVNode("b", null, "Attempt #" + _toDisplayString(event.attempt), 1 /* TEXT */),
                          _createTextVNode(),
                          _hoisted_17
                        ], 64 /* STABLE_FRAGMENT */))
                      : (event.type === 'created')
                        ? (_openBlock(), _createElementBlock(_Fragment, { key: 3 }, [
                          _hoisted_18,
                          _createTextVNode(),
                          _hoisted_19
                        ], 64 /* STABLE_FRAGMENT */))
                      : _createCommentVNode("v-if", true)
                  ])
                ]),
                right: _withCtx(({ event, timestamp, delta }) => [
                  _createElementVNode("div", { style: "margin: 8px 0;" }, [
                    (event.type === 'attempt')
                      ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                        _hoisted_20
                      ], 64 /* STABLE_FRAGMENT */))
                      : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                        _createElementVNode("div", null, [
                          _createTextVNode("at "),
                          _createVNode(_component_MkTime, {
                            time: timestamp,
                            mode: "detail"
                          }, null, 8 /* PROPS */, ["time"])
                        ]),
                        _createElementVNode("div", _hoisted_21, _toDisplayString(timestamp) + " (+" + _toDisplayString(msSMH(delta)) + ")", 1 /* TEXT */)
                      ], 64 /* STABLE_FRAGMENT */))
                  ])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["events"])
            ]))
          : (tab.value === 'data')
            ? (_openBlock(), _createElementBlock("div", { key: 2 }, [
              _createVNode(MkCode, {
                code: JSON5.stringify(__props.job.data, null, '\t'),
                lang: "js"
              }, null, 8 /* PROPS */, ["code"])
            ]))
          : (tab.value === 'dataEdit')
            ? (_openBlock(), _createElementBlock("div", {
              key: 3,
              class: "_gaps_s"
            }, [
              _createVNode(MkCodeEditor, {
                lang: "json5",
                modelValue: editData.value,
                "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((editData).value = $event))
              }, null, 8 /* PROPS */, ["modelValue"]),
              _createVNode(MkButton, null, {
                default: _withCtx(() => [
                  _hoisted_22,
                  _createTextVNode(" Update")
                ]),
                _: 1 /* STABLE */
              })
            ]))
          : (tab.value === 'result')
            ? (_openBlock(), _createElementBlock("div", { key: 4 }, [
              _createVNode(MkCode, {
                code: JSON5.stringify(__props.job.returnValue, null, '\t'),
                lang: "json5"
              }, null, 8 /* PROPS */, ["code"])
            ]))
          : (tab.value === 'error')
            ? (_openBlock(), _createElementBlock("div", {
              key: 5,
              class: "_gaps_s"
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.job.stacktrace, (log) => {
                return (_openBlock(), _createBlock(MkCode, { code: log, lang: "stacktrace" }, null, 8 /* PROPS */, ["code"]))
              }), 256 /* UNKEYED_FRAGMENT */))
            ]))
          : (tab.value === 'logs')
            ? (_openBlock(), _createElementBlock("div", { key: 6 }, [
              _createVNode(MkButton, {
                primary: "",
                rounded: "",
                onClick: _cache[6] || (_cache[6] = ($event: any) => (loadLogs()))
              }, {
                default: _withCtx(() => [
                  _hoisted_23,
                  _createTextVNode(" Load logs")
                ]),
                _: 1 /* STABLE */
              }),
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(logs.value, (log) => {
                return (_openBlock(), _createElementBlock("div", null, _toDisplayString(log), 1 /* TEXT */))
              }), 256 /* UNKEYED_FRAGMENT */))
            ]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
