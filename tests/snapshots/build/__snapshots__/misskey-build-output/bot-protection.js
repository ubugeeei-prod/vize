import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, createSlots as _createSlots, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-shield" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-key" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-key" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", null, "ref: ")
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("a", { href: "https://docs.hcaptcha.com/#integration-testing-test-keys", target: "_blank" }, "hCaptcha Developer Guide")
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-key" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-key" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-link" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-key" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-key" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("span", null, "ref: ")
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("a", { href: "https://developers.google.com/recaptcha/docs/faq?hl=ja#id-like-to-run-automated-tests-with-recaptcha.-what-should-i-do", target: "_blank" }, "reCAPTCHA FAQ")
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-key" })
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-key" })
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("span", null, "ref: ")
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("a", { href: "https://developers.cloudflare.com/turnstile/troubleshooting/testing/", target: "_blank" }, "Cloudflare Docs")
import { computed, defineAsyncComponent, ref, watch } from 'vue'
import * as Misskey from 'misskey-js'
import type { ApiWithDialogCustomErrors } from '@/os.js'
import MkRadios from '@/components/MkRadios.vue'
import MkInput from '@/components/MkInput.vue'
import FormSlot from '@/components/form/slot.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { fetchInstance } from '@/instance.js'
import { i18n } from '@/i18n.js'
import { useForm } from '@/composables/use-form.js'
import MkFormFooter from '@/components/MkFormFooter.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkInfo from '@/components/MkInfo.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'bot-protection',
  async setup(__props) {

let __temp: any, __restore: any

const MkCaptcha = defineAsyncComponent(() => import('@/components/MkCaptcha.vue'));
const errorHandler: ApiWithDialogCustomErrors = {
	// 検証リクエストそのものに失敗
	'0f4fe2f1-2c15-4d6e-b714-efbfcde231cd': {
		title: i18n.ts._captcha._error._requestFailed.title,
		text: i18n.ts._captcha._error._requestFailed.text,
	},
	// 検証リクエストの結果が不正
	'c41c067f-24f3-4150-84b2-b5a3ae8c2214': {
		title: i18n.ts._captcha._error._verificationFailed.title,
		text: i18n.ts._captcha._error._verificationFailed.text,
	},
	// 不明なエラー
	'f868d509-e257-42a9-99c1-42614b031a97': {
		title: i18n.ts._captcha._error._unknown.title,
		text: i18n.ts._captcha._error._unknown.text,
	},
};
const captchaResult = ref<string | null>(null);
const meta = await misskeyApi('admin/captcha/current');
const botProtectionForm = useForm({
	provider: meta.provider,
	hcaptchaSiteKey: meta.hcaptcha.siteKey,
	hcaptchaSecretKey: meta.hcaptcha.secretKey,
	mcaptchaSiteKey: meta.mcaptcha.siteKey,
	mcaptchaSecretKey: meta.mcaptcha.secretKey,
	mcaptchaInstanceUrl: meta.mcaptcha.instanceUrl,
	recaptchaSiteKey: meta.recaptcha.siteKey,
	recaptchaSecretKey: meta.recaptcha.secretKey,
	turnstileSiteKey: meta.turnstile.siteKey,
	turnstileSecretKey: meta.turnstile.secretKey,
}, async (state) => {
	const provider = state.provider;
	if (provider === 'none') {
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog()),
  await __temp,
  __restore()
)
			'admin/captcha/save',
			{ provider: provider as Misskey.entities.AdminCaptchaSaveRequest['provider'] },
			undefined,
			errorHandler,
		);
	} else {
		const sitekey = provider === 'hcaptcha'
			? state.hcaptchaSiteKey
			: provider === 'mcaptcha'
				? state.mcaptchaSiteKey
				: provider === 'recaptcha'
					? state.recaptchaSiteKey
					: provider === 'turnstile'
						? state.turnstileSiteKey
						: null;
		const secret = provider === 'hcaptcha'
			? state.hcaptchaSecretKey
			: provider === 'mcaptcha'
				? state.mcaptchaSecretKey
				: provider === 'recaptcha'
					? state.recaptchaSecretKey
					: provider === 'turnstile'
						? state.turnstileSecretKey
						: null;

;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog()),
  await __temp,
  __restore()
)
			'admin/captcha/save',
			{
				provider: provider as Misskey.entities.AdminCaptchaSaveRequest['provider'],
				sitekey: sitekey,
				secret: secret,
				instanceUrl: state.mcaptchaInstanceUrl,
				captchaResult: captchaResult.value,
			},
			undefined,
			errorHandler,
		);
	}

;(
  ([__temp,__restore] = _withAsyncContext(() => fetchInstance(true))),
  await __temp,
  __restore()
)
});
watch(botProtectionForm.state, () => {
	captchaResult.value = null;
});
const canSaving = computed((): boolean => {
	return (botProtectionForm.state.provider === 'none') ||
		(botProtectionForm.state.provider === 'hcaptcha' && !!captchaResult.value) ||
		(botProtectionForm.state.provider === 'mcaptcha' && !!captchaResult.value) ||
		(botProtectionForm.state.provider === 'recaptcha' && !!captchaResult.value) ||
		(botProtectionForm.state.provider === 'turnstile' && !!captchaResult.value) ||
		(botProtectionForm.state.provider === 'testcaptcha' && !!captchaResult.value);
});

return (_ctx: any,_cache: any) => {
  const _component_SearchIcon = _resolveComponent("SearchIcon")
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_SearchMarker = _resolveComponent("SearchMarker")

  return (_openBlock(), _createBlock(_component_SearchMarker, {
      markerId: "botProtection",
      keywords: ['bot', 'protection', 'captcha', 'hcaptcha', 'mcaptcha', 'recaptcha', 'turnstile']
    }, {
      default: _withCtx(() => [
        _createVNode(MkFolder, null, _createSlots({ _: 2 /* DYNAMIC */ }, [
          {
            name: "icon",
            fn: _withCtx(() => [
              _createVNode(_component_SearchIcon, null, {
                default: _withCtx(() => [
                  _hoisted_1
                ]),
                _: 1 /* STABLE */
              })
            ])
          },
          {
            name: "label",
            fn: _withCtx(() => [
              _createVNode(_component_SearchLabel, null, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.botProtection), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ])
          },
          (_unref(botProtectionForm).savedState.provider === 'hcaptcha')
            ? {
              name: "suffix",
              fn: _withCtx(() => [
                _createTextVNode("hCaptcha")
              ]),
              key: "0"
            }
          : (_unref(botProtectionForm).savedState.provider === 'mcaptcha')
            ? {
              name: "suffix",
              fn: _withCtx(() => [
                _createTextVNode("mCaptcha")
              ]),
              key: "1"
            }
          : (_unref(botProtectionForm).savedState.provider === 'recaptcha')
            ? {
              name: "suffix",
              fn: _withCtx(() => [
                _createTextVNode("reCAPTCHA")
              ]),
              key: "2"
            }
          : (_unref(botProtectionForm).savedState.provider === 'turnstile')
            ? {
              name: "suffix",
              fn: _withCtx(() => [
                _createTextVNode("Turnstile")
              ]),
              key: "3"
            }
          : (_unref(botProtectionForm).savedState.provider === 'testcaptcha')
            ? {
              name: "suffix",
              fn: _withCtx(() => [
                _createTextVNode("testCaptcha")
              ]),
              key: "4"
            }
          : {
            name: "suffix",
            fn: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.none) + " (" + _toDisplayString(_unref(i18n).ts.notRecommended) + ")", 1 /* TEXT */)
            ]),
            key: "5"
          }
          : undefined,
          (_unref(botProtectionForm).modified.value)
            ? {
              name: "footer",
              fn: _withCtx(() => [
                _createVNode(MkFormFooter, {
                  canSaving: canSaving.value,
                  form: _unref(botProtectionForm)
                }, null, 8 /* PROPS */, ["canSaving", "form"])
              ]),
              key: "0"
            }
          : undefined
        ]), 1024 /* DYNAMIC_SLOTS */)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["keywords"]))
}
}

})
