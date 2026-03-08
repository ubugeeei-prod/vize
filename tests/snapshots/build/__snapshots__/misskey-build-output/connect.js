import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-api" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-webhook" })
import { computed, ref, defineAsyncComponent, markRaw } from 'vue'
import MkPagination from '@/components/MkPagination.vue'
import FormSection from '@/components/form/section.vue'
import FormLink from '@/components/form/link.vue'
import { definePage } from '@/page.js'
import { i18n } from '@/i18n.js'
import MkFeatureBanner from '@/components/MkFeatureBanner.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import MkButton from '@/components/MkButton.vue'
import MkFolder from '@/components/MkFolder.vue'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'connect',
  setup(__props) {

const isDesktop = ref(window.innerWidth >= 1100);
const paginator = markRaw(new Paginator('i/webhooks/list', {
	limit: 100,
	noPaging: true,
}));
async function generateToken() {
	const { dispose } = await os.popupAsyncWithDialog(import('@/components/MkTokenGenerateWindow.vue').then(x => x.default), {}, {
		done: async result => {
			const { name, permissions } = result;
			const { token } = await misskeyApi('miauth/gen-token', {
				session: null,
				name: name,
				permission: permissions,
			});

			os.alert({
				type: 'success',
				title: i18n.ts.token,
				text: token,
			});
		},
		closed: () => dispose(),
	});
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts._settings.serviceConnection,
	icon: 'ti ti-link',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchText = _resolveComponent("SearchText")
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_SearchMarker = _resolveComponent("SearchMarker")
  const _component_MkTime = _resolveComponent("MkTime")

  return (_openBlock(), _createBlock(_component_SearchMarker, {
      path: "/settings/connect",
      label: _unref(i18n).ts._settings.serviceConnection,
      keywords: ['app', 'service', 'connect', 'webhook', 'api', 'token'],
      icon: "ti ti-link"
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", { class: "_gaps_m" }, [
          _createVNode(MkFeatureBanner, {
            icon: "/client-assets/link_3d.png",
            color: "#ff0088"
          }, {
            default: _withCtx(() => [
              _createVNode(_component_SearchText, null, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._settings.serviceConnectionBanner), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(_component_SearchMarker, { keywords: ['api', 'app', 'token', 'accessToken'] }, {
            default: _withCtx(() => [
              _createVNode(FormSection, null, {
                label: _withCtx(() => [
                  _hoisted_1,
                  _createTextVNode(" "),
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._settings.api), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps_m" }, [
                    _createVNode(MkButton, {
                      primary: "",
                      onClick: generateToken
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.generateAccessToken), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(FormLink, { to: "/settings/apps" }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.manageAccessTokens), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(FormLink, {
                      to: "/api-console",
                      behavior: isDesktop.value ? 'window' : null
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode("API console")
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["behavior"])
                  ])
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['webhook'] }, {
            default: _withCtx(() => [
              _createVNode(FormSection, null, {
                label: _withCtx(() => [
                  _hoisted_2,
                  _createTextVNode(" "),
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._settings.webhook), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps_m" }, [
                    _createVNode(FormLink, { to: `/settings/webhook/new` }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._webhookSettings.createWebhook), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["to"]),
                    _createVNode(MkFolder, { defaultOpen: true }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.manage), 1 /* TEXT */)
                      ]),
                      default: _withCtx(() => [
                        _createVNode(MkPagination, {
                          paginator: _unref(paginator),
                          withControl: ""
                        }, {
                          default: _withCtx(({items}) => [
                            _createElementVNode("div", { class: "_gaps" }, [
                              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (webhook) => {
                                return (_openBlock(), _createBlock(FormLink, {
                                  key: webhook.id,
                                  to: `/settings/webhook/edit/${webhook.id}`
                                }, {
                                  icon: _withCtx(() => [
                                    (webhook.active === false)
                                      ? (_openBlock(), _createElementBlock("i", {
                                        key: 0,
                                        class: "ti ti-player-pause"
                                      }))
                                      : (webhook.latestStatus === null)
                                        ? (_openBlock(), _createElementBlock("i", {
                                          key: 1,
                                          class: "ti ti-circle"
                                        }))
                                      : ([200, 201, 204].includes(webhook.latestStatus))
                                        ? (_openBlock(), _createElementBlock("i", {
                                          key: 2,
                                          class: "ti ti-check",
                                          style: { color: 'var(--MI_THEME-success)' }
                                        }))
                                      : (_openBlock(), _createElementBlock("i", {
                                        key: 3,
                                        class: "ti ti-alert-triangle",
                                        style: { color: 'var(--MI_THEME-error)' }
                                      }))
                                  ]),
                                  suffix: _withCtx(() => [
                                    (webhook.latestSentAt)
                                      ? (_openBlock(), _createBlock(_component_MkTime, {
                                        key: 0,
                                        time: webhook.latestSentAt
                                      }, null, 8 /* PROPS */, ["time"]))
                                      : _createCommentVNode("v-if", true)
                                  ]),
                                  default: _withCtx(() => [
                                    _createTextVNode("\n\t\t\t\t\t\t\t\t\t\t"),
                                    _createTextVNode(_toDisplayString(webhook.name || webhook.url), 1 /* TEXT */),
                                    _createTextVNode("\n\t\t\t\t\t\t\t\t\t\t")
                                  ]),
                                  _: 2 /* DYNAMIC */
                                }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["to"]))
                              }), 128 /* KEYED_FRAGMENT */))
                            ])
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["paginator"])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["defaultOpen"])
                  ])
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["label", "keywords"]))
}
}

})
