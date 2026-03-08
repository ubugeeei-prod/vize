import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, createSlots as _createSlots, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-info-circle" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-mail" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-link" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-link" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-link" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-link" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-link" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-user-star" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-world-cog" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-key" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-key" })
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-ad" })
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-world-search" })
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-planet" })
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-list" })
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_17 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_18 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-ghost" })
import { computed } from 'vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkInput from '@/components/MkInput.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import MkInfo from '@/components/MkInfo.vue'
import FormSplit from '@/components/form/split.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { fetchInstance, instance } from '@/instance.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import MkButton from '@/components/MkButton.vue'
import MkFolder from '@/components/MkFolder.vue'
import { useForm } from '@/composables/use-form.js'
import MkFormFooter from '@/components/MkFormFooter.vue'
import MkRadios from '@/components/MkRadios.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'settings',
  async setup(__props) {

let __temp: any, __restore: any

const meta = await misskeyApi('admin/meta');
const proxyAccount = await misskeyApi('users/show', { userId: meta.proxyAccountId });
const infoForm = useForm({
	name: meta.name ?? '',
	shortName: meta.shortName ?? '',
	description: meta.description ?? '',
	maintainerName: meta.maintainerName ?? '',
	maintainerEmail: meta.maintainerEmail ?? '',
	tosUrl: meta.tosUrl ?? '',
	privacyPolicyUrl: meta.privacyPolicyUrl ?? '',
	inquiryUrl: meta.inquiryUrl ?? '',
	repositoryUrl: meta.repositoryUrl ?? '',
	impressumUrl: meta.impressumUrl ?? '',
}, async (state) => {
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog('admin/update-meta', {)),
  await __temp,
  __restore()
)
		name: state.name,
		shortName: state.shortName === '' ? null : state.shortName,
		description: state.description,
		maintainerName: state.maintainerName,
		maintainerEmail: state.maintainerEmail,
		tosUrl: state.tosUrl,
		privacyPolicyUrl: state.privacyPolicyUrl,
		inquiryUrl: state.inquiryUrl,
		repositoryUrl: state.repositoryUrl,
		impressumUrl: state.impressumUrl,
	});
	fetchInstance(true);
});
const pinnedUsersForm = useForm({
	pinnedUsers: meta.pinnedUsers.join('\n'),
}, async (state) => {
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog('admin/update-meta', {)),
  await __temp,
  __restore()
)
		pinnedUsers: state.pinnedUsers.split('\n'),
	});
	fetchInstance(true);
});
const serviceWorkerForm = useForm({
	enableServiceWorker: meta.enableServiceWorker,
	swPublicKey: meta.swPublickey ?? '',
	swPrivateKey: meta.swPrivateKey ?? '',
}, async (state) => {
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog('admin/update-meta', {)),
  await __temp,
  __restore()
)
		enableServiceWorker: state.enableServiceWorker,
		swPublicKey: state.swPublicKey,
		swPrivateKey: state.swPrivateKey,
	});
	fetchInstance(true);
});
const adForm = useForm({
	notesPerOneAd: meta.notesPerOneAd,
}, async (state) => {
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog('admin/update-meta', {)),
  await __temp,
  __restore()
)
		notesPerOneAd: state.notesPerOneAd,
	});
	fetchInstance(true);
});
const urlPreviewForm = useForm({
	urlPreviewEnabled: meta.urlPreviewEnabled,
	urlPreviewAllowRedirect: meta.urlPreviewAllowRedirect,
	urlPreviewTimeout: meta.urlPreviewTimeout,
	urlPreviewMaximumContentLength: meta.urlPreviewMaximumContentLength,
	urlPreviewRequireContentLength: meta.urlPreviewRequireContentLength,
	urlPreviewUserAgent: meta.urlPreviewUserAgent ?? '',
	urlPreviewSummaryProxyUrl: meta.urlPreviewSummaryProxyUrl ?? '',
}, async (state) => {
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog('admin/update-meta', {)),
  await __temp,
  __restore()
)
		urlPreviewEnabled: state.urlPreviewEnabled,
		urlPreviewAllowRedirect: state.urlPreviewAllowRedirect,
		urlPreviewTimeout: state.urlPreviewTimeout,
		urlPreviewMaximumContentLength: state.urlPreviewMaximumContentLength,
		urlPreviewRequireContentLength: state.urlPreviewRequireContentLength,
		urlPreviewUserAgent: state.urlPreviewUserAgent,
		urlPreviewSummaryProxyUrl: state.urlPreviewSummaryProxyUrl,
	});
	fetchInstance(true);
});
const federationForm = useForm({
	federation: meta.federation,
	federationHosts: meta.federationHosts.join('\n'),
	deliverSuspendedSoftware: meta.deliverSuspendedSoftware,
	signToActivityPubGet: meta.signToActivityPubGet,
	proxyRemoteFiles: meta.proxyRemoteFiles,
	allowExternalApRedirect: meta.allowExternalApRedirect,
	cacheRemoteFiles: meta.cacheRemoteFiles,
	cacheRemoteSensitiveFiles: meta.cacheRemoteSensitiveFiles,
}, async (state) => {
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog('admin/update-meta', {)),
  await __temp,
  __restore()
)
		federation: state.federation,
		federationHosts: state.federationHosts.split('\n'),
		deliverSuspendedSoftware: state.deliverSuspendedSoftware,
		signToActivityPubGet: state.signToActivityPubGet,
		proxyRemoteFiles: state.proxyRemoteFiles,
		allowExternalApRedirect: state.allowExternalApRedirect,
		cacheRemoteFiles: state.cacheRemoteFiles,
		cacheRemoteSensitiveFiles: state.cacheRemoteSensitiveFiles,
	});
	fetchInstance(true);
});
const proxyAccountForm = useForm({
	description: proxyAccount.description,
}, async (state) => {
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog('admin/update-proxy-account', {)),
  await __temp,
  __restore()
)
		description: state.description,
	});
	fetchInstance(true);
});
async function openSetupWizard() {
	const { canceled } = await os.confirm({
		type: 'warning',
		title: i18n.ts._serverSettings.restartServerSetupWizardConfirm_title,
		text: i18n.ts._serverSettings.restartServerSetupWizardConfirm_text,
	});
	if (canceled) return;
	const { dispose } = await os.popupAsyncWithDialog(import('@/components/MkServerSetupWizardDialog.vue').then(x => x.default), {
	}, {
		closed: () => dispose(),
	});
}
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.general,
	icon: 'ti ti-settings',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchIcon = _resolveComponent("SearchIcon")
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_SearchMarker = _resolveComponent("SearchMarker")
  const _component_SearchText = _resolveComponent("SearchText")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")
  const _directive_panel = _resolveDirective("panel")

  return (_openBlock(), _createBlock(_component_PageWithHeader, { tabs: headerTabs.value }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 700px; --MI_SPACER-min: 16px; --MI_SPACER-max: 32px;"
        }, [
          _createVNode(_component_SearchMarker, {
            path: "/admin/settings",
            label: _unref(i18n).ts.general,
            keywords: ['general', 'settings'],
            icon: "ti ti-settings"
          }, {
            default: _withCtx(() => [
              _createElementVNode("div", { class: "_gaps_m" }, [
                _createVNode(_component_SearchMarker, { keywords: ['information', 'meta'] }, {
                  default: _withCtx((slotProps) => [
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
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.info), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      },
                      (_unref(infoForm).modified.value)
                        ? {
                          name: "footer",
                          fn: _withCtx(() => [
                            _createVNode(MkFormFooter, { form: _unref(infoForm) }, null, 8 /* PROPS */, ["form"])
                          ]),
                          key: "0"
                        }
                      : undefined
                    ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["defaultOpen"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(_component_SearchMarker, { keywords: ['pinned', 'users'] }, {
                  default: _withCtx((slotProps) => [
                    _createVNode(MkFolder, { defaultOpen: slotProps.isParentOfTarget }, _createSlots({ _: 2 /* DYNAMIC */ }, [
                      {
                        name: "icon",
                        fn: _withCtx(() => [
                          _createVNode(_component_SearchIcon, null, {
                            default: _withCtx(() => [
                              _hoisted_8
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
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.pinnedUsers), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      },
                      (_unref(pinnedUsersForm).modified.value)
                        ? {
                          name: "footer",
                          fn: _withCtx(() => [
                            _createVNode(MkFormFooter, { form: _unref(pinnedUsersForm) }, null, 8 /* PROPS */, ["form"])
                          ]),
                          key: "0"
                        }
                      : undefined
                    ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["defaultOpen"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(_component_SearchMarker, { keywords: ['serviceWorker'] }, {
                  default: _withCtx((slotProps) => [
                    _createVNode(MkFolder, { defaultOpen: slotProps.isParentOfTarget }, _createSlots({ _: 2 /* DYNAMIC */ }, [
                      {
                        name: "icon",
                        fn: _withCtx(() => [
                          _createVNode(_component_SearchIcon, null, {
                            default: _withCtx(() => [
                              _hoisted_9
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
                              _createTextVNode("ServiceWorker")
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      },
                      (_unref(serviceWorkerForm).modified.value)
                        ? {
                          name: "footer",
                          fn: _withCtx(() => [
                            _createVNode(MkFormFooter, { form: _unref(serviceWorkerForm) }, null, 8 /* PROPS */, ["form"])
                          ]),
                          key: "0"
                        }
                      : undefined
                    ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["defaultOpen"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(_component_SearchMarker, { keywords: ['ads'] }, {
                  default: _withCtx((slotProps) => [
                    _createVNode(MkFolder, { defaultOpen: slotProps.isParentOfTarget }, _createSlots({ _: 2 /* DYNAMIC */ }, [
                      {
                        name: "icon",
                        fn: _withCtx(() => [
                          _createVNode(_component_SearchIcon, null, {
                            default: _withCtx(() => [
                              _hoisted_12
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
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._ad.adsSettings), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      },
                      (_unref(adForm).modified.value)
                        ? {
                          name: "footer",
                          fn: _withCtx(() => [
                            _createVNode(MkFormFooter, { form: _unref(adForm) }, null, 8 /* PROPS */, ["form"])
                          ]),
                          key: "0"
                        }
                      : undefined
                    ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["defaultOpen"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(_component_SearchMarker, { keywords: ['url', 'preview'] }, {
                  default: _withCtx((slotProps) => [
                    _createVNode(MkFolder, { defaultOpen: slotProps.isParentOfTarget }, _createSlots({ _: 2 /* DYNAMIC */ }, [
                      {
                        name: "icon",
                        fn: _withCtx(() => [
                          _createVNode(_component_SearchIcon, null, {
                            default: _withCtx(() => [
                              _hoisted_13
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
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._urlPreviewSetting.title), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      },
                      (_unref(urlPreviewForm).modified.value)
                        ? {
                          name: "footer",
                          fn: _withCtx(() => [
                            _createVNode(MkFormFooter, { form: _unref(urlPreviewForm) }, null, 8 /* PROPS */, ["form"])
                          ]),
                          key: "0"
                        }
                      : undefined
                    ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["defaultOpen"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(_component_SearchMarker, { keywords: ['federation'] }, {
                  default: _withCtx((slotProps) => [
                    _createVNode(MkFolder, { defaultOpen: slotProps.isParentOfTarget }, _createSlots({ _: 2 /* DYNAMIC */ }, [
                      {
                        name: "icon",
                        fn: _withCtx(() => [
                          _createVNode(_component_SearchIcon, null, {
                            default: _withCtx(() => [
                              _hoisted_14
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
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.federation), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      },
                      (_unref(federationForm).savedState.federation === 'all')
                        ? {
                          name: "suffix",
                          fn: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.all), 1 /* TEXT */)
                          ]),
                          key: "0"
                        }
                      : (_unref(federationForm).savedState.federation === 'specified')
                        ? {
                          name: "suffix",
                          fn: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.specifyHost), 1 /* TEXT */)
                          ]),
                          key: "1"
                        }
                      : (_unref(federationForm).savedState.federation === 'none')
                        ? {
                          name: "suffix",
                          fn: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.none), 1 /* TEXT */)
                          ]),
                          key: "2"
                        }
                      : undefined,
                      (_unref(federationForm).modified.value)
                        ? {
                          name: "footer",
                          fn: _withCtx(() => [
                            _createVNode(MkFormFooter, { form: _unref(federationForm) }, null, 8 /* PROPS */, ["form"])
                          ]),
                          key: "0"
                        }
                      : undefined
                    ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["defaultOpen"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(_component_SearchMarker, { keywords: ['proxy', 'account'] }, {
                  default: _withCtx((slotProps) => [
                    _createVNode(MkFolder, { defaultOpen: slotProps.isParentOfTarget }, _createSlots({ _: 2 /* DYNAMIC */ }, [
                      {
                        name: "icon",
                        fn: _withCtx(() => [
                          _createVNode(_component_SearchIcon, null, {
                            default: _withCtx(() => [
                              _hoisted_18
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
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.proxyAccount), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ])
                      },
                      (_unref(proxyAccountForm).modified.value)
                        ? {
                          name: "footer",
                          fn: _withCtx(() => [
                            _createVNode(MkFormFooter, { form: _unref(proxyAccountForm) }, null, 8 /* PROPS */, ["form"])
                          ]),
                          key: "0"
                        }
                      : undefined
                    ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["defaultOpen"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(MkButton, {
                  primary: "",
                  onClick: openSetupWizard
                }, {
                  default: _withCtx(() => [
                    _createTextVNode("\n\t\t\t\t\tOpen setup wizard\n\t\t\t\t")
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
    }, 8 /* PROPS */, ["tabs"]))
}
}

})
