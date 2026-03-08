import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, createSlots as _createSlots, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-bolt" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-bolt" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-recycle" })
import { ref, computed } from 'vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { fetchInstance } from '@/instance.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import MkSwitch from '@/components/MkSwitch.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkInput from '@/components/MkInput.vue'
import MkLink from '@/components/MkLink.vue'
import { useForm } from '@/composables/use-form.js'
import MkFormFooter from '@/components/MkFormFooter.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'performance',
  async setup(__props) {

let __temp: any, __restore: any

const meta = await misskeyApi('admin/meta');
const enableServerMachineStats = ref(meta.enableServerMachineStats);
const enableIdenticonGeneration = ref(meta.enableIdenticonGeneration);
const enableChartsForRemoteUser = ref(meta.enableChartsForRemoteUser);
const enableStatsForFederatedInstances = ref(meta.enableStatsForFederatedInstances);
const enableChartsForFederatedInstances = ref(meta.enableChartsForFederatedInstances);
const showRoleBadgesOfRemoteUsers = ref(meta.showRoleBadgesOfRemoteUsers);
function onChange_enableServerMachineStats(value: boolean) {
	os.apiWithDialog('admin/update-meta', {
		enableServerMachineStats: value,
	}).then(() => {
		fetchInstance(true);
	});
}
function onChange_enableIdenticonGeneration(value: boolean) {
	os.apiWithDialog('admin/update-meta', {
		enableIdenticonGeneration: value,
	}).then(() => {
		fetchInstance(true);
	});
}
function onChange_enableChartsForRemoteUser(value: boolean) {
	os.apiWithDialog('admin/update-meta', {
		enableChartsForRemoteUser: value,
	}).then(() => {
		fetchInstance(true);
	});
}
function onChange_enableStatsForFederatedInstances(value: boolean) {
	os.apiWithDialog('admin/update-meta', {
		enableStatsForFederatedInstances: value,
	}).then(() => {
		fetchInstance(true);
	});
}
function onChange_enableChartsForFederatedInstances(value: boolean) {
	os.apiWithDialog('admin/update-meta', {
		enableChartsForFederatedInstances: value,
	}).then(() => {
		fetchInstance(true);
	});
}
function onChange_showRoleBadgesOfRemoteUsers(value: boolean) {
	os.apiWithDialog('admin/update-meta', {
		showRoleBadgesOfRemoteUsers: value,
	}).then(() => {
		fetchInstance(true);
	});
}
const fttForm = useForm({
	enableFanoutTimeline: meta.enableFanoutTimeline,
	enableFanoutTimelineDbFallback: meta.enableFanoutTimelineDbFallback,
	perLocalUserUserTimelineCacheMax: meta.perLocalUserUserTimelineCacheMax,
	perRemoteUserUserTimelineCacheMax: meta.perRemoteUserUserTimelineCacheMax,
	perUserHomeTimelineCacheMax: meta.perUserHomeTimelineCacheMax,
	perUserListTimelineCacheMax: meta.perUserListTimelineCacheMax,
}, async (state) => {
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog('admin/update-meta', {)),
  await __temp,
  __restore()
)
		enableFanoutTimeline: state.enableFanoutTimeline,
		enableFanoutTimelineDbFallback: state.enableFanoutTimelineDbFallback,
		perLocalUserUserTimelineCacheMax: state.perLocalUserUserTimelineCacheMax,
		perRemoteUserUserTimelineCacheMax: state.perRemoteUserUserTimelineCacheMax,
		perUserHomeTimelineCacheMax: state.perUserHomeTimelineCacheMax,
		perUserListTimelineCacheMax: state.perUserListTimelineCacheMax,
	});
	fetchInstance(true);
});
const rbtForm = useForm({
	enableReactionsBuffering: meta.enableReactionsBuffering,
}, async (state) => {
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog('admin/update-meta', {)),
  await __temp,
  __restore()
)
		enableReactionsBuffering: state.enableReactionsBuffering,
	});
	fetchInstance(true);
});
const remoteNotesCleaningForm = useForm({
	enableRemoteNotesCleaning: meta.enableRemoteNotesCleaning,
	remoteNotesCleaningExpiryDaysForEachNotes: meta.remoteNotesCleaningExpiryDaysForEachNotes,
	remoteNotesCleaningMaxProcessingDurationInMinutes: meta.remoteNotesCleaningMaxProcessingDurationInMinutes,
}, async (state) => {
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog('admin/update-meta', {)),
  await __temp,
  __restore()
)
		enableRemoteNotesCleaning: state.enableRemoteNotesCleaning,
		remoteNotesCleaningExpiryDaysForEachNotes: state.remoteNotesCleaningExpiryDaysForEachNotes,
		remoteNotesCleaningMaxProcessingDurationInMinutes: state.remoteNotesCleaningMaxProcessingDurationInMinutes,
	});
	fetchInstance(true);
});
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.performance,
	icon: 'ti ti-bolt',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_SearchMarker = _resolveComponent("SearchMarker")
  const _component_SearchIcon = _resolveComponent("SearchIcon")
  const _component_SearchText = _resolveComponent("SearchText")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 700px; --MI_SPACER-min: 16px; --MI_SPACER-max: 32px;"
        }, [
          _createVNode(_component_SearchMarker, {
            path: "/admin/performance",
            label: _unref(i18n).ts.performance,
            keywords: ['performance'],
            icon: "ti ti-bolt"
          }, {
            default: _withCtx(() => [
              _createElementVNode("div", { class: "_gaps" }, [
                _createVNode(_component_SearchMarker, null, {
                  default: _withCtx(() => [
                    _createElementVNode("div", {
                      class: "_panel",
                      style: "padding: 16px;"
                    }, [
                      _createVNode(MkSwitch, {
                        onChange: onChange_enableServerMachineStats,
                        modelValue: enableServerMachineStats.value,
                        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((enableServerMachineStats).value = $event))
                      }, {
                        label: _withCtx(() => [
                          _createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.enableServerMachineStats), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]),
                        caption: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.turnOffToImprovePerformance), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["modelValue"])
                    ])
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(_component_SearchMarker, null, {
                  default: _withCtx(() => [
                    _createElementVNode("div", {
                      class: "_panel",
                      style: "padding: 16px;"
                    }, [
                      _createVNode(MkSwitch, {
                        onChange: onChange_enableIdenticonGeneration,
                        modelValue: enableIdenticonGeneration.value,
                        "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((enableIdenticonGeneration).value = $event))
                      }, {
                        label: _withCtx(() => [
                          _createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.enableIdenticonGeneration), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]),
                        caption: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.turnOffToImprovePerformance), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["modelValue"])
                    ])
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(_component_SearchMarker, null, {
                  default: _withCtx(() => [
                    _createElementVNode("div", {
                      class: "_panel",
                      style: "padding: 16px;"
                    }, [
                      _createVNode(MkSwitch, {
                        onChange: onChange_enableChartsForRemoteUser,
                        modelValue: enableChartsForRemoteUser.value,
                        "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((enableChartsForRemoteUser).value = $event))
                      }, {
                        label: _withCtx(() => [
                          _createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.enableChartsForRemoteUser), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]),
                        caption: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.turnOffToImprovePerformance), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["modelValue"])
                    ])
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(_component_SearchMarker, null, {
                  default: _withCtx(() => [
                    _createElementVNode("div", {
                      class: "_panel",
                      style: "padding: 16px;"
                    }, [
                      _createVNode(MkSwitch, {
                        onChange: onChange_enableStatsForFederatedInstances,
                        modelValue: enableStatsForFederatedInstances.value,
                        "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((enableStatsForFederatedInstances).value = $event))
                      }, {
                        label: _withCtx(() => [
                          _createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.enableStatsForFederatedInstances), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]),
                        caption: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.turnOffToImprovePerformance), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["modelValue"])
                    ])
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(_component_SearchMarker, null, {
                  default: _withCtx(() => [
                    _createElementVNode("div", {
                      class: "_panel",
                      style: "padding: 16px;"
                    }, [
                      _createVNode(MkSwitch, {
                        onChange: onChange_enableChartsForFederatedInstances,
                        modelValue: enableChartsForFederatedInstances.value,
                        "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((enableChartsForFederatedInstances).value = $event))
                      }, {
                        label: _withCtx(() => [
                          _createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.enableChartsForFederatedInstances), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]),
                        caption: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.turnOffToImprovePerformance), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["modelValue"])
                    ])
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(_component_SearchMarker, null, {
                  default: _withCtx(() => [
                    _createElementVNode("div", {
                      class: "_panel",
                      style: "padding: 16px;"
                    }, [
                      _createVNode(MkSwitch, {
                        onChange: onChange_showRoleBadgesOfRemoteUsers,
                        modelValue: showRoleBadgesOfRemoteUsers.value,
                        "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((showRoleBadgesOfRemoteUsers).value = $event))
                      }, {
                        label: _withCtx(() => [
                          _createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.showRoleBadgesOfRemoteUsers), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]),
                        caption: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.turnOffToImprovePerformance), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["modelValue"])
                    ])
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(_component_SearchMarker, null, {
                  default: _withCtx(() => [
                    _createVNode(MkFolder, { defaultOpen: true }, _createSlots({ _: 2 /* DYNAMIC */ }, [
                      {
                        name: "icon",
                        fn: _withCtx(() => [
                          _createVNode(_component_SearchIcon, null, {
                            default: _withCtx(() => [
                              _hoisted_1
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      },
                      {
                        name: "label",
                        fn: _withCtx(() => [
                          _createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [
                              _createTextVNode("Misskey® Fan-out Timeline Technology™ (FTT)")
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      },
                      (_unref(fttForm).savedState.enableFanoutTimeline)
                        ? {
                          name: "suffix",
                          fn: _withCtx(() => [
                            _createTextVNode("Enabled")
                          ]),
                          key: "0"
                        }
                      : {
                        name: "suffix",
                        fn: _withCtx(() => [
                          _createTextVNode("Disabled")
                        ]),
                        key: "1"
                      }
                      : undefined,
                      (_unref(fttForm).modified.value)
                        ? {
                          name: "footer",
                          fn: _withCtx(() => [
                            _createVNode(MkFormFooter, { form: _unref(fttForm) }, null, 8 /* PROPS */, ["form"])
                          ]),
                          key: "0"
                        }
                      : undefined
                    ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["defaultOpen"])
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(_component_SearchMarker, null, {
                  default: _withCtx(() => [
                    _createVNode(MkFolder, { defaultOpen: true }, _createSlots({ _: 2 /* DYNAMIC */ }, [
                      {
                        name: "icon",
                        fn: _withCtx(() => [
                          _createVNode(_component_SearchIcon, null, {
                            default: _withCtx(() => [
                              _hoisted_2
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      },
                      {
                        name: "label",
                        fn: _withCtx(() => [
                          _createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [
                              _createTextVNode("Misskey® Reactions Boost Technology™ (RBT)")
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      },
                      (_unref(rbtForm).savedState.enableReactionsBuffering)
                        ? {
                          name: "suffix",
                          fn: _withCtx(() => [
                            _createTextVNode("Enabled")
                          ]),
                          key: "0"
                        }
                      : {
                        name: "suffix",
                        fn: _withCtx(() => [
                          _createTextVNode("Disabled")
                        ]),
                        key: "1"
                      }
                      : undefined,
                      (_unref(rbtForm).modified.value)
                        ? {
                          name: "footer",
                          fn: _withCtx(() => [
                            _createVNode(MkFormFooter, { form: _unref(rbtForm) }, null, 8 /* PROPS */, ["form"])
                          ]),
                          key: "0"
                        }
                      : undefined
                    ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["defaultOpen"])
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(_component_SearchMarker, null, {
                  default: _withCtx(() => [
                    _createVNode(MkFolder, { defaultOpen: true }, _createSlots({ _: 2 /* DYNAMIC */ }, [
                      {
                        name: "icon",
                        fn: _withCtx(() => [
                          _createVNode(_component_SearchIcon, null, {
                            default: _withCtx(() => [
                              _hoisted_3
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      },
                      {
                        name: "label",
                        fn: _withCtx(() => [
                          _createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [
                              _createTextVNode("Remote Notes Cleaning (仮)")
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      },
                      (_unref(remoteNotesCleaningForm).savedState.enableRemoteNotesCleaning)
                        ? {
                          name: "suffix",
                          fn: _withCtx(() => [
                            _createTextVNode("Enabled")
                          ]),
                          key: "0"
                        }
                      : {
                        name: "suffix",
                        fn: _withCtx(() => [
                          _createTextVNode("Disabled")
                        ]),
                        key: "1"
                      }
                      : undefined,
                      (_unref(remoteNotesCleaningForm).modified.value)
                        ? {
                          name: "footer",
                          fn: _withCtx(() => [
                            _createVNode(MkFormFooter, { form: _unref(remoteNotesCleaningForm) }, null, 8 /* PROPS */, ["form"])
                          ]),
                          key: "0"
                        }
                      : undefined
                    ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["defaultOpen"])
                  ]),
                  _: 1 /* STABLE */
                })
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["label", "keywords"])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
