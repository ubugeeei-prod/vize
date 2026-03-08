import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-send" })
import { ref, computed } from 'vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkInput from '@/components/MkInput.vue'
import FormInfo from '@/components/MkInfo.vue'
import FormSplit from '@/components/form/split.vue'
import FormSection from '@/components/form/section.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { fetchInstance, instance } from '@/instance.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import MkButton from '@/components/MkButton.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'email-settings',
  async setup(__props) {

let __temp: any, __restore: any

const meta = await misskeyApi('admin/meta');
const enableEmail = ref(meta.enableEmail);
const email = ref(meta.email);
const smtpSecure = ref(meta.smtpSecure);
const smtpHost = ref(meta.smtpHost);
const smtpPort = ref(meta.smtpPort);
const smtpUser = ref(meta.smtpUser);
const smtpPass = ref(meta.smtpPass);
async function testEmail() {
	const { canceled, result: destination } = await os.inputText({
		title: 'To',
		type: 'email',
		default: instance.maintainerEmail ?? '',
		placeholder: 'test@example.com',
		minLength: 1,
	});
	if (canceled) return;
	os.apiWithDialog('admin/send-email', {
		to: destination,
		subject: 'Test email',
		text: 'Yo',
	});
}
function save() {
	os.apiWithDialog('admin/update-meta', {
		enableEmail: enableEmail.value,
		email: email.value,
		smtpSecure: smtpSecure.value,
		smtpHost: smtpHost.value,
		smtpPort: smtpPort.value,
		smtpUser: smtpUser.value,
		smtpPass: smtpPass.value,
	}).then(() => {
		fetchInstance(true);
	});
}
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.emailServer,
	icon: 'ti ti-mail',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_SearchText = _resolveComponent("SearchText")
  const _component_SearchMarker = _resolveComponent("SearchMarker")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, { tabs: headerTabs.value }, {
      footer: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.footer)
        }, [
          _createElementVNode("div", {
            class: "_spacer",
            style: "--MI_SPACER-w: 700px; --MI_SPACER-min: 16px; --MI_SPACER-max: 16px;"
          }, [
            _createElementVNode("div", { class: "_buttons" }, [
              _createVNode(MkButton, {
                primary: "",
                rounded: "",
                onClick: save
              }, {
                default: _withCtx(() => [
                  _hoisted_1,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }),
              _createVNode(MkButton, {
                rounded: "",
                onClick: testEmail
              }, {
                default: _withCtx(() => [
                  _hoisted_2,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.testEmail), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ])
          ])
        ])
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 700px; --MI_SPACER-min: 16px; --MI_SPACER-max: 32px;"
        }, [
          _createVNode(_component_SearchMarker, {
            path: "/admin/email-settings",
            label: _unref(i18n).ts.emailServer,
            keywords: ['email'],
            icon: "ti ti-mail"
          }, {
            default: _withCtx(() => [
              _createElementVNode("div", { class: "_gaps_m" }, [
                _createVNode(_component_SearchMarker, null, {
                  default: _withCtx(() => [
                    _createVNode(MkSwitch, {
                      modelValue: enableEmail.value,
                      "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((enableEmail).value = $event))
                    }, {
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.enableEmail), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }),
                        _createTextVNode(" ("),
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.recommended), 1 /* TEXT */),
                        _createTextVNode(")")
                      ]),
                      caption: _withCtx(() => [
                        _createVNode(_component_SearchText, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.emailConfigInfo), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["modelValue"])
                  ]),
                  _: 1 /* STABLE */
                }),
                (enableEmail.value)
                  ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                    _createVNode(_component_SearchMarker, null, {
                      default: _withCtx(() => [
                        _createVNode(MkInput, {
                          type: "email",
                          modelValue: email.value,
                          "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((email).value = $event))
                        }, {
                          label: _withCtx(() => [
                            _createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.emailAddress), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["modelValue"])
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(_component_SearchMarker, null, {
                      default: _withCtx(() => [
                        _createVNode(FormSection, null, {
                          label: _withCtx(() => [
                            _createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.smtpConfig), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          default: _withCtx(() => [
                            _createElementVNode("div", { class: "_gaps_m" }, [
                              _createVNode(FormSplit, { minWidth: 280 }, {
                                default: _withCtx(() => [
                                  _createVNode(_component_SearchMarker, null, {
                                    default: _withCtx(() => [
                                      _createVNode(MkInput, {
                                        modelValue: smtpHost.value,
                                        "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((smtpHost).value = $event))
                                      }, {
                                        label: _withCtx(() => [
                                          _createVNode(_component_SearchLabel, null, {
                                            default: _withCtx(() => [
                                              _createTextVNode(_toDisplayString(_unref(i18n).ts.smtpHost), 1 /* TEXT */)
                                            ]),
                                            _: 1 /* STABLE */
                                          })
                                        ]),
                                        _: 1 /* STABLE */
                                      }, 8 /* PROPS */, ["modelValue"])
                                    ]),
                                    _: 1 /* STABLE */
                                  }),
                                  _createVNode(_component_SearchMarker, null, {
                                    default: _withCtx(() => [
                                      _createVNode(MkInput, {
                                        type: "number",
                                        modelValue: smtpPort.value,
                                        "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((smtpPort).value = $event))
                                      }, {
                                        label: _withCtx(() => [
                                          _createVNode(_component_SearchLabel, null, {
                                            default: _withCtx(() => [
                                              _createTextVNode(_toDisplayString(_unref(i18n).ts.smtpPort), 1 /* TEXT */)
                                            ]),
                                            _: 1 /* STABLE */
                                          })
                                        ]),
                                        _: 1 /* STABLE */
                                      }, 8 /* PROPS */, ["modelValue"])
                                    ]),
                                    _: 1 /* STABLE */
                                  })
                                ]),
                                _: 1 /* STABLE */
                              }, 8 /* PROPS */, ["minWidth"]),
                              _createVNode(FormSplit, { minWidth: 280 }, {
                                default: _withCtx(() => [
                                  _createVNode(_component_SearchMarker, null, {
                                    default: _withCtx(() => [
                                      _createVNode(MkInput, {
                                        modelValue: smtpUser.value,
                                        "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((smtpUser).value = $event))
                                      }, {
                                        label: _withCtx(() => [
                                          _createVNode(_component_SearchLabel, null, {
                                            default: _withCtx(() => [
                                              _createTextVNode(_toDisplayString(_unref(i18n).ts.smtpUser), 1 /* TEXT */)
                                            ]),
                                            _: 1 /* STABLE */
                                          })
                                        ]),
                                        _: 1 /* STABLE */
                                      }, 8 /* PROPS */, ["modelValue"])
                                    ]),
                                    _: 1 /* STABLE */
                                  }),
                                  _createVNode(_component_SearchMarker, null, {
                                    default: _withCtx(() => [
                                      _createVNode(MkInput, {
                                        type: "password",
                                        modelValue: smtpPass.value,
                                        "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((smtpPass).value = $event))
                                      }, {
                                        label: _withCtx(() => [
                                          _createVNode(_component_SearchLabel, null, {
                                            default: _withCtx(() => [
                                              _createTextVNode(_toDisplayString(_unref(i18n).ts.smtpPass), 1 /* TEXT */)
                                            ]),
                                            _: 1 /* STABLE */
                                          })
                                        ]),
                                        _: 1 /* STABLE */
                                      }, 8 /* PROPS */, ["modelValue"])
                                    ]),
                                    _: 1 /* STABLE */
                                  })
                                ]),
                                _: 1 /* STABLE */
                              }, 8 /* PROPS */, ["minWidth"]),
                              _createVNode(FormInfo, null, {
                                default: _withCtx(() => [
                                  _createTextVNode(_toDisplayString(_unref(i18n).ts.emptyToDisableSmtpAuth), 1 /* TEXT */)
                                ]),
                                _: 1 /* STABLE */
                              }),
                              _createVNode(_component_SearchMarker, null, {
                                default: _withCtx(() => [
                                  _createVNode(MkSwitch, {
                                    modelValue: smtpSecure.value,
                                    "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((smtpSecure).value = $event))
                                  }, {
                                    label: _withCtx(() => [
                                      _createVNode(_component_SearchLabel, null, {
                                        default: _withCtx(() => [
                                          _createTextVNode(_toDisplayString(_unref(i18n).ts.smtpSecure), 1 /* TEXT */)
                                        ]),
                                        _: 1 /* STABLE */
                                      })
                                    ]),
                                    caption: _withCtx(() => [
                                      _createVNode(_component_SearchText, null, {
                                        default: _withCtx(() => [
                                          _createTextVNode(_toDisplayString(_unref(i18n).ts.smtpSecureInfo), 1 /* TEXT */)
                                        ]),
                                        _: 1 /* STABLE */
                                      })
                                    ]),
                                    _: 1 /* STABLE */
                                  }, 8 /* PROPS */, ["modelValue"])
                                ]),
                                _: 1 /* STABLE */
                              })
                            ])
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _: 1 /* STABLE */
                    })
                  ], 64 /* STABLE_FRAGMENT */))
                  : _createCommentVNode("v-if", true)
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
