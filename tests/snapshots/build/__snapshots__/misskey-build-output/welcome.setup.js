import { defineComponent as _defineComponent } from 'vue'
import { Suspense as _Suspense, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("stop", { offset: "0%", "stop-color": "#86b300" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("stop", { offset: "100%", "stop-color": "#4ab300" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("animate", { attributeName: "d", dur: "20s", repeatCount: "indefinite", keyTimes: "0;0.333;0.667;1", calcmod: "spline", keySplines: "0.2 0 0.2 1;0.2 0 0.2 1;0.2 0 0.2 1", begin: "0s", values: "M0 0L 0 220Q 213.5 260 427 230T 854 255L 854 0 Z;M0 0L 0 245Q 213.5 260 427 240T 854 230L 854 0 Z;M0 0L 0 265Q 213.5 235 427 265T 854 230L 854 0 Z;M0 0L 0 220Q 213.5 260 427 230T 854 255L 854 0 Z" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("animate", { attributeName: "d", dur: "20s", repeatCount: "indefinite", keyTimes: "0;0.333;0.667;1", calcmod: "spline", keySplines: "0.2 0 0.2 1;0.2 0 0.2 1;0.2 0 0.2 1", begin: "-10s", values: "M0 0L 0 235Q 213.5 280 427 250T 854 260L 854 0 Z;M0 0L 0 250Q 213.5 220 427 220T 854 240L 854 0 Z;M0 0L 0 245Q 213.5 225 427 250T 854 265L 854 0 Z;M0 0L 0 235Q 213.5 280 427 250T 854 260L 854 0 Z" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("div", null, "Welcome to Misskey!")
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-help-circle" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-lock" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-help-circle" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-lock" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("br")
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("br")
import { ref } from 'vue'
import { host, version } from '@@/js/config.js'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { login } from '@/accounts.js'
import MkLink from '@/components/MkLink.vue'
import MkServerSetupWizard from '@/components/MkServerSetupWizard.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'welcome.setup',
  setup(__props) {

const username = ref('');
const password = ref('');
const setupPassword = ref('');
const accountCreating = ref(false);
const accountCreated = ref(false);
const step = ref(0);
let token: string | null = null;
function createAccount() {
	if (accountCreating.value) return;
	accountCreating.value = true;
	const _close = os.waiting();
	misskeyApi('admin/accounts/create', {
		username: username.value,
		password: password.value,
		setupPassword: setupPassword.value === '' ? null : setupPassword.value,
	}).then(res => {
		token = res.token;
		accountCreated.value = true;
	}).catch((err) => {
		accountCreating.value = false;
		let title = i18n.ts.somethingHappened;
		let text = err.message + '\n' + err.id;
		if (err.code === 'ACCESS_DENIED') {
			title = i18n.ts.permissionDeniedError;
			text = i18n.ts.operationForbidden;
		} else if (err.code === 'INCORRECT_INITIAL_PASSWORD') {
			title = i18n.ts.permissionDeniedError;
			text = i18n.ts.incorrectPassword;
		}
		os.alert({
			type: 'error',
			title,
			text,
		});
	}).finally(() => {
		_close();
	});
}
function onWizardFinished() {
	step.value++;
}
function skipSettings() {
	step.value++;
}
function finish() {
	if (token == null) return;
	login(token);
}

return (_ctx: any,_cache: any) => {
  const _component_MkEllipsis = _resolveComponent("MkEllipsis")
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_PageWithAnimBg = _resolveComponent("PageWithAnimBg")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createBlock(_component_PageWithAnimBg, null, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.formContainer)
        }, [
          _createElementVNode("div", {
            class: _normalizeClass(["_panel", _ctx.$style.form])
          }, [
            _createElementVNode("svg", {
              xmlns: "http://www.w3.org/2000/svg",
              "xmlns:xlink": "http://www.w3.org/1999/xlink",
              style: "z-index:1;position:relative",
              viewBox: "0 0 854 300"
            }, [
              _createElementVNode("defs", null, [
                _createElementVNode("linearGradient", {
                  id: "linear",
                  x1: "0%",
                  y1: "0%",
                  x2: "100%",
                  y2: "0%"
                }, [
                  _hoisted_1,
                  _hoisted_2
                ])
              ]),
              _createElementVNode("g", { transform: "translate(427, 150) scale(1, 1) translate(-427, -150)" }, [
                _createElementVNode("path", {
                  d: "",
                  fill: "url(#linear)",
                  opacity: "0.4"
                }, [
                  _hoisted_3
                ]),
                _createElementVNode("path", {
                  d: "",
                  fill: "url(#linear)",
                  opacity: "0.4"
                }, [
                  _hoisted_4
                ])
              ])
            ]),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.title)
            }, [
              _hoisted_5,
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.version)
              }, "v" + _toDisplayString(_unref(version)), 1 /* TEXT */)
            ]),
            _createElementVNode("div", { style: "padding: 16px 32px 32px 32px;" }, [
              (!accountCreated.value)
                ? (_openBlock(), _createElementBlock("form", {
                  key: 0,
                  class: "_gaps_m",
                  onSubmit: _cache[0] || (_cache[0] = _withModifiers(($event: any) => (createAccount()), ["prevent"]))
                }, [
                  _createElementVNode("div", {
                    style: "text-align: center;",
                    class: "_gaps_s"
                  }, [
                    _createElementVNode("div", null, [
                      _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._serverSetupWizard.installCompleted), 1 /* TEXT */)
                    ]),
                    _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._serverSetupWizard.firstCreateAccount), 1 /* TEXT */)
                  ]),
                  _createVNode(MkInput, {
                    type: "password",
                    "data-cy-admin-initial-password": "",
                    modelValue: setupPassword.value,
                    "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((setupPassword).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.initialPasswordForSetup), 1 /* TEXT */),
                      _createTextVNode(" "),
                      _createElementVNode("div", { class: "_button _help" }, [
                        _hoisted_6
                      ])
                    ]),
                    prefix: _withCtx(() => [
                      _hoisted_7
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["modelValue"]),
                  _createVNode(MkInput, {
                    pattern: "^[a-zA-Z0-9_]{1,20}$",
                    spellcheck: false,
                    required: "",
                    "data-cy-admin-username": "",
                    modelValue: username.value,
                    "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((username).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.username), 1 /* TEXT */),
                      _createTextVNode(" "),
                      _createElementVNode("div", { class: "_button _help" }, [
                        _hoisted_8
                      ])
                    ]),
                    prefix: _withCtx(() => [
                      _createTextVNode("@")
                    ]),
                    suffix: _withCtx(() => [
                      _createTextVNode("@" + _toDisplayString(_unref(host)), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["spellcheck", "modelValue"]),
                  _createVNode(MkInput, {
                    type: "password",
                    "data-cy-admin-password": "",
                    modelValue: password.value,
                    "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((password).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.password), 1 /* TEXT */)
                    ]),
                    prefix: _withCtx(() => [
                      _hoisted_9
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["modelValue"]),
                  _createElementVNode("div", null, [
                    _createVNode(MkButton, {
                      gradate: "",
                      large: "",
                      rounded: "",
                      disabled: accountCreating.value,
                      "data-cy-admin-ok": "",
                      style: "margin: 0 auto;",
                      type: "submit"
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(accountCreating.value ? _unref(i18n).ts.processing : _unref(i18n).ts.next), 1 /* TEXT */),
                        (accountCreating.value)
                          ? (_openBlock(), _createBlock(_component_MkEllipsis, { key: 0 }))
                          : _createCommentVNode("v-if", true)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled"])
                  ])
                ]))
                : (step.value === 0)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 1,
                    class: "_gaps_m"
                  }, [
                    _createElementVNode("div", {
                      style: "text-align: center;",
                      class: "_gaps_s"
                    }, [
                      _createElementVNode("div", null, [
                        _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._serverSetupWizard.accountCreated), 1 /* TEXT */)
                      ])
                    ]),
                    _createVNode(MkButton, {
                      gradate: "",
                      large: "",
                      rounded: "",
                      "data-cy-next": "",
                      style: "margin: 0 auto;",
                      onClick: _cache[4] || (_cache[4] = ($event: any) => (step.value++))
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.next), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]))
                : (step.value === 1)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 2,
                    class: "_gaps_m"
                  }, [
                    _createElementVNode("div", {
                      style: "text-align: center;",
                      class: "_gaps_s"
                    }, [
                      _createElementVNode("div", { style: "font-size: 120%;" }, [
                        _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._serverSetupWizard.serverSetting), 1 /* TEXT */)
                      ]),
                      _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._serverSetupWizard.youCanEasilyConfigureOptimalServerSettingsWithThisWizard), 1 /* TEXT */),
                      _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._serverSetupWizard.settingsYouMakeHereCanBeChangedLater), 1 /* TEXT */)
                    ]),
                    _createVNode(_Suspense, null, {
                      default: _withCtx(() => [
                        _createVNode(MkServerSetupWizard, {
                          token: _unref(token),
                          onFinished: onWizardFinished
                        }, null, 8 /* PROPS */, ["token"])
                      ]),
                      fallback: _withCtx(() => [
                        _createVNode(_component_MkLoading)
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(MkButton, {
                      rounded: "",
                      style: "margin: 0 auto;",
                      onClick: skipSettings
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard.skipSettings), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]))
                : (step.value === 2)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 3,
                    class: "_gaps_m"
                  }, [
                    _createElementVNode("div", {
                      style: "text-align: center;",
                      class: "_gaps_s"
                    }, [
                      _createElementVNode("div", null, [
                        _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._serverSetupWizard.settingsCompleted), 1 /* TEXT */)
                      ]),
                      _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._serverSetupWizard.settingsCompleted_description), 1 /* TEXT */),
                      _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._serverSetupWizard.settingsCompleted_description2), 1 /* TEXT */)
                    ]),
                    _createElementVNode("div", {
                      class: _normalizeClass(["_gaps_s", _ctx.$style.donation])
                    }, [
                      _createElementVNode("div", null, [
                        _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._serverSetupWizard.donationRequest), 1 /* TEXT */)
                      ]),
                      _createElementVNode("div", null, [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard._donationRequest.text1), 1 /* TEXT */),
                        _hoisted_10,
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard._donationRequest.text2), 1 /* TEXT */),
                        _hoisted_11,
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard._donationRequest.text3), 1 /* TEXT */)
                      ]),
                      _createVNode(MkLink, {
                        target: "_blank",
                        url: "https://misskey-hub.net/docs/donate/",
                        style: "margin: 0 auto;"
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.learnMore), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      })
                    ]),
                    _createElementVNode("div", { class: "_buttonsCenter" }, [
                      _createVNode(MkButton, {
                        gradate: "",
                        large: "",
                        rounded: "",
                        "data-cy-next": "",
                        style: "margin: 0 auto;",
                        onClick: finish
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.start), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      })
                    ])
                  ]))
                : _createCommentVNode("v-if", true)
            ])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
