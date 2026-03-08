import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-checklist" })
const _hoisted_2 = { style: "font-weight: bold; margin-top: 0.5em;" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-external-link" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-external-link" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-external-link" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
import { computed, ref } from 'vue'
import { instance } from '@/instance.js'
import { i18n } from '@/i18n.js'
import MkButton from '@/components/MkButton.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkInfo from '@/components/MkInfo.vue'
import * as os from '@/os.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSignupDialog.rules',
  emits: ["cancel", "done"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const availableServerRules = instance.serverRules.length > 0;
const availableTos = instance.tosUrl != null && instance.tosUrl !== '';
const availablePrivacyPolicy = instance.privacyPolicyUrl != null && instance.privacyPolicyUrl !== '';
const agreeServerRules = ref(false);
const agreeTosAndPrivacyPolicy = ref(false);
const agreeNote = ref(false);
const agreed = computed(() => {
	return (!availableServerRules || agreeServerRules.value) && ((!availableTos && !availablePrivacyPolicy) || agreeTosAndPrivacyPolicy.value) && agreeNote.value;
});
const tosPrivacyPolicyLabel = computed(() => {
	if (availableTos && availablePrivacyPolicy) {
		return i18n.ts.tosAndPrivacyPolicy;
	} else if (availableTos) {
		return i18n.ts.termsOfService;
	} else if (availablePrivacyPolicy) {
		return i18n.ts.privacyPolicy;
	} else {
		return '';
	}
});
async function updateAgreeServerRules(v: boolean) {
	if (v) {
		const confirm = await os.confirm({
			type: 'question',
			title: i18n.ts.doYouAgree,
			text: i18n.tsx.iHaveReadXCarefullyAndAgree({ x: i18n.ts.serverRules }),
		});
		if (confirm.canceled) return;
		agreeServerRules.value = true;
	} else {
		agreeServerRules.value = false;
	}
}
async function updateAgreeTosAndPrivacyPolicy(v: boolean) {
	if (v) {
		const confirm = await os.confirm({
			type: 'question',
			title: i18n.ts.doYouAgree,
			text: i18n.tsx.iHaveReadXCarefullyAndAgree({
				x: tosPrivacyPolicyLabel.value,
			}),
		});
		if (confirm.canceled) return;
		agreeTosAndPrivacyPolicy.value = true;
	} else {
		agreeTosAndPrivacyPolicy.value = false;
	}
}
async function updateAgreeNote(v: boolean) {
	if (v) {
		const confirm = await os.confirm({
			type: 'question',
			title: i18n.ts.doYouAgree,
			text: i18n.tsx.iHaveReadXCarefullyAndAgree({ x: i18n.ts.basicNotesBeforeCreateAccount }),
		});
		if (confirm.canceled) return;
		agreeNote.value = true;
	} else {
		agreeNote.value = false;
	}
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.banner)
      }, [ _hoisted_1 ]), _createElementVNode("div", {
        class: "_spacer",
        style: "--MI_SPACER-min: 20px; --MI_SPACER-max: 28px;"
      }, [ _createElementVNode("div", { class: "_gaps_m" }, [ (_unref(instance).disableRegistration || _unref(instance).federation !== 'all') ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "_gaps_s"
            }, [ (_unref(instance).disableRegistration) ? (_openBlock(), _createBlock(MkInfo, {
                  key: 0,
                  warn: ""
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.invitationRequiredToRegister), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                })) : _createCommentVNode("v-if", true), (_unref(instance).federation === 'specified') ? (_openBlock(), _createBlock(MkInfo, {
                  key: 0,
                  warn: ""
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.federationSpecified), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                })) : (_unref(instance).federation === 'none') ? (_openBlock(), _createBlock(MkInfo, {
                    key: 1,
                    warn: ""
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.federationDisabled), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true), _createElementVNode("div", { style: "text-align: center;" }, [ _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts.pleaseConfirmBelowBeforeSignup), 1 /* TEXT */), _createElementVNode("div", _hoisted_2, _toDisplayString(_unref(i18n).ts.beSureToReadThisAsItIsImportant), 1 /* TEXT */) ]), (_unref(availableServerRules)) ? (_openBlock(), _createBlock(MkFolder, {
              key: 0,
              defaultOpen: true
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.serverRules), 1 /* TEXT */)
              ]),
              suffix: _withCtx(() => [
                (agreeServerRules.value)
                  ? (_openBlock(), _createElementBlock("i", {
                    key: 0,
                    class: "ti ti-check",
                    style: "color: var(--MI_THEME-success)"
                  }))
                  : _createCommentVNode("v-if", true)
              ]),
              default: _withCtx(() => [
                _createElementVNode("ol", {
                  class: _normalizeClass(["_gaps_s", _ctx.$style.rules])
                }, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(instance).serverRules, (item) => {
                    return (_openBlock(), _createElementBlock("li", { class: _normalizeClass(_ctx.$style.rule) }, [
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.ruleText),
                        innerHTML: item
                      }, null, 8 /* PROPS */, ["innerHTML"])
                    ]))
                  }), 256 /* UNKEYED_FRAGMENT */))
                ]),
                _createVNode(MkSwitch, {
                  modelValue: agreeServerRules.value,
                  style: "margin-top: 16px;",
                  "onUpdate:modelValue": updateAgreeServerRules
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.agree), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["modelValue"])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["defaultOpen"])) : _createCommentVNode("v-if", true), (_unref(availableTos) || _unref(availablePrivacyPolicy)) ? (_openBlock(), _createBlock(MkFolder, {
              key: 0,
              defaultOpen: true
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(tosPrivacyPolicyLabel.value), 1 /* TEXT */)
              ]),
              suffix: _withCtx(() => [
                (agreeTosAndPrivacyPolicy.value)
                  ? (_openBlock(), _createElementBlock("i", {
                    key: 0,
                    class: "ti ti-check",
                    style: "color: var(--MI_THEME-success)"
                  }))
                  : _createCommentVNode("v-if", true)
              ]),
              default: _withCtx(() => [
                _createElementVNode("div", { class: "_gaps_s" }, [
                  (_unref(availableTos))
                    ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                      _createElementVNode("a", {
                        href: _unref(instance).tosUrl ?? undefined,
                        class: "_link",
                        target: "_blank"
                      }, [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.termsOfService) + " ", 1 /* TEXT */),
                        _hoisted_3
                      ], 8 /* PROPS */, ["href"])
                    ]))
                    : _createCommentVNode("v-if", true),
                  (_unref(availablePrivacyPolicy))
                    ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                      _createElementVNode("a", {
                        href: _unref(instance).privacyPolicyUrl ?? undefined,
                        class: "_link",
                        target: "_blank"
                      }, [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.privacyPolicy) + " ", 1 /* TEXT */),
                        _hoisted_4
                      ], 8 /* PROPS */, ["href"])
                    ]))
                    : _createCommentVNode("v-if", true)
                ]),
                _createVNode(MkSwitch, {
                  modelValue: agreeTosAndPrivacyPolicy.value,
                  style: "margin-top: 16px;",
                  "onUpdate:modelValue": updateAgreeTosAndPrivacyPolicy
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.agree), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["modelValue"])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["defaultOpen"])) : _createCommentVNode("v-if", true), _createVNode(MkFolder, { defaultOpen: true }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.basicNotesBeforeCreateAccount), 1 /* TEXT */)
            ]),
            suffix: _withCtx(() => [
              (agreeNote.value)
                ? (_openBlock(), _createElementBlock("i", {
                  key: 0,
                  class: "ti ti-check",
                  style: "color: var(--MI_THEME-success)"
                }))
                : _createCommentVNode("v-if", true)
            ]),
            default: _withCtx(() => [
              _createElementVNode("a", {
                href: "https://misskey-hub.net/docs/for-users/onboarding/warning/",
                class: "_link",
                target: "_blank"
              }, [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.basicNotesBeforeCreateAccount) + " ", 1 /* TEXT */),
                _hoisted_5
              ]),
              _createVNode(MkSwitch, {
                modelValue: agreeNote.value,
                style: "margin-top: 16px;",
                "data-cy-signup-rules-notes-agree": "",
                "onUpdate:modelValue": updateAgreeNote
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.agree), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["defaultOpen"]), (!agreed.value) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              style: "text-align: center;"
            }, _toDisplayString(_unref(i18n).ts.pleaseAgreeAllToContinue), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createElementVNode("div", { class: "_buttonsCenter" }, [ _createVNode(MkButton, {
              inline: "",
              rounded: "",
              onClick: _cache[0] || (_cache[0] = ($event: any) => (emit('cancel')))
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.cancel), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }), _createVNode(MkButton, {
              inline: "",
              primary: "",
              rounded: "",
              gradate: "",
              disabled: !agreed.value,
              "data-cy-signup-rules-continue": "",
              onClick: _cache[1] || (_cache[1] = ($event: any) => (emit('done')))
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.continue), 1 /* TEXT */),
                _createTextVNode(" "),
                _hoisted_6
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["disabled"]) ]) ]) ]) ]))
}
}

})
