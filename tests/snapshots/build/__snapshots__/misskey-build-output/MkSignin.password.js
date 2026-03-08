import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-lock" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
import { ref, computed, useTemplateRef, defineAsyncComponent } from 'vue'
import * as Misskey from 'misskey-js'
import { instance } from '@/instance.js'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import MkCaptcha from '@/components/MkCaptcha.vue'

export type PwResponse = {
	password: string;
	captcha: {
		hCaptchaResponse: string | null;
		mCaptchaResponse: string | null;
		reCaptchaResponse: string | null;
		turnstileResponse: string | null;
		testcaptchaResponse: string | null;
	};
};

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSignin.password',
  props: {
    user: { type: null, required: true },
    needCaptcha: { type: Boolean, required: true }
  },
  emits: ["passwordSubmitted"],
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const password = ref('');
const hCaptcha = useTemplateRef('hcaptcha');
const mCaptcha = useTemplateRef('mcaptcha');
const reCaptcha = useTemplateRef('recaptcha');
const turnstile = useTemplateRef('turnstile');
const testcaptcha = useTemplateRef('testcaptcha');
const hCaptchaResponse = ref<string | null>(null);
const mCaptchaResponse = ref<string | null>(null);
const reCaptchaResponse = ref<string | null>(null);
const turnstileResponse = ref<string | null>(null);
const testcaptchaResponse = ref<string | null>(null);
const captchaFailed = computed((): boolean => {
	return (
		(instance.enableHcaptcha && !hCaptchaResponse.value) ||
		(instance.enableMcaptcha && !mCaptchaResponse.value) ||
		(instance.enableRecaptcha && !reCaptchaResponse.value) ||
		(instance.enableTurnstile && !turnstileResponse.value) ||
		(instance.enableTestcaptcha && !testcaptchaResponse.value)
	);
});
function resetPassword(): void {
	const { dispose } = os.popup(defineAsyncComponent(() => import('@/components/MkForgotPassword.vue')), {}, {
		closed: () => dispose(),
	});
}
function onSubmit() {
	emit('passwordSubmitted', {
		password: password.value,
		captcha: {
			hCaptchaResponse: hCaptchaResponse.value,
			mCaptchaResponse: mCaptchaResponse.value,
			reCaptchaResponse: reCaptchaResponse.value,
			turnstileResponse: turnstileResponse.value,
			testcaptchaResponse: testcaptchaResponse.value,
		},
	});
}
function resetCaptcha() {
	hCaptcha.value?.reset();
	mCaptcha.value?.reset();
	reCaptcha.value?.reset();
	turnstile.value?.reset();
	testcaptcha.value?.reset();
}
__expose({
	resetCaptcha,
})

return (_ctx: any,_cache: any) => {
  const _component_Mfm = _resolveComponent("Mfm")
  const _component_I18n = _resolveComponent("I18n")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.wrapper),
      "data-cy-signin-page-password": ""
    }, [ _createElementVNode("div", {
        class: _normalizeClass(["_gaps", _ctx.$style.root])
      }, [ _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.avatar),
          style: _normalizeStyle({ backgroundImage: __props.user ? `url('${__props.user.avatarUrl}')` : undefined })
        }, null, 4 /* STYLE */), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.welcomeBackMessage)
        }, [ _createVNode(_component_I18n, {
            src: _unref(i18n).ts.welcomeBackWithName,
            tag: "span"
          }, {
            name: _withCtx(() => [
              _createVNode(_component_Mfm, {
                text: __props.user.name ?? __props.user.username,
                plain: true
              }, null, 8 /* PROPS */, ["text", "plain"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["src"]) ]), _createTextVNode("\n\n\t\t" + "\n\t\t"), _createElementVNode("form", {
          class: "_gaps_s",
          onSubmit: _withModifiers(onSubmit, ["prevent"])
        }, [ _createElementVNode("input", {
            type: "hidden",
            name: "username",
            autocomplete: "username",
            value: __props.user.username
          }, null, 8 /* PROPS */, ["value"]), _createVNode(MkInput, {
            placeholder: _unref(i18n).ts.password,
            type: "password",
            autocomplete: "current-password webauthn",
            withPasswordToggle: true,
            required: "",
            autofocus: "",
            "data-cy-signin-password": "",
            modelValue: password.value,
            "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((password).value = $event))
          }, {
            prefix: _withCtx(() => [
              _hoisted_1
            ]),
            caption: _withCtx(() => [
              _createElementVNode("button", {
                class: "_textButton",
                type: "button",
                onClick: resetPassword
              }, _toDisplayString(_unref(i18n).ts.forgotPassword), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["placeholder", "withPasswordToggle", "modelValue"]), (__props.needCaptcha) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ (_unref(instance).enableHcaptcha) ? (_openBlock(), _createBlock(MkCaptcha, {
                  key: 0,
                  ref: "hcaptcha",
                  provider: "hcaptcha",
                  sitekey: _unref(instance).hcaptchaSiteKey,
                  modelValue: hCaptchaResponse.value,
                  "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((hCaptchaResponse).value = $event))
                }, null, 8 /* PROPS */, ["sitekey", "modelValue"])) : _createCommentVNode("v-if", true), (_unref(instance).enableMcaptcha) ? (_openBlock(), _createBlock(MkCaptcha, {
                  key: 0,
                  ref: "mcaptcha",
                  provider: "mcaptcha",
                  sitekey: _unref(instance).mcaptchaSiteKey,
                  instanceUrl: _unref(instance).mcaptchaInstanceUrl,
                  modelValue: mCaptchaResponse.value,
                  "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((mCaptchaResponse).value = $event))
                }, null, 8 /* PROPS */, ["sitekey", "instanceUrl", "modelValue"])) : _createCommentVNode("v-if", true), (_unref(instance).enableRecaptcha) ? (_openBlock(), _createBlock(MkCaptcha, {
                  key: 0,
                  ref: "recaptcha",
                  provider: "recaptcha",
                  sitekey: _unref(instance).recaptchaSiteKey,
                  modelValue: reCaptchaResponse.value,
                  "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((reCaptchaResponse).value = $event))
                }, null, 8 /* PROPS */, ["sitekey", "modelValue"])) : _createCommentVNode("v-if", true), (_unref(instance).enableTurnstile) ? (_openBlock(), _createBlock(MkCaptcha, {
                  key: 0,
                  ref: "turnstile",
                  provider: "turnstile",
                  sitekey: _unref(instance).turnstileSiteKey,
                  modelValue: turnstileResponse.value,
                  "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((turnstileResponse).value = $event))
                }, null, 8 /* PROPS */, ["sitekey", "modelValue"])) : _createCommentVNode("v-if", true), (_unref(instance).enableTestcaptcha) ? (_openBlock(), _createBlock(MkCaptcha, {
                  key: 0,
                  ref: "testcaptcha",
                  provider: "testcaptcha",
                  sitekey: null,
                  modelValue: testcaptchaResponse.value,
                  "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((testcaptchaResponse).value = $event))
                }, null, 8 /* PROPS */, ["sitekey", "modelValue"])) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true), _createVNode(MkButton, {
            type: "submit",
            disabled: __props.needCaptcha && captchaFailed.value,
            large: "",
            primary: "",
            rounded: "",
            style: "margin: 0 auto;",
            "data-cy-signin-page-password-continue": ""
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.continue), 1 /* TEXT */),
              _createTextVNode(" "),
              _hoisted_2
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["disabled"]) ], 32 /* NEED_HYDRATION */) ]) ]))
}
}

})
