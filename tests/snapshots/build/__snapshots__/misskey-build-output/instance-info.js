import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "_monospace" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-refresh" })
import { ref, computed, watch, markRaw } from 'vue'
import * as Misskey from 'misskey-js'
import type { ChartSrc } from '@/components/MkChart.vue'
import MkChart from '@/components/MkChart.vue'
import MkObjectView from '@/components/MkObjectView.vue'
import FormLink from '@/components/form/link.vue'
import MkLink from '@/components/MkLink.vue'
import MkButton from '@/components/MkButton.vue'
import FormSection from '@/components/form/section.vue'
import MkKeyValue from '@/components/MkKeyValue.vue'
import MkSelect from '@/components/MkSelect.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import number from '@/filters/number.js'
import { iAmModerator, iAmAdmin } from '@/i.js'
import { definePage } from '@/page.js'
import { i18n } from '@/i18n.js'
import MkUserCardMini from '@/components/MkUserCardMini.vue'
import MkPagination from '@/components/MkPagination.vue'
import { getProxiedImageUrlNullable } from '@/utility/media-proxy.js'
import { dateString } from '@/filters/date.js'
import { useMkSelect } from '@/composables/use-mkselect.js'
import MkTextarea from '@/components/MkTextarea.vue'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'instance-info',
  props: {
    host: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const tab = ref('overview');
const {
	model: chartSrc,
	def: chartSrcDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts._instanceCharts.requests, value: 'instance-requests' },
		{ label: i18n.ts._instanceCharts.users, value: 'instance-users' },
		{ label: i18n.ts._instanceCharts.usersTotal, value: 'instance-users-total' },
		{ label: i18n.ts._instanceCharts.notes, value: 'instance-notes' },
		{ label: i18n.ts._instanceCharts.notesTotal, value: 'instance-notes-total' },
		{ label: i18n.ts._instanceCharts.ff, value: 'instance-ff' },
		{ label: i18n.ts._instanceCharts.ffTotal, value: 'instance-ff-total' },
		{ label: i18n.ts._instanceCharts.cacheSize, value: 'instance-drive-usage' },
		{ label: i18n.ts._instanceCharts.cacheSizeTotal, value: 'instance-drive-usage-total' },
		{ label: i18n.ts._instanceCharts.files, value: 'instance-drive-files' },
		{ label: i18n.ts._instanceCharts.filesTotal, value: 'instance-drive-files-total' },
	],
	initialValue: 'instance-requests',
});
const meta = ref<Misskey.entities.AdminMetaResponse | null>(null);
const instance = ref<Misskey.entities.FederationInstance | null>(null);
const suspensionState = ref<'none' | 'manuallySuspended' | 'goneSuspended' | 'autoSuspendedForNotResponding' | 'softwareSuspended'>('none');
const isBlocked = ref(false);
const isSilenced = ref(false);
const isMediaSilenced = ref(false);
const faviconUrl = ref<string | null>(null);
const moderationNote = ref('');
const usersPaginator = iAmModerator ? markRaw(new Paginator('admin/show-users', {
	limit: 10,
	params: {
		sort: '+updatedAt',
		state: 'all',
		hostname: props.host,
	},
	offsetMode: true,
})) : markRaw(new Paginator('users', {
	limit: 10,
	params: {
		sort: '+updatedAt',
		state: 'all',
		hostname: props.host,
	},
	offsetMode: true,
}));
if (iAmModerator) {
	watch(moderationNote, async () => {
		if (instance.value == null) return;
		await misskeyApi('admin/federation/update-instance', { host: instance.value.host, moderationNote: moderationNote.value });
	});
}
async function _fetch_(): Promise<void> {
	if (iAmAdmin) {
		meta.value = await misskeyApi('admin/meta');
	}
	instance.value = await misskeyApi('federation/show-instance', {
		host: props.host,
	});
	suspensionState.value = instance.value?.suspensionState ?? 'none';
	isBlocked.value = instance.value?.isBlocked ?? false;
	isSilenced.value = instance.value?.isSilenced ?? false;
	isMediaSilenced.value = instance.value?.isMediaSilenced ?? false;
	faviconUrl.value = getProxiedImageUrlNullable(instance.value?.faviconUrl, 'preview') ?? getProxiedImageUrlNullable(instance.value?.iconUrl, 'preview');
	moderationNote.value = instance.value?.moderationNote ?? '';
}
async function toggleBlock(): Promise<void> {
	if (!iAmAdmin) return;
	if (!meta.value) throw new Error('No meta?');
	if (!instance.value) throw new Error('No instance?');
	const { host } = instance.value;
	await misskeyApi('admin/update-meta', {
		blockedHosts: isBlocked.value ? meta.value.blockedHosts.concat([host]) : meta.value.blockedHosts.filter(x => x !== host),
	});
}
async function toggleSilenced(): Promise<void> {
	if (!iAmAdmin) return;
	if (!meta.value) throw new Error('No meta?');
	if (!instance.value) throw new Error('No instance?');
	const { host } = instance.value;
	const silencedHosts = meta.value.silencedHosts ?? [];
	await misskeyApi('admin/update-meta', {
		silencedHosts: isSilenced.value ? silencedHosts.concat([host]) : silencedHosts.filter(x => x !== host),
	});
}
async function toggleMediaSilenced(): Promise<void> {
	if (!iAmAdmin) return;
	if (!meta.value) throw new Error('No meta?');
	if (!instance.value) throw new Error('No instance?');
	const { host } = instance.value;
	const mediaSilencedHosts = meta.value.mediaSilencedHosts ?? [];
	await misskeyApi('admin/update-meta', {
		mediaSilencedHosts: isMediaSilenced.value ? mediaSilencedHosts.concat([host]) : mediaSilencedHosts.filter(x => x !== host),
	});
}
async function stopDelivery(): Promise<void> {
	if (!iAmModerator) return;
	if (!instance.value) throw new Error('No instance?');
	suspensionState.value = 'manuallySuspended';
	await misskeyApi('admin/federation/update-instance', {
		host: instance.value.host,
		isSuspended: true,
	});
}
async function resumeDelivery(): Promise<void> {
	if (!iAmModerator) return;
	if (!instance.value) throw new Error('No instance?');
	suspensionState.value = 'none';
	await misskeyApi('admin/federation/update-instance', {
		host: instance.value.host,
		isSuspended: false,
	});
}
function refreshMetadata(): void {
	if (!iAmModerator) return;
	if (!instance.value) throw new Error('No instance?');
	misskeyApi('admin/federation/refresh-remote-instance-metadata', {
		host: instance.value.host,
	});
	os.alert({
		text: 'Refresh requested',
	});
}
_fetch_();
const headerActions = computed(() => [{
	text: `https://${props.host}`,
	icon: 'ti ti-external-link',
	handler: () => {
		window.open(`https://${props.host}`, '_blank', 'noopener');
	},
}]);
const headerTabs = computed(() => [{
	key: 'overview',
	title: i18n.ts.overview,
	icon: 'ti ti-info-circle',
}, ...(iAmModerator ? [{
	key: 'chart',
	title: i18n.ts.charts,
	icon: 'ti ti-chart-line',
}, {
	key: 'users',
	title: i18n.ts.users,
	icon: 'ti ti-users',
}] : []), {
	key: 'raw',
	title: 'Raw',
	icon: 'ti ti-code',
}]);
definePage(() => ({
	title: props.host,
	icon: 'ti ti-server',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_MkA = _resolveComponent("MkA")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value,
      swipable: true,
      tab: tab.value,
      "onUpdate:tab": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
    }, {
      default: _withCtx(() => [
        (instance.value)
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "_spacer",
            style: "--MI_SPACER-w: 600px; --MI_SPACER-min: 16px; --MI_SPACER-max: 32px;"
          }, [
            (tab.value === 'overview')
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: "_gaps_m"
              }, [
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.faviconAndName)
                }, [
                  (faviconUrl.value)
                    ? (_openBlock(), _createElementBlock("img", {
                      key: 0,
                      src: faviconUrl.value,
                      alt: "",
                      class: _normalizeClass(_ctx.$style.icon)
                    }))
                    : _createCommentVNode("v-if", true),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.name)
                  }, _toDisplayString(instance.value.name || `(${_unref(i18n).ts.unknown})`), 1 /* TEXT */)
                ]),
                _createElementVNode("div", { style: "display: flex; flex-direction: column; gap: 1em;" }, [
                  _createVNode(MkKeyValue, {
                    copy: __props.host,
                    oneline: ""
                  }, {
                    key: _withCtx(() => [
                      _createTextVNode("Host")
                    ]),
                    value: _withCtx(() => [
                      _createElementVNode("span", { class: "_monospace" }, [
                        _createVNode(MkLink, { url: `https://${__props.host}` }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(__props.host), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["url"])
                      ])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["copy"]),
                  _createVNode(MkKeyValue, { oneline: "" }, {
                    key: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.software), 1 /* TEXT */)
                    ]),
                    value: _withCtx(() => [
                      _createElementVNode("span", _hoisted_1, _toDisplayString(instance.value.softwareName || `(${_unref(i18n).ts.unknown})`) + " / " + _toDisplayString(instance.value.softwareVersion || `(${_unref(i18n).ts.unknown})`), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }),
                  _createVNode(MkKeyValue, { oneline: "" }, {
                    key: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.administrator), 1 /* TEXT */)
                    ]),
                    value: _withCtx(() => [
                      _createTextVNode(_toDisplayString(instance.value.maintainerName || `(${_unref(i18n).ts.unknown})`) + " (" + _toDisplayString(instance.value.maintainerEmail || `(${_unref(i18n).ts.unknown})`) + ")", 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _createVNode(MkKeyValue, null, {
                  key: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.description), 1 /* TEXT */)
                  ]),
                  value: _withCtx(() => [
                    _createTextVNode(_toDisplayString(instance.value.description), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }),
                (_unref(iAmModerator))
                  ? (_openBlock(), _createBlock(FormSection, { key: 0 }, {
                    label: _withCtx(() => [
                      _createTextVNode("Moderation")
                    ]),
                    default: _withCtx(() => [
                      _createElementVNode("div", { class: "_gaps_s" }, [
                        _createVNode(MkKeyValue, null, {
                          key: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts._delivery.status), 1 /* TEXT */)
                          ]),
                          value: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts._delivery._type[suspensionState.value]), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }),
                        (suspensionState.value === 'none')
                          ? (_openBlock(), _createBlock(MkButton, {
                            key: 0,
                            disabled: !instance.value,
                            danger: "",
                            onClick: stopDelivery
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._delivery.stop), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["disabled"]))
                          : _createCommentVNode("v-if", true),
                        (suspensionState.value !== 'none')
                          ? (_openBlock(), _createBlock(MkButton, {
                            key: 0,
                            disabled: !instance.value || suspensionState.value == 'softwareSuspended',
                            onClick: resumeDelivery
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._delivery.resume), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["disabled"]))
                          : _createCommentVNode("v-if", true),
                        _createVNode(MkSwitch, {
                          disabled: !meta.value || !instance.value,
                          "onUpdate:modelValue": [toggleBlock, ($event: any) => ((isBlocked).value = $event)],
                          modelValue: isBlocked.value
                        }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.blockThisInstance), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["disabled", "modelValue"]),
                        _createVNode(MkSwitch, {
                          disabled: !meta.value || !instance.value,
                          "onUpdate:modelValue": [toggleSilenced, ($event: any) => ((isSilenced).value = $event)],
                          modelValue: isSilenced.value
                        }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.silenceThisInstance), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["disabled", "modelValue"]),
                        _createVNode(MkSwitch, {
                          disabled: !meta.value || !instance.value,
                          "onUpdate:modelValue": [toggleMediaSilenced, ($event: any) => ((isMediaSilenced).value = $event)],
                          modelValue: isMediaSilenced.value
                        }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.mediaSilenceThisInstance), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["disabled", "modelValue"]),
                        _createVNode(MkButton, { onClick: refreshMetadata }, {
                          default: _withCtx(() => [
                            _hoisted_2,
                            _createTextVNode(" Refresh metadata")
                          ]),
                          _: 1 /* STABLE */
                        }),
                        _createVNode(MkTextarea, {
                          manualSave: "",
                          modelValue: moderationNote.value,
                          "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((moderationNote).value = $event))
                        }, {
                          label: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.moderationNote), 1 /* TEXT */)
                          ]),
                          caption: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.moderationNoteDescription), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["modelValue"])
                      ])
                    ]),
                    _: 1 /* STABLE */
                  }))
                  : _createCommentVNode("v-if", true),
                _createVNode(FormSection, null, {
                  default: _withCtx(() => [
                    _createVNode(MkKeyValue, {
                      oneline: "",
                      style: "margin: 1em 0;"
                    }, {
                      key: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.registeredAt), 1 /* TEXT */)
                      ]),
                      value: _withCtx(() => [
                        _createVNode(_component_MkTime, {
                          mode: "detail",
                          time: instance.value.firstRetrievedAt
                        }, null, 8 /* PROPS */, ["time"])
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(MkKeyValue, {
                      oneline: "",
                      style: "margin: 1em 0;"
                    }, {
                      key: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.updatedAt), 1 /* TEXT */)
                      ]),
                      value: _withCtx(() => [
                        _createVNode(_component_MkTime, {
                          mode: "detail",
                          time: instance.value.infoUpdatedAt
                        }, null, 8 /* PROPS */, ["time"])
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(MkKeyValue, {
                      oneline: "",
                      style: "margin: 1em 0;"
                    }, {
                      key: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.latestRequestReceivedAt), 1 /* TEXT */)
                      ]),
                      value: _withCtx(() => [
                        (instance.value.latestRequestReceivedAt)
                          ? (_openBlock(), _createBlock(_component_MkTime, {
                            key: 0,
                            time: instance.value.latestRequestReceivedAt
                          }, null, 8 /* PROPS */, ["time"]))
                          : (_openBlock(), _createElementBlock("span", { key: 1 }, "N/A"))
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(FormSection, null, {
                  default: _withCtx(() => [
                    _createVNode(MkKeyValue, {
                      oneline: "",
                      style: "margin: 1em 0;"
                    }, {
                      key: _withCtx(() => [
                        _createTextVNode("Following (Pub)")
                      ]),
                      value: _withCtx(() => [
                        _createTextVNode(_toDisplayString(number(instance.value.followingCount)), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(MkKeyValue, {
                      oneline: "",
                      style: "margin: 1em 0;"
                    }, {
                      key: _withCtx(() => [
                        _createTextVNode("Followers (Sub)")
                      ]),
                      value: _withCtx(() => [
                        _createTextVNode(_toDisplayString(number(instance.value.followersCount)), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(FormSection, null, {
                  label: _withCtx(() => [
                    _createTextVNode("Well-known resources")
                  ]),
                  default: _withCtx(() => [
                    _createVNode(FormLink, {
                      to: `https://${__props.host}/.well-known/host-meta`,
                      external: "",
                      style: "margin-bottom: 8px;"
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode("host-meta")
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["to"]),
                    _createVNode(FormLink, {
                      to: `https://${__props.host}/.well-known/host-meta.json`,
                      external: "",
                      style: "margin-bottom: 8px;"
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode("host-meta.json")
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["to"]),
                    _createVNode(FormLink, {
                      to: `https://${__props.host}/.well-known/nodeinfo`,
                      external: "",
                      style: "margin-bottom: 8px;"
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode("nodeinfo")
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["to"]),
                    _createVNode(FormLink, {
                      to: `https://${__props.host}/robots.txt`,
                      external: "",
                      style: "margin-bottom: 8px;"
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode("robots.txt")
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["to"]),
                    _createVNode(FormLink, {
                      to: `https://${__props.host}/manifest.json`,
                      external: "",
                      style: "margin-bottom: 8px;"
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode("manifest.json")
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["to"])
                  ]),
                  _: 1 /* STABLE */
                })
              ]))
              : (tab.value === 'chart')
                ? (_openBlock(), _createElementBlock("div", {
                  key: 1,
                  class: "_gaps_m"
                }, [
                  _createElementVNode("div", null, [
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.selects)
                    }, [
                      _createVNode(MkSelect, {
                        items: _unref(chartSrcDef),
                        style: "margin: 0 10px 0 0; flex: 1;",
                        modelValue: _unref(chartSrc),
                        "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((chartSrc).value = $event))
                      }, null, 8 /* PROPS */, ["items", "modelValue"])
                    ]),
                    _createElementVNode("div", null, [
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.label)
                      }, _toDisplayString(_unref(i18n).tsx.recentNHours({ n: 90 })), 1 /* TEXT */),
                      _createVNode(MkChart, {
                        src: _unref(chartSrc),
                        span: "hour",
                        limit: 90,
                        args: { host: __props.host },
                        detailed: true
                      }, null, 8 /* PROPS */, ["src", "limit", "args", "detailed"]),
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.label)
                      }, _toDisplayString(_unref(i18n).tsx.recentNDays({ n: 90 })), 1 /* TEXT */),
                      _createVNode(MkChart, {
                        src: _unref(chartSrc),
                        span: "day",
                        limit: 90,
                        args: { host: __props.host },
                        detailed: true
                      }, null, 8 /* PROPS */, ["src", "limit", "args", "detailed"])
                    ])
                  ])
                ]))
              : (tab.value === 'users')
                ? (_openBlock(), _createElementBlock("div", {
                  key: 2,
                  class: "_gaps_m"
                }, [
                  _createVNode(MkPagination, { paginator: _unref(usersPaginator) }, {
                    default: _withCtx(({ items }) => [
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.users)
                      }, [
                        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (user) => {
                          return (_openBlock(), _createBlock(_component_MkA, {
                            key: user.id,
                            to: `/admin/user/${user.id}`
                          }, {
                            default: _withCtx(() => [
                              _createVNode(MkUserCardMini, { user: user }, null, 8 /* PROPS */, ["user"])
                            ]),
                            _: 2 /* DYNAMIC */
                          }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["to"]))
                        }), 128 /* KEYED_FRAGMENT */))
                      ])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["paginator"])
                ]))
              : (tab.value === 'raw')
                ? (_openBlock(), _createElementBlock("div", {
                  key: 3,
                  class: "_gaps_m"
                }, [
                  _createVNode(MkObjectView, {
                    tall: "",
                    value: instance.value
                  }, null, 8 /* PROPS */, ["value"])
                ]))
              : _createCommentVNode("v-if", true)
          ]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs", "swipable", "tab"]))
}
}

})
