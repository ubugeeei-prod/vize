import { defineComponent as _defineComponent } from 'vue'
import { Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"

import { nextTick, onBeforeUnmount, ref, shallowRef, useTemplateRef } from 'vue'
import * as Misskey from 'misskey-js'
import { supported as webAuthnSupported, parseRequestOptionsFromJSON } from '@github/webauthn-json/browser-ponyfill'
import type { AuthenticationPublicKeyCredential } from '@github/webauthn-json/browser-ponyfill'
import type { OpenOnRemoteOptions } from '@/utility/please-login.js'
import type { PwResponse } from '@/components/MkSignin.password.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { showSuspendedDialog } from '@/utility/show-suspended-dialog.js'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'
import XInput from '@/components/MkSignin.input.vue'
import XPassword from '@/components/MkSignin.password.vue'
import XTotp from '@/components/MkSignin.totp.vue'
import XPasskey from '@/components/MkSignin.passkey.vue'
import { login } from '@/accounts.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSignin',
  props: {
    autoSet: { type: Boolean, required: false, default: false },
    message: { type: String, required: false, default: '' },
    openOnRemote: { type: null, required: false, default: undefined },
    initialUsername: { type: String, required: false, default: undefined }
  },
  emits: ["login"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const page = ref<'input' | 'password' | 'totp' | 'passkey'>('input');
const waiting = ref(false);
const passwordPageEl = useTemplateRef('passwordPageEl');
const needCaptcha = ref(false);
const userInfo = ref<null | Misskey.entities.UserDetailed>(null);
const password = ref('');
//#region Passkey Passwordless
const credentialRequest = shallowRef<CredentialRequestOptions | null>(null);
const passkeyContext = ref('');
const doingPasskeyFromInputPage = ref(false);
function onPasskeyLogin(): void {
	if (webAuthnSupported()) {
		doingPasskeyFromInputPage.value = true;
		waiting.value = true;
		misskeyApi('signin-with-passkey', {})
			.then((res) => {
				passkeyContext.value = res.context ?? '';
				credentialRequest.value = parseRequestOptionsFromJSON({
					// @ts-expect-error TODO: misskey-js由来の型（@simplewebauthn/types）とフロントエンド由来の型（@github/webauthn-json）が合わない
					publicKey: res.option,
				});
				page.value = 'passkey';
				waiting.value = false;
			})
			.catch(onSigninApiError);
	}
}
function onPasskeyDone(credential: AuthenticationPublicKeyCredential): void {
	waiting.value = true;
	if (doingPasskeyFromInputPage.value) {
		misskeyApi<Misskey.entities.SigninWithPasskeyResponse>('signin-with-passkey', {
			credential: credential.toJSON(),
			context: passkeyContext.value,
		}).then((res) => {
			if (res.signinResponse == null) {
				onSigninApiError();
				return;
			}
			emit('login', res.signinResponse);
			onLoginSucceeded(res.signinResponse);
		}).catch(onSigninApiError);
	} else if (userInfo.value != null) {
		tryLogin({
			username: userInfo.value.username,
			password: password.value,
			// @ts-expect-error TODO: misskey-js由来の型（@simplewebauthn/types）とフロントエンド由来の型（@github/webauthn-json）が合わない
			credential: credential.toJSON(),
		});
	}
}
function onUseTotp(): void {
	page.value = 'totp';
}
//#endregion
async function onUsernameSubmitted(username: string) {
	waiting.value = true;
	userInfo.value = await misskeyApi('users/show', {
		username,
	}).catch(() => null);
	await tryLogin({
		username,
	});
}
async function onPasswordSubmitted(pw: PwResponse) {
	waiting.value = true;
	password.value = pw.password;
	if (userInfo.value == null) {
		await os.alert({
			type: 'error',
			title: i18n.ts.noSuchUser,
			text: i18n.ts.signinFailed,
		});
		waiting.value = false;
		return;
	} else {
		await tryLogin({
			username: userInfo.value.username,
			password: pw.password,
			'hcaptcha-response': pw.captcha.hCaptchaResponse,
			'm-captcha-response': pw.captcha.mCaptchaResponse,
			'g-recaptcha-response': pw.captcha.reCaptchaResponse,
			'turnstile-response': pw.captcha.turnstileResponse,
			'testcaptcha-response': pw.captcha.testcaptchaResponse,
		});
	}
}
async function onTotpSubmitted(token: string) {
	waiting.value = true;
	if (userInfo.value == null) {
		await os.alert({
			type: 'error',
			title: i18n.ts.noSuchUser,
			text: i18n.ts.signinFailed,
		});
		waiting.value = false;
		return;
	} else {
		await tryLogin({
			username: userInfo.value.username,
			password: password.value,
			token,
		});
	}
}
async function tryLogin(req: Partial<Misskey.entities.SigninFlowRequest>): Promise<Misskey.entities.SigninFlowResponse> {
	const _req = {
		username: req.username ?? userInfo.value?.username,
		...req,
	};
	function assertIsSigninFlowRequest(x: Partial<Misskey.entities.SigninFlowRequest>): x is Misskey.entities.SigninFlowRequest {
		return x.username != null;
	}
	if (!assertIsSigninFlowRequest(_req)) {
		throw new Error('Invalid request');
	}
	return await misskeyApi('signin-flow', _req).then(async (res) => {
		if (res.finished) {
			emit('login', res);
			await onLoginSucceeded(res);
		} else {
			switch (res.next) {
				case 'captcha': {
					needCaptcha.value = true;
					page.value = 'password';
					break;
				}
				case 'password': {
					needCaptcha.value = false;
					page.value = 'password';
					break;
				}
				case 'totp': {
					page.value = 'totp';
					break;
				}
				case 'passkey': {
					if (webAuthnSupported()) {
						credentialRequest.value = parseRequestOptionsFromJSON({
							// @ts-expect-error TODO: misskey-js由来の型（@simplewebauthn/types）とフロントエンド由来の型（@github/webauthn-json）が合わない
							publicKey: res.authRequest,
						});
						page.value = 'passkey';
					} else {
						page.value = 'totp';
					}
					break;
				}
			}
			if (doingPasskeyFromInputPage.value === true) {
				doingPasskeyFromInputPage.value = false;
				page.value = 'input';
				password.value = '';
			}
			passwordPageEl.value?.resetCaptcha();
			nextTick(() => {
				waiting.value = false;
			});
		}
		return res;
	}).catch((err) => {
		onSigninApiError(err);
		return Promise.reject(err);
	});
}
async function onLoginSucceeded(res: Misskey.entities.SigninFlowResponse & { finished: true }) {
	if (props.autoSet) {
		await login(res.i);
	}
}
function onSigninApiError(err?: any): void {
	const id = err?.id ?? null;
	switch (id) {
		case '6cc579cc-885d-43d8-95c2-b8c7fc963280': {
			os.alert({
				type: 'error',
				title: i18n.ts.loginFailed,
				text: i18n.ts.noSuchUser,
			});
			break;
		}
		case '932c904e-9460-45b7-9ce6-7ed33be7eb2c': {
			os.alert({
				type: 'error',
				title: i18n.ts.loginFailed,
				text: i18n.ts.incorrectPassword,
			});
			break;
		}
		case 'e03a5f46-d309-4865-9b69-56282d94e1eb': {
			showSuspendedDialog();
			break;
		}
		case '22d05606-fbcf-421a-a2db-b32610dcfd1b': {
			os.alert({
				type: 'error',
				title: i18n.ts.loginFailed,
				text: i18n.ts.rateLimitExceeded,
			});
			break;
		}
		case 'cdf1235b-ac71-46d4-a3a6-84ccce48df6f': {
			os.alert({
				type: 'error',
				title: i18n.ts.loginFailed,
				text: i18n.ts.incorrectTotp,
			});
			break;
		}
		case '36b96a7d-b547-412d-aeed-2d611cdc8cdc': {
			os.alert({
				type: 'error',
				title: i18n.ts.loginFailed,
				text: i18n.ts.unknownWebAuthnKey,
			});
			break;
		}
		case '93b86c4b-72f9-40eb-9815-798928603d1e': {
			os.alert({
				type: 'error',
				title: i18n.ts.loginFailed,
				text: i18n.ts.passkeyVerificationFailed,
			});
			break;
		}
		case 'b18c89a7-5b5e-4cec-bb5b-0419f332d430': {
			os.alert({
				type: 'error',
				title: i18n.ts.loginFailed,
				text: i18n.ts.passkeyVerificationFailed,
			});
			break;
		}
		case '2d84773e-f7b7-4d0b-8f72-bb69b584c912': {
			os.alert({
				type: 'error',
				title: i18n.ts.loginFailed,
				text: i18n.ts.passkeyVerificationSucceededButPasswordlessLoginDisabled,
			});
			break;
		}
		default: {
			console.error(err);
			os.alert({
				type: 'error',
				title: i18n.ts.loginFailed,
				text: JSON.stringify(err),
			});
		}
	}
	if (doingPasskeyFromInputPage.value === true) {
		doingPasskeyFromInputPage.value = false;
		page.value = 'input';
		password.value = '';
	}
	passwordPageEl.value?.resetCaptcha();
	nextTick(() => {
		waiting.value = false;
	});
}
onBeforeUnmount(() => {
	password.value = '';
	needCaptcha.value = false;
	userInfo.value = null;
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.signinRoot)
    }, [ _createVNode(_Transition, {
        mode: "out-in",
        enterActiveClass: _ctx.$style.transition_enterActive,
        leaveActiveClass: _ctx.$style.transition_leaveActive,
        enterFromClass: _ctx.$style.transition_enterFrom,
        leaveToClass: _ctx.$style.transition_leaveTo,
        inert: waiting.value
      }, {
        default: _withCtx(() => [
          (page.value === 'input')
            ? (_openBlock(), _createBlock(XInput, {
              key: "input",
              message: __props.message,
              openOnRemote: __props.openOnRemote,
              initialUsername: __props.initialUsername,
              onUsernameSubmitted: onUsernameSubmitted,
              onPasskeyClick: onPasskeyLogin
            }, null, 8 /* PROPS */, ["message", "openOnRemote", "initialUsername"]))
            : (page.value === 'password')
              ? (_openBlock(), _createBlock(XPassword, {
                key: "password",
                ref: "passwordPageEl",
                user: userInfo.value,
                needCaptcha: needCaptcha.value,
                onPasswordSubmitted: onPasswordSubmitted
              }, null, 8 /* PROPS */, ["user", "needCaptcha"]))
            : (page.value === 'totp')
              ? (_openBlock(), _createBlock(XTotp, {
                key: "totp",
                onTotpSubmitted: onTotpSubmitted
              }))
            : (page.value === 'passkey')
              ? (_openBlock(), _createBlock(XPasskey, {
                key: "passkey",
                credentialRequest: credentialRequest.value,
                isPerformingPasswordlessLogin: doingPasskeyFromInputPage.value,
                onDone: onPasskeyDone,
                onUseTotp: onUseTotp
              }, null, 8 /* PROPS */, ["credentialRequest", "isPerformingPasswordlessLogin"]))
            : _createCommentVNode("v-if", true),
          _createTextVNode("\n\n\t\t"),
          _createTextVNode("\n\t\t"),
          _createTextVNode("\n\n\t\t"),
          _createTextVNode("\n\t\t"),
          _createTextVNode("\n\n\t\t"),
          _createTextVNode("\n\t\t")
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass", "inert"]), (waiting.value) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: _normalizeClass(_ctx.$style.waitingRoot)
        }, [ _createVNode(_component_MkLoading) ])) : _createCommentVNode("v-if", true) ]))
}
}

})
