import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("a", { href: "https://authy.com/", rel: "noopener", target: "_blank", class: "_link" }, "Authy")
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("a", { href: "https://support.google.com/accounts/answer/1066447", rel: "noopener", target: "_blank", class: "_link" }, "Google Authenticator")
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-left" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_6 = { style: "text-align: center;" }
const _hoisted_7 = { style: "text-align: center;" }
const _hoisted_8 = { style: "text-align: center; font-weight: bold;" }
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-key" })
const _hoisted_10 = { class: "_monospace" }
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
import { hostname, port } from '@@/js/config'
import { useTemplateRef, ref } from 'vue'
import MkButton from '@/components/MkButton.vue'
import MkModalWindow from '@/components/MkModalWindow.vue'
import MkKeyValue from '@/components/MkKeyValue.vue'
import MkInput from '@/components/MkInput.vue'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'
import MkFolder from '@/components/MkFolder.vue'
import MkInfo from '@/components/MkInfo.vue'
import MkLink from '@/components/MkLink.vue'
import { confetti } from '@/utility/confetti.js'
import { ensureSignin } from '@/i.js'

export default /*@__PURE__*/_defineComponent({
  __name: '2fa.qrdialog',
  props: {
    twoFactorData: { type: Object, required: true }
  },
  emits: ["closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const $i = ensureSignin();
const dialog = useTemplateRef('dialog');
const page = ref(0);
const token = ref<string | null>(null);
const backupCodes = ref<string[]>();
function cancel() {
	dialog.value?.close();
}
async function tokenDone() {
	if (token.value == null) return;
	const res = await os.apiWithDialog('i/2fa/done', {
		token: token.value.toString(), // 実装ミスなどでnumberが入る可能性を払拭できないため念のためtoString
	});
	backupCodes.value = res.backupCodes;
	page.value++;
	confetti({
		duration: 1000 * 3,
	});
}
function downloadBackupCodes() {
	if (backupCodes.value !== undefined) {
		const txtBlob = new Blob([backupCodes.value.join('\n')], { type: 'text/plain' });
		const dummya = window.document.createElement('a');
		dummya.href = URL.createObjectURL(txtBlob);
		dummya.download = `${$i.username}@${hostname}` + (port !== '' ? `_${port}` : '') + '-2fa-backup-codes.txt';
		dummya.click();
	}
}
function allDone() {
	dialog.value?.close();
}

return (_ctx: any,_cache: any) => {
  const _component_I18n = _resolveComponent("I18n")

  return (_openBlock(), _createBlock(MkModalWindow, {
      ref_key: "dialog", ref: dialog,
      width: 500,
      height: 550,
      onClose: cancel,
      onClosed: _cache[0] || (_cache[0] = ($event: any) => (emit('closed')))
    }, {
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(i18n).ts.setupOf2fa), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", { style: "overflow-x: clip;" }, [
          _createVNode(_Transition, {
            mode: "out-in",
            enterActiveClass: _ctx.$style.transition_x_enterActive,
            leaveActiveClass: _ctx.$style.transition_x_leaveActive,
            enterFromClass: _ctx.$style.transition_x_enterFrom,
            leaveToClass: _ctx.$style.transition_x_leaveTo
          }, {
            default: _withCtx(() => [
              (page.value === 0)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  style: "height: 100cqh; overflow: auto; text-align: center;"
                }, [
                  _createElementVNode("div", {
                    class: "_spacer",
                    style: "--MI_SPACER-min: 20px; --MI_SPACER-max: 28px;"
                  }, [
                    _createElementVNode("div", { class: "_gaps" }, [
                      _createVNode(MkInfo, null, {
                        default: _withCtx(() => [
                          _createVNode(MkLink, {
                            url: "https://misskey-hub.net/docs/for-users/stepped-guides/how-to-enable-2fa/",
                            target: "_blank"
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._2fa.moreDetailedGuideHere), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]),
                        _: 1 /* STABLE */
                      }),
                      _createVNode(_component_I18n, {
                        src: _unref(i18n).ts._2fa.step1,
                        tag: "div"
                      }, {
                        a: _withCtx(() => [
                          _hoisted_1
                        ]),
                        b: _withCtx(() => [
                          _hoisted_2
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["src"]),
                      _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._2fa.step2), 1 /* TEXT */),
                      _createElementVNode("div", null, [
                        _createElementVNode("a", {
                          class: _normalizeClass(_ctx.$style.qrRoot),
                          href: __props.twoFactorData.url
                        }, [
                          _createElementVNode("img", {
                            class: _normalizeClass(_ctx.$style.qr),
                            src: __props.twoFactorData.qr
                          }, null, 8 /* PROPS */, ["src"])
                        ], 8 /* PROPS */, ["href"]),
                        _createTextVNode("\n\t\t\t\t\t\t\t\t" + "\n\t\t\t\t\t\t\t\t"),
                        _createElementVNode("div", null, [
                          _createVNode(MkButton, {
                            inline: "",
                            rounded: "",
                            link: "",
                            to: __props.twoFactorData.url,
                            linkBehavior: 'browser'
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.launchApp), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["to", "linkBehavior"])
                        ])
                      ]),
                      _createVNode(MkKeyValue, { copy: __props.twoFactorData.url }, {
                        key: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts._2fa.step2Uri), 1 /* TEXT */)
                        ]),
                        value: _withCtx(() => [
                          _createTextVNode(_toDisplayString(__props.twoFactorData.url), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["copy"])
                    ]),
                    _createElementVNode("div", {
                      class: "_buttonsCenter",
                      style: "margin-top: 16px;"
                    }, [
                      _createVNode(MkButton, {
                        rounded: "",
                        onClick: cancel
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.cancel), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }),
                      _createVNode(MkButton, {
                        primary: "",
                        rounded: "",
                        gradate: "",
                        onClick: _cache[1] || (_cache[1] = ($event: any) => (page.value++))
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.continue), 1 /* TEXT */),
                          _createTextVNode(" "),
                          _hoisted_3
                        ]),
                        _: 1 /* STABLE */
                      })
                    ])
                  ])
                ]))
                : (page.value === 1)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 1,
                    style: "height: 100cqh; overflow: auto;"
                  }, [
                    _createElementVNode("div", {
                      class: "_spacer",
                      style: "--MI_SPACER-min: 20px; --MI_SPACER-max: 28px;"
                    }, [
                      _createElementVNode("div", { class: "_gaps" }, [
                        _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._2fa.step3Title), 1 /* TEXT */),
                        _createVNode(MkInput, {
                          autocomplete: "one-time-code",
                          inputmode: "numeric",
                          modelValue: token.value,
                          "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((token).value = $event))
                        }, null, 8 /* PROPS */, ["modelValue"]),
                        _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._2fa.step3), 1 /* TEXT */)
                      ]),
                      _createElementVNode("div", {
                        class: "_buttonsCenter",
                        style: "margin-top: 16px;"
                      }, [
                        _createVNode(MkButton, {
                          rounded: "",
                          onClick: _cache[3] || (_cache[3] = ($event: any) => (page.value--))
                        }, {
                          default: _withCtx(() => [
                            _hoisted_4,
                            _createTextVNode(" "),
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.goBack), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }),
                        _createVNode(MkButton, {
                          primary: "",
                          rounded: "",
                          gradate: "",
                          onClick: tokenDone
                        }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.continue), 1 /* TEXT */),
                            _createTextVNode(" "),
                            _hoisted_5
                          ]),
                          _: 1 /* STABLE */
                        })
                      ])
                    ])
                  ]))
                : (page.value === 2)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 2,
                    style: "height: 100cqh; overflow: auto;"
                  }, [
                    _createElementVNode("div", {
                      class: "_spacer",
                      style: "--MI_SPACER-min: 20px; --MI_SPACER-max: 28px;"
                    }, [
                      _createElementVNode("div", { class: "_gaps" }, [
                        _createElementVNode("div", _hoisted_6, _toDisplayString(_unref(i18n).ts._2fa.setupCompleted) + "🎉", 1 /* TEXT */),
                        _createElementVNode("div", _hoisted_7, _toDisplayString(_unref(i18n).ts._2fa.step4), 1 /* TEXT */),
                        _createElementVNode("div", _hoisted_8, _toDisplayString(_unref(i18n).ts._2fa.checkBackupCodesBeforeCloseThisWizard), 1 /* TEXT */),
                        _createVNode(MkFolder, { defaultOpen: true }, {
                          icon: _withCtx(() => [
                            _hoisted_9
                          ]),
                          label: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts._2fa.backupCodes), 1 /* TEXT */)
                          ]),
                          default: _withCtx(() => [
                            _createElementVNode("div", { class: "_gaps" }, [
                              _createVNode(MkInfo, { warn: "" }, {
                                default: _withCtx(() => [
                                  _createTextVNode(_toDisplayString(_unref(i18n).ts._2fa.backupCodesDescription), 1 /* TEXT */)
                                ]),
                                _: 1 /* STABLE */
                              }),
                              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(backupCodes.value, (code, i) => {
                                return (_openBlock(), _createElementBlock("div", {
                                  key: code,
                                  class: "_gaps_s"
                                }, [
                                  _createVNode(MkKeyValue, { copy: code }, {
                                    key: _withCtx(() => [
                                      _createTextVNode("#" + _toDisplayString(i + 1), 1 /* TEXT */)
                                    ]),
                                    value: _withCtx(() => [
                                      _createElementVNode("code", _hoisted_10, _toDisplayString(code), 1 /* TEXT */)
                                    ]),
                                    _: 2 /* DYNAMIC */
                                  }, 8 /* PROPS */, ["copy"])
                                ]))
                              }), 128 /* KEYED_FRAGMENT */)),
                              _createVNode(MkButton, {
                                primary: "",
                                rounded: "",
                                gradate: "",
                                onClick: downloadBackupCodes
                              }, {
                                default: _withCtx(() => [
                                  _hoisted_11,
                                  _createTextVNode(" "),
                                  _createTextVNode(_toDisplayString(_unref(i18n).ts.download), 1 /* TEXT */)
                                ]),
                                _: 1 /* STABLE */
                              })
                            ])
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["defaultOpen"])
                      ]),
                      _createElementVNode("div", {
                        class: "_buttonsCenter",
                        style: "margin-top: 16px;"
                      }, [
                        _createVNode(MkButton, {
                          primary: "",
                          rounded: "",
                          gradate: "",
                          onClick: allDone
                        }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.done), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ])
                    ])
                  ]))
                : _createCommentVNode("v-if", true)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass"])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["width", "height"]))
}
}

})
