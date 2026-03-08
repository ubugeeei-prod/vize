import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-user-edit" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-key" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-help-circle" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check ti-fw" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-help-circle" })
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-mail" })
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check ti-fw" })
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" })
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" })
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" })
const _hoisted_17 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" })
const _hoisted_18 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" })
const _hoisted_19 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" })
const _hoisted_20 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" })
const _hoisted_21 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" })
const _hoisted_22 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-lock" })
const _hoisted_23 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" })
const _hoisted_24 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check ti-fw" })
const _hoisted_25 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check ti-fw" })
const _hoisted_26 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-lock" })
const _hoisted_27 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check ti-fw" })
const _hoisted_28 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" })
import { ref, computed } from 'vue'
import { toUnicode } from 'punycode.js'
import * as Misskey from 'misskey-js'
import * as config from '@@/js/config.js'
import MkButton from './MkButton.vue'
import MkInput from './MkInput.vue'
import type { Captcha } from '@/components/MkCaptcha.vue'
import MkCaptcha from '@/components/MkCaptcha.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { instance } from '@/instance.js'
import { i18n } from '@/i18n.js'
import { login } from '@/accounts.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSignupDialog.form',
  props: {
    autoSet: { type: Boolean, required: false, default: false }
  },
  emits: ["signup", "signupEmailPending"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const host = toUnicode(config.host);
const hcaptcha = ref<Captcha | undefined>();
const mcaptcha = ref<Captcha | undefined>();
const recaptcha = ref<Captcha | undefined>();
const turnstile = ref<Captcha | undefined>();
const testcaptcha = ref<Captcha | undefined>();
const username = ref<string>('');
const password = ref<string>('');
const retypedPassword = ref<string>('');
const invitationCode = ref<string>('');
const email = ref('');
const usernameState = ref<null | 'wait' | 'ok' | 'unavailable' | 'error' | 'invalid-format' | 'min-range' | 'max-range'>(null);
const emailState = ref<null | 'wait' | 'ok' | 'unavailable:used' | 'unavailable:format' | 'unavailable:disposable' | 'unavailable:banned' | 'unavailable:mx' | 'unavailable:smtp' | 'unavailable' | 'error'>(null);
const passwordStrength = ref<'' | 'low' | 'medium' | 'high'>('');
const passwordRetypeState = ref<null | 'match' | 'not-match'>(null);
const submitting = ref<boolean>(false);
const hCaptchaResponse = ref<string | null>(null);
const mCaptchaResponse = ref<string | null>(null);
const reCaptchaResponse = ref<string | null>(null);
const turnstileResponse = ref<string | null>(null);
const testcaptchaResponse = ref<string | null>(null);
const usernameAbortController = ref<null | AbortController>(null);
const emailAbortController = ref<null | AbortController>(null);
const shouldDisableSubmitting = computed((): boolean => {
	return submitting.value ||
		instance.enableHcaptcha && !hCaptchaResponse.value ||
		instance.enableMcaptcha && !mCaptchaResponse.value ||
		instance.enableRecaptcha && !reCaptchaResponse.value ||
		instance.enableTurnstile && !turnstileResponse.value ||
		instance.enableTestcaptcha && !testcaptchaResponse.value ||
		instance.emailRequiredForSignup && emailState.value !== 'ok' ||
		instance.disableRegistration && invitationCode.value === '' ||
		usernameState.value !== 'ok' ||
		passwordRetypeState.value !== 'match';
});
function getPasswordStrength(source: string): number {
	let strength = 0;
	let power = 0.018;
	// 英数字
	if (/[a-zA-Z]/.test(source) && /[0-9]/.test(source)) {
		power += 0.020;
	}
	// 大文字と小文字が混ざってたら
	if (/[a-z]/.test(source) && /[A-Z]/.test(source)) {
		power += 0.015;
	}
	// 記号が混ざってたら
	if (/[!\x22\#$%&@'()*+,-./_]/.test(source)) {
		power += 0.02;
	}
	strength = power * source.length;
	return Math.max(0, Math.min(1, strength));
}
function onChangeUsername(): void {
	if (username.value === '') {
		usernameState.value = null;
		return;
	}
	{
		const err =
			!username.value.match(/^[a-zA-Z0-9_]+$/) ? 'invalid-format' :
			username.value.length < 1 ? 'min-range' :
			username.value.length > 20 ? 'max-range' :
			null;
		if (err) {
			usernameState.value = err;
			return;
		}
	}
	if (usernameAbortController.value != null) {
		usernameAbortController.value.abort();
	}
	usernameState.value = 'wait';
	usernameAbortController.value = new AbortController();
	misskeyApi('username/available', {
		username: username.value,
	}, undefined, usernameAbortController.value.signal).then(result => {
		usernameState.value = result.available ? 'ok' : 'unavailable';
	}).catch((err) => {
		if (err.name !== 'AbortError') {
			usernameState.value = 'error';
		}
	});
}
function onChangeEmail(): void {
	if (email.value === '') {
		emailState.value = null;
		return;
	}
	if (emailAbortController.value != null) {
		emailAbortController.value.abort();
	}
	emailState.value = 'wait';
	emailAbortController.value = new AbortController();
	misskeyApi('email-address/available', {
		emailAddress: email.value,
	}, undefined, emailAbortController.value.signal).then(result => {
		emailState.value = result.available ? 'ok' :
			result.reason === 'used' ? 'unavailable:used' :
			result.reason === 'format' ? 'unavailable:format' :
			result.reason === 'disposable' ? 'unavailable:disposable' :
			result.reason === 'banned' ? 'unavailable:banned' :
			result.reason === 'mx' ? 'unavailable:mx' :
			result.reason === 'smtp' ? 'unavailable:smtp' :
			'unavailable';
	}).catch((err) => {
		if (err.name !== 'AbortError') {
			emailState.value = 'error';
		}
	});
}
function onChangePassword(): void {
	if (password.value === '') {
		passwordStrength.value = '';
		return;
	}
	const strength = getPasswordStrength(password.value);
	passwordStrength.value = strength > 0.7 ? 'high' : strength > 0.3 ? 'medium' : 'low';
}
function onChangePasswordRetype(): void {
	if (retypedPassword.value === '') {
		passwordRetypeState.value = null;
		return;
	}
	passwordRetypeState.value = password.value === retypedPassword.value ? 'match' : 'not-match';
}
async function onSubmit(): Promise<void> {
	if (submitting.value) return;
	submitting.value = true;
	const signupPayload: Misskey.entities.SignupRequest = {
		username: username.value,
		password: password.value,
		emailAddress: email.value,
		invitationCode: invitationCode.value,
		'hcaptcha-response': hCaptchaResponse.value,
		'm-captcha-response': mCaptchaResponse.value,
		'g-recaptcha-response': reCaptchaResponse.value,
		'turnstile-response': turnstileResponse.value,
		'testcaptcha-response': testcaptchaResponse.value,
	};
	const res = await window.fetch(`${config.apiUrl}/signup`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json',
		},
		body: JSON.stringify(signupPayload),
	}).catch(() => {
		onSignupApiError();
		return null;
	});
	if (res && res.ok) {
		if (res.status === 204 || instance.emailRequiredForSignup) {
			os.alert({
				type: 'success',
				title: i18n.ts._signup.almostThere,
				text: i18n.tsx._signup.emailSent({ email: email.value }),
			});
			emit('signupEmailPending');
		} else {
			const resJson = (await res.json()) as Misskey.entities.SignupResponse;
			if (_DEV_) console.log(resJson);
			emit('signup', resJson);
			if (props.autoSet) {
				await login(resJson.token);
			}
		}
	} else {
		onSignupApiError();
	}
	submitting.value = false;
}
function onSignupApiError() {
	submitting.value = false;
	hcaptcha.value?.reset?.();
	mcaptcha.value?.reset?.();
	recaptcha.value?.reset?.();
	turnstile.value?.reset?.();
	testcaptcha.value?.reset?.();
	os.alert({
		type: 'error',
		text: i18n.ts.somethingHappened,
	});
}

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.banner)
      }, [ _hoisted_1 ]), _createElementVNode("div", {
        class: "_spacer",
        style: "--MI_SPACER-min: 20px; --MI_SPACER-max: 32px;"
      }, [ _createElementVNode("form", {
          class: "_gaps_m",
          autocomplete: "new-password",
          onSubmit: _withModifiers(onSubmit, ["prevent"])
        }, [ (_unref(instance).disableRegistration) ? (_openBlock(), _createBlock(MkInput, {
              key: 0,
              type: "text",
              spellcheck: false,
              required: "",
              "data-cy-signup-invitation-code": "",
              modelValue: invitationCode.value,
              "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((invitationCode).value = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.invitationCode), 1 /* TEXT */)
              ]),
              prefix: _withCtx(() => [
                _hoisted_2
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["spellcheck", "modelValue"])) : _createCommentVNode("v-if", true), _createVNode(MkInput, {
            type: "text",
            pattern: "^[a-zA-Z0-9_]{1,20}$",
            spellcheck: false,
            autocomplete: "username",
            required: "",
            "data-cy-signup-username": "",
            "onUpdate:modelValue": [onChangeUsername, ($event: any) => ((username).value = $event)],
            modelValue: username.value
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.username), 1 /* TEXT */),
              _createTextVNode(" "),
              _createElementVNode("div", { class: "_button _help" }, [
                _hoisted_3
              ])
            ]),
            prefix: _withCtx(() => [
              _createTextVNode("@")
            ]),
            suffix: _withCtx(() => [
              _createTextVNode("@" + _toDisplayString(_unref(host)), 1 /* TEXT */)
            ]),
            caption: _withCtx(() => [
              _createElementVNode("div", null, [
                _hoisted_4,
                _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.cannotBeChangedLater), 1 /* TEXT */)
              ]),
              (usernameState.value === 'wait')
                ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  style: "color:#999"
                }, [
                  _createVNode(_component_MkLoading, { em: true }, null, 8 /* PROPS */, ["em"]),
                  _createTextVNode(),
                  _toDisplayString(_unref(i18n).ts.checking)
                ]))
                : (usernameState.value === 'ok')
                  ? (_openBlock(), _createElementBlock("span", {
                    key: 1,
                    style: "color: var(--MI_THEME-success)"
                  }, [
                    _hoisted_5,
                    _createTextVNode(),
                    _toDisplayString(_unref(i18n).ts.available)
                  ]))
                : (usernameState.value === 'unavailable')
                  ? (_openBlock(), _createElementBlock("span", {
                    key: 2,
                    style: "color: var(--MI_THEME-error)"
                  }, [
                    _hoisted_6,
                    _createTextVNode(),
                    _toDisplayString(_unref(i18n).ts.unavailable)
                  ]))
                : (usernameState.value === 'error')
                  ? (_openBlock(), _createElementBlock("span", {
                    key: 3,
                    style: "color: var(--MI_THEME-error)"
                  }, [
                    _hoisted_7,
                    _createTextVNode(),
                    _toDisplayString(_unref(i18n).ts.error)
                  ]))
                : (usernameState.value === 'invalid-format')
                  ? (_openBlock(), _createElementBlock("span", {
                    key: 4,
                    style: "color: var(--MI_THEME-error)"
                  }, [
                    _hoisted_8,
                    _createTextVNode(),
                    _toDisplayString(_unref(i18n).ts.usernameInvalidFormat)
                  ]))
                : (usernameState.value === 'min-range')
                  ? (_openBlock(), _createElementBlock("span", {
                    key: 5,
                    style: "color: var(--MI_THEME-error)"
                  }, [
                    _hoisted_9,
                    _createTextVNode(),
                    _toDisplayString(_unref(i18n).ts.tooShort)
                  ]))
                : (usernameState.value === 'max-range')
                  ? (_openBlock(), _createElementBlock("span", {
                    key: 6,
                    style: "color: var(--MI_THEME-error)"
                  }, [
                    _hoisted_10,
                    _createTextVNode(),
                    _toDisplayString(_unref(i18n).ts.tooLong)
                  ]))
                : _createCommentVNode("v-if", true)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["spellcheck", "modelValue"]), (_unref(instance).emailRequiredForSignup) ? (_openBlock(), _createBlock(MkInput, {
              key: 0,
              debounce: true,
              type: "email",
              spellcheck: false,
              required: "",
              "data-cy-signup-email": "",
              "onUpdate:modelValue": onChangeEmail,
              modelValue: email.value
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.emailAddress), 1 /* TEXT */),
                _createTextVNode(" "),
                _createElementVNode("div", { class: "_button _help" }, [
                  _hoisted_11
                ])
              ]),
              prefix: _withCtx(() => [
                _hoisted_12
              ]),
              caption: _withCtx(() => [
                (emailState.value === 'wait')
                  ? (_openBlock(), _createElementBlock("span", {
                    key: 0,
                    style: "color:#999"
                  }, [
                    _createVNode(_component_MkLoading, { em: true }, null, 8 /* PROPS */, ["em"]),
                    _createTextVNode(),
                    _toDisplayString(_unref(i18n).ts.checking)
                  ]))
                  : (emailState.value === 'ok')
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 1,
                      style: "color: var(--MI_THEME-success)"
                    }, [
                      _hoisted_13,
                      _createTextVNode(),
                      _toDisplayString(_unref(i18n).ts.available)
                    ]))
                  : (emailState.value === 'unavailable:used')
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 2,
                      style: "color: var(--MI_THEME-error)"
                    }, [
                      _hoisted_14,
                      _createTextVNode(),
                      _toDisplayString(_unref(i18n).ts._emailUnavailable.used)
                    ]))
                  : (emailState.value === 'unavailable:format')
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 3,
                      style: "color: var(--MI_THEME-error)"
                    }, [
                      _hoisted_15,
                      _createTextVNode(),
                      _toDisplayString(_unref(i18n).ts._emailUnavailable.format)
                    ]))
                  : (emailState.value === 'unavailable:disposable')
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 4,
                      style: "color: var(--MI_THEME-error)"
                    }, [
                      _hoisted_16,
                      _createTextVNode(),
                      _toDisplayString(_unref(i18n).ts._emailUnavailable.disposable)
                    ]))
                  : (emailState.value === 'unavailable:banned')
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 5,
                      style: "color: var(--MI_THEME-error)"
                    }, [
                      _hoisted_17,
                      _createTextVNode(),
                      _toDisplayString(_unref(i18n).ts._emailUnavailable.banned)
                    ]))
                  : (emailState.value === 'unavailable:mx')
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 6,
                      style: "color: var(--MI_THEME-error)"
                    }, [
                      _hoisted_18,
                      _createTextVNode(),
                      _toDisplayString(_unref(i18n).ts._emailUnavailable.mx)
                    ]))
                  : (emailState.value === 'unavailable:smtp')
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 7,
                      style: "color: var(--MI_THEME-error)"
                    }, [
                      _hoisted_19,
                      _createTextVNode(),
                      _toDisplayString(_unref(i18n).ts._emailUnavailable.smtp)
                    ]))
                  : (emailState.value === 'unavailable')
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 8,
                      style: "color: var(--MI_THEME-error)"
                    }, [
                      _hoisted_20,
                      _createTextVNode(),
                      _toDisplayString(_unref(i18n).ts.unavailable)
                    ]))
                  : (emailState.value === 'error')
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 9,
                      style: "color: var(--MI_THEME-error)"
                    }, [
                      _hoisted_21,
                      _createTextVNode(),
                      _toDisplayString(_unref(i18n).ts.error)
                    ]))
                  : _createCommentVNode("v-if", true)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["debounce", "spellcheck", "modelValue"])) : _createCommentVNode("v-if", true), _createVNode(MkInput, {
            type: "password",
            autocomplete: "new-password",
            required: "",
            "data-cy-signup-password": "",
            "onUpdate:modelValue": [onChangePassword, ($event: any) => ((password).value = $event)],
            modelValue: password.value
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.password), 1 /* TEXT */)
            ]),
            prefix: _withCtx(() => [
              _hoisted_22
            ]),
            caption: _withCtx(() => [
              (passwordStrength.value == 'low')
                ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  style: "color: var(--MI_THEME-error)"
                }, [
                  _hoisted_23,
                  _createTextVNode(),
                  _toDisplayString(_unref(i18n).ts.weakPassword)
                ]))
                : _createCommentVNode("v-if", true),
              (passwordStrength.value == 'medium')
                ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  style: "color: var(--MI_THEME-warn)"
                }, [
                  _hoisted_24,
                  _createTextVNode(),
                  _toDisplayString(_unref(i18n).ts.normalPassword)
                ]))
                : _createCommentVNode("v-if", true),
              (passwordStrength.value == 'high')
                ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  style: "color: var(--MI_THEME-success)"
                }, [
                  _hoisted_25,
                  _createTextVNode(),
                  _toDisplayString(_unref(i18n).ts.strongPassword)
                ]))
                : _createCommentVNode("v-if", true)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkInput, {
            type: "password",
            autocomplete: "new-password",
            required: "",
            "data-cy-signup-password-retype": "",
            "onUpdate:modelValue": [onChangePasswordRetype, ($event: any) => ((retypedPassword).value = $event)],
            modelValue: retypedPassword.value
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.password) + " (" + _toDisplayString(_unref(i18n).ts.retype) + ")", 1 /* TEXT */)
            ]),
            prefix: _withCtx(() => [
              _hoisted_26
            ]),
            caption: _withCtx(() => [
              (passwordRetypeState.value == 'match')
                ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  style: "color: var(--MI_THEME-success)"
                }, [
                  _hoisted_27,
                  _createTextVNode(),
                  _toDisplayString(_unref(i18n).ts.passwordMatched)
                ]))
                : _createCommentVNode("v-if", true),
              (passwordRetypeState.value == 'not-match')
                ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  style: "color: var(--MI_THEME-error)"
                }, [
                  _hoisted_28,
                  _createTextVNode(),
                  _toDisplayString(_unref(i18n).ts.passwordNotMatched)
                ]))
                : _createCommentVNode("v-if", true)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]), (_unref(instance).enableHcaptcha) ? (_openBlock(), _createBlock(MkCaptcha, {
              key: 0,
              ref: "hcaptcha",
              class: _normalizeClass(_ctx.$style.captcha),
              provider: "hcaptcha",
              sitekey: _unref(instance).hcaptchaSiteKey,
              modelValue: hCaptchaResponse.value,
              "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((hCaptchaResponse).value = $event))
            }, null, 8 /* PROPS */, ["sitekey", "modelValue"])) : _createCommentVNode("v-if", true), (_unref(instance).enableMcaptcha) ? (_openBlock(), _createBlock(MkCaptcha, {
              key: 0,
              ref: "mcaptcha",
              class: _normalizeClass(_ctx.$style.captcha),
              provider: "mcaptcha",
              sitekey: _unref(instance).mcaptchaSiteKey,
              instanceUrl: _unref(instance).mcaptchaInstanceUrl,
              modelValue: mCaptchaResponse.value,
              "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((mCaptchaResponse).value = $event))
            }, null, 8 /* PROPS */, ["sitekey", "instanceUrl", "modelValue"])) : _createCommentVNode("v-if", true), (_unref(instance).enableRecaptcha) ? (_openBlock(), _createBlock(MkCaptcha, {
              key: 0,
              ref: "recaptcha",
              class: _normalizeClass(_ctx.$style.captcha),
              provider: "recaptcha",
              sitekey: _unref(instance).recaptchaSiteKey,
              modelValue: reCaptchaResponse.value,
              "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((reCaptchaResponse).value = $event))
            }, null, 8 /* PROPS */, ["sitekey", "modelValue"])) : _createCommentVNode("v-if", true), (_unref(instance).enableTurnstile) ? (_openBlock(), _createBlock(MkCaptcha, {
              key: 0,
              ref: "turnstile",
              class: _normalizeClass(_ctx.$style.captcha),
              provider: "turnstile",
              sitekey: _unref(instance).turnstileSiteKey,
              modelValue: turnstileResponse.value,
              "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((turnstileResponse).value = $event))
            }, null, 8 /* PROPS */, ["sitekey", "modelValue"])) : _createCommentVNode("v-if", true), (_unref(instance).enableTestcaptcha) ? (_openBlock(), _createBlock(MkCaptcha, {
              key: 0,
              ref: "testcaptcha",
              class: _normalizeClass(_ctx.$style.captcha),
              provider: "testcaptcha",
              sitekey: null,
              modelValue: testcaptchaResponse.value,
              "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((testcaptchaResponse).value = $event))
            }, null, 8 /* PROPS */, ["sitekey", "modelValue"])) : _createCommentVNode("v-if", true), _createVNode(MkButton, {
            type: "submit",
            disabled: shouldDisableSubmitting.value,
            large: "",
            gradate: "",
            rounded: "",
            "data-cy-signup-submit": "",
            style: "margin: 0 auto;"
          }, {
            default: _withCtx(() => [
              (submitting.value)
                ? (_openBlock(), _createBlock(_component_MkLoading, {
                  key: 0,
                  em: true,
                  colored: false
                }, null, 8 /* PROPS */, ["em", "colored"]))
                : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                  _toDisplayString(_unref(i18n).ts.start)
                ], 64 /* STABLE_FRAGMENT */))
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["disabled"]) ], 32 /* NEED_HYDRATION */) ]) ]))
}
}

})
