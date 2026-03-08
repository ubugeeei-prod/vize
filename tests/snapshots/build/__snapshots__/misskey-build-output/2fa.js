import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-shield-lock" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-help-circle" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-key" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-forms" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
import { defineAsyncComponent, computed } from 'vue'
import { supported as webAuthnSupported, create as webAuthnCreate, parseCreationOptionsFromJSON } from '@github/webauthn-json/browser-ponyfill'
import * as Misskey from 'misskey-js'
import MkButton from '@/components/MkButton.vue'
import MkInfo from '@/components/MkInfo.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import FormSection from '@/components/form/section.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkLink from '@/components/MkLink.vue'
import * as os from '@/os.js'
import { ensureSignin } from '@/i.js'
import { i18n } from '@/i18n.js'
import { updateCurrentAccountPartial } from '@/accounts.js'

export default /*@__PURE__*/_defineComponent({
  __name: '2fa',
  props: {
    first: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {

const $i = ensureSignin();
// メモ: 各エンドポイントはmeUpdatedを発行するため、refreshAccountは不要
const usePasswordLessLogin = computed(() => $i.usePasswordLessLogin ?? false);
async function registerTOTP(): Promise<void> {
	const auth = await os.authenticateDialog();
	if (auth.canceled) return;
	const twoFactorData = await os.apiWithDialog('i/2fa/register', {
		password: auth.result.password,
		token: auth.result.token,
	});
	const { dispose } = await os.popupAsyncWithDialog(import('./2fa.qrdialog.vue').then(x => x.default), {
		twoFactorData,
	}, {
		closed: () => dispose(),
	});
}
async function unregisterTOTP(): Promise<void> {
	const auth = await os.authenticateDialog();
	if (auth.canceled) return;
	os.apiWithDialog('i/2fa/unregister', {
		password: auth.result.password,
		token: auth.result.token,
	}).then(res => {
		updateCurrentAccountPartial({
			twoFactorEnabled: false,
		});
	}).catch(error => {
		os.alert({
			type: 'error',
			text: error,
		});
	});
}
function renewTOTP(): void {
	os.confirm({
		type: 'question',
		title: i18n.ts._2fa.renewTOTP,
		text: i18n.ts._2fa.renewTOTPConfirm,
		okText: i18n.ts._2fa.renewTOTPOk,
		cancelText: i18n.ts._2fa.renewTOTPCancel,
	}).then(({ canceled }) => {
		if (canceled) return;
		registerTOTP();
	});
}
async function unregisterKey(key: NonNullable<Misskey.entities.MeDetailedOnly['securityKeysList']>[number]) {
	const confirm = await os.confirm({
		type: 'question',
		title: i18n.ts._2fa.removeKey,
		text: i18n.tsx._2fa.removeKeyConfirm({ name: key.name }),
	});
	if (confirm.canceled) return;
	const auth = await os.authenticateDialog();
	if (auth.canceled) return;
	await os.apiWithDialog('i/2fa/remove-key', {
		password: auth.result.password,
		token: auth.result.token,
		credentialId: key.id,
	});
	os.success();
}
async function renameKey(key: NonNullable<Misskey.entities.MeDetailedOnly['securityKeysList']>[number]) {
	const name = await os.inputText({
		title: i18n.ts.rename,
		default: key.name,
		type: 'text',
		minLength: 1,
		maxLength: 30,
	});
	if (name.canceled) return;
	await os.apiWithDialog('i/2fa/update-key', {
		name: name.result,
		credentialId: key.id,
	});
}
async function addSecurityKey() {
	const auth = await os.authenticateDialog();
	if (auth.canceled) return;
	const registrationOptions = parseCreationOptionsFromJSON({
		// @ts-expect-error misskey-js側に型がない
		publicKey: await os.apiWithDialog('i/2fa/register-key', {
			password: auth.result.password,
			token: auth.result.token,
		}),
	});
	const name = await os.inputText({
		title: i18n.ts._2fa.registerSecurityKey,
		text: i18n.ts._2fa.securityKeyName,
		type: 'text',
		minLength: 1,
		maxLength: 30,
	});
	if (name.canceled) return;
	const credential = await os.promiseDialog(
		webAuthnCreate(registrationOptions),
		null,
		() => {}, // ユーザーのキャンセルはrejectなのでエラーダイアログを出さない
		i18n.ts._2fa.tapSecurityKey,
	);
	if (!credential) return;
	const auth2 = await os.authenticateDialog();
	if (auth2.canceled) return;
	await os.apiWithDialog('i/2fa/key-done', {
		password: auth.result.password,
		token: auth.result.token,
		name: name.result,
		// @ts-expect-error misskey-js側に型がない
		credential: credential.toJSON(),
	});
}
async function updatePasswordLessLogin(value: boolean) {
	await os.apiWithDialog('i/2fa/password-less', {
		value,
	});
}

return (_ctx: any,_cache: any) => {
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_SearchText = _resolveComponent("SearchText")
  const _component_SearchMarker = _resolveComponent("SearchMarker")
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_I18n = _resolveComponent("I18n")

  return (_openBlock(), _createBlock(_component_SearchMarker, {
      markerId: "2fa",
      keywords: ['2fa']
    }, {
      default: _withCtx(() => [
        _createVNode(FormSection, { first: __props.first }, {
          label: _withCtx(() => [
            _createVNode(_component_SearchLabel, null, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts['2fa']), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            })
          ]),
          default: _withCtx(() => [
            (_unref($i))
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: "_gaps_s"
              }, [
                (_unref($i).twoFactorEnabled && _unref($i).twoFactorBackupCodesStock === 'partial')
                  ? (_openBlock(), _createBlock(MkInfo, {
                    key: 0,
                    warn: ""
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._2fa.backupCodeUsedWarning), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }))
                  : _createCommentVNode("v-if", true),
                (_unref($i).twoFactorEnabled && _unref($i).twoFactorBackupCodesStock === 'none')
                  ? (_openBlock(), _createBlock(MkInfo, {
                    key: 0,
                    warn: ""
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._2fa.backupCodesExhaustedWarning), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }))
                  : _createCommentVNode("v-if", true),
                _createVNode(_component_SearchMarker, { keywords: ['totp', 'app'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkFolder, { defaultOpen: true }, {
                      icon: _withCtx(() => [
                        _hoisted_1
                      ]),
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.totp), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      caption: _withCtx(() => [
                        _createVNode(_component_SearchText, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.totpDescription), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      suffix: _withCtx(() => [
                        (_unref($i).twoFactorEnabled)
                          ? (_openBlock(), _createElementBlock("i", {
                            key: 0,
                            class: "ti ti-check",
                            style: "color: var(--MI_THEME-success)"
                          }))
                          : _createCommentVNode("v-if", true)
                      ]),
                      default: _withCtx(() => [
                        (_unref($i).twoFactorEnabled)
                          ? (_openBlock(), _createElementBlock("div", {
                            key: 0,
                            class: "_gaps_s"
                          }, [
                            _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._2fa.alreadyRegistered), 1 /* TEXT */),
                            (_unref($i).securityKeysList.length > 0)
                              ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                                _createVNode(MkButton, { onClick: renewTOTP }, {
                                  default: _withCtx(() => [
                                    _createTextVNode(_toDisplayString(_unref(i18n).ts._2fa.renewTOTP), 1 /* TEXT */)
                                  ]),
                                  _: 1 /* STABLE */
                                }),
                                _createVNode(MkInfo, null, {
                                  default: _withCtx(() => [
                                    _createTextVNode(_toDisplayString(_unref(i18n).ts._2fa.whyTOTPOnlyRenew), 1 /* TEXT */)
                                  ]),
                                  _: 1 /* STABLE */
                                })
                              ], 64 /* STABLE_FRAGMENT */))
                              : (_openBlock(), _createBlock(MkButton, {
                                key: 1,
                                danger: "",
                                onClick: unregisterTOTP
                              }, {
                                default: _withCtx(() => [
                                  _createTextVNode(_toDisplayString(_unref(i18n).ts.unregister), 1 /* TEXT */)
                                ]),
                                _: 1 /* STABLE */
                              }))
                          ]))
                          : (!_unref($i).twoFactorEnabled)
                            ? (_openBlock(), _createElementBlock("div", {
                              key: 1,
                              class: "_gaps_s"
                            }, [
                              _createVNode(MkButton, {
                                primary: "",
                                gradate: "",
                                onClick: registerTOTP
                              }, {
                                default: _withCtx(() => [
                                  _createTextVNode(_toDisplayString(_unref(i18n).ts._2fa.registerTOTP), 1 /* TEXT */)
                                ]),
                                _: 1 /* STABLE */
                              }),
                              _createVNode(MkLink, {
                                url: "https://misskey-hub.net/docs/for-users/stepped-guides/how-to-enable-2fa/",
                                target: "_blank"
                              }, {
                                default: _withCtx(() => [
                                  _hoisted_2,
                                  _createTextVNode(" "),
                                  _createTextVNode(_toDisplayString(_unref(i18n).ts.learnMore), 1 /* TEXT */)
                                ]),
                                _: 1 /* STABLE */
                              })
                            ]))
                          : _createCommentVNode("v-if", true)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["defaultOpen"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(_component_SearchMarker, { keywords: ['security', 'key', 'passkey'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkFolder, null, {
                      icon: _withCtx(() => [
                        _hoisted_3
                      ]),
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.securityKeyAndPasskey), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      default: _withCtx(() => [
                        _createElementVNode("div", { class: "_gaps_s" }, [
                          _createVNode(MkInfo, null, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._2fa.securityKeyInfo), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }),
                          (!_unref(webAuthnSupported)())
                            ? (_openBlock(), _createBlock(MkInfo, {
                              key: 0,
                              warn: ""
                            }, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts._2fa.securityKeyNotSupported), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            }))
                            : (_unref(webAuthnSupported)() && !_unref($i).twoFactorEnabled)
                              ? (_openBlock(), _createBlock(MkInfo, {
                                key: 1,
                                warn: ""
                              }, {
                                default: _withCtx(() => [
                                  _createTextVNode(_toDisplayString(_unref(i18n).ts._2fa.registerTOTPBeforeKey), 1 /* TEXT */)
                                ]),
                                _: 1 /* STABLE */
                              }))
                            : (_openBlock(), _createElementBlock(_Fragment, { key: 2 }, [
                              _createVNode(MkButton, {
                                primary: "",
                                onClick: addSecurityKey
                              }, {
                                default: _withCtx(() => [
                                  _createTextVNode(_toDisplayString(_unref(i18n).ts._2fa.registerSecurityKey), 1 /* TEXT */)
                                ]),
                                _: 1 /* STABLE */
                              }),
                              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref($i).securityKeysList, (key) => {
                                return (_openBlock(), _createBlock(MkFolder, { key: key.id }, {
                                  label: _withCtx(() => [
                                    _createTextVNode(_toDisplayString(key.name), 1 /* TEXT */)
                                  ]),
                                  suffix: _withCtx(() => [
                                    _createVNode(_component_I18n, { src: _unref(i18n).ts.lastUsedAt }, {
                                      t: _withCtx(() => [
                                        _createVNode(_component_MkTime, { time: key.lastUsed }, null, 8 /* PROPS */, ["time"])
                                      ]),
                                      _: 2 /* DYNAMIC */
                                    }, 8 /* PROPS */, ["src"])
                                  ]),
                                  default: _withCtx(() => [
                                    _createElementVNode("div", { class: "_buttons" }, [
                                      _createVNode(MkButton, {
                                        onClick: _cache[0] || (_cache[0] = ($event: any) => (renameKey(key)))
                                      }, {
                                        default: _withCtx(() => [
                                          _hoisted_4,
                                          _createTextVNode(" "),
                                          _createTextVNode(_toDisplayString(_unref(i18n).ts.rename), 1 /* TEXT */)
                                        ]),
                                        _: 2 /* DYNAMIC */
                                      }),
                                      _createVNode(MkButton, {
                                        danger: "",
                                        onClick: _cache[1] || (_cache[1] = ($event: any) => (unregisterKey(key)))
                                      }, {
                                        default: _withCtx(() => [
                                          _hoisted_5,
                                          _createTextVNode(" "),
                                          _createTextVNode(_toDisplayString(_unref(i18n).ts.unregister), 1 /* TEXT */)
                                        ]),
                                        _: 2 /* DYNAMIC */
                                      })
                                    ])
                                  ]),
                                  _: 2 /* DYNAMIC */
                                }, 1024 /* DYNAMIC_SLOTS */))
                              }), 128 /* KEYED_FRAGMENT */))
                            ], 64 /* STABLE_FRAGMENT */))
                        ])
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(_component_SearchMarker, { keywords: ['password', 'less', 'key', 'passkey', 'login', 'signin'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkSwitch, {
                      disabled: !_unref($i).twoFactorEnabled || _unref($i).securityKeysList.length === 0,
                      modelValue: usePasswordLessLogin.value,
                      "onUpdate:modelValue": _cache[2] || (_cache[2] = v => updatePasswordLessLogin(v))
                    }, {
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.passwordLessLogin), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      caption: _withCtx(() => [
                        _createVNode(_component_SearchText, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.passwordLessLoginDescription), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled", "modelValue"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"])
              ]))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["first"])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["keywords"]))
}
}

})
