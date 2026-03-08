import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "ip _monospace" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-refresh" })
import { computed, markRaw } from 'vue'
import X2fa from './2fa.vue'
import FormSection from '@/components/form/section.vue'
import FormSlot from '@/components/form/slot.vue'
import MkButton from '@/components/MkButton.vue'
import MkPagination from '@/components/MkPagination.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import MkFeatureBanner from '@/components/MkFeatureBanner.vue'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'security',
  setup(__props) {

const paginator = markRaw(new Paginator('i/signin-history', {
	limit: 5,
}));
async function change() {
	const { canceled: canceled2, result: newPassword } = await os.inputText({
		title: i18n.ts.newPassword,
		type: 'password',
		autocomplete: 'new-password',
	});
	if (canceled2 || newPassword == null) return;
	const { canceled: canceled3, result: newPassword2 } = await os.inputText({
		title: i18n.ts.newPasswordRetype,
		type: 'password',
		autocomplete: 'new-password',
	});
	if (canceled3 || newPassword2 == null) return;
	if (newPassword !== newPassword2) {
		os.alert({
			type: 'error',
			text: i18n.ts.retypedNotMatch,
		});
		return;
	}
	const auth = await os.authenticateDialog();
	if (auth.canceled) return;
	os.apiWithDialog('i/change-password', {
		currentPassword: auth.result.password,
		token: auth.result.token,
		newPassword,
	});
}
async function regenerateToken() {
	const auth = await os.authenticateDialog();
	if (auth.canceled) return;
	misskeyApi('i/regenerate-token', {
		password: auth.result.password,
		token: auth.result.token,
	});
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.security,
	icon: 'ti ti-lock',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchText = _resolveComponent("SearchText")
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_SearchMarker = _resolveComponent("SearchMarker")
  const _component_MkTime = _resolveComponent("MkTime")
  const _directive_panel = _resolveDirective("panel")

  return (_openBlock(), _createBlock(_component_SearchMarker, {
      path: "/settings/security",
      label: _unref(i18n).ts.security,
      keywords: ['security'],
      icon: "ti ti-lock",
      inlining: ['2fa']
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", { class: "_gaps_m" }, [
          _createVNode(MkFeatureBanner, {
            icon: "/client-assets/locked_with_key_3d.png",
            color: "#ffbf00"
          }, {
            default: _withCtx(() => [
              _createVNode(_component_SearchText, null, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._settings.securityBanner), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(_component_SearchMarker, { keywords: ['password'] }, {
            default: _withCtx(() => [
              _createVNode(FormSection, { first: "" }, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.password), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                default: _withCtx(() => [
                  _createVNode(_component_SearchMarker, null, {
                    default: _withCtx(() => [
                      _createVNode(MkButton, {
                        primary: "",
                        onClick: _cache[0] || (_cache[0] = ($event: any) => (change()))
                      }, {
                        default: _withCtx(() => [
                          _createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.changePassword), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]),
                        _: 1 /* STABLE */
                      })
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(X2fa),
          _createVNode(_component_SearchMarker, { keywords: ['signin', 'login', 'history', 'log'] }, {
            default: _withCtx(() => [
              _createVNode(FormSection, null, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.signinHistory), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                default: _withCtx(() => [
                  _createVNode(MkPagination, {
                    paginator: _unref(paginator),
                    withControl: "",
                    forceDisableInfiniteScroll: true
                  }, {
                    default: _withCtx(({items}) => [
                      _createElementVNode("div", null, [
                        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
                          return (_openBlock(), _createElementBlock("div", {
                            key: item.id,
                            class: "timnmucd"
                          }, [
                            _createElementVNode("header", null, [
                              (item.success)
                                ? (_openBlock(), _createElementBlock("i", {
                                  key: 0,
                                  class: "ti ti-check icon succ"
                                }))
                                : (_openBlock(), _createElementBlock("i", {
                                  key: 1,
                                  class: "ti ti-circle-x icon fail"
                                })),
                              _createElementVNode("code", _hoisted_1, _toDisplayString(item.ip), 1 /* TEXT */),
                              _createVNode(_component_MkTime, {
                                time: item.createdAt,
                                class: "time"
                              }, null, 8 /* PROPS */, ["time"])
                            ])
                          ]))
                        }), 128 /* KEYED_FRAGMENT */))
                      ])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["paginator", "forceDisableInfiniteScroll"])
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['regenerate', 'refresh', 'reset', 'token'] }, {
            default: _withCtx(() => [
              _createVNode(FormSection, null, {
                default: _withCtx(() => [
                  _createVNode(FormSlot, null, {
                    caption: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.regenerateLoginTokenDescription), 1 /* TEXT */)
                    ]),
                    default: _withCtx(() => [
                      _createVNode(MkButton, {
                        danger: "",
                        onClick: regenerateToken
                      }, {
                        default: _withCtx(() => [
                          _hoisted_2,
                          _createTextVNode(" "),
                          _createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.regenerateLoginToken), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]),
                        _: 1 /* STABLE */
                      })
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["label", "keywords", "inlining"]))
}
}

})
