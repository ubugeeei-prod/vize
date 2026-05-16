import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-user-edit" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-key" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-help-circle" });
const _hoisted_4 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" });
const _hoisted_5 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check ti-fw" });
const _hoisted_6 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" });
const _hoisted_7 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" });
const _hoisted_8 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" });
const _hoisted_9 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" });
const _hoisted_10 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" });
const _hoisted_11 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-help-circle" });
const _hoisted_12 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-mail" });
const _hoisted_13 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check ti-fw" });
const _hoisted_14 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" });
const _hoisted_15 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" });
const _hoisted_16 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" });
const _hoisted_17 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" });
const _hoisted_18 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" });
const _hoisted_19 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" });
const _hoisted_20 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" });
const _hoisted_21 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" });
const _hoisted_22 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-lock" });
const _hoisted_23 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" });
const _hoisted_24 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check ti-fw" });
const _hoisted_25 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check ti-fw" });
const _hoisted_26 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-lock" });
const _hoisted_27 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check ti-fw" });
const _hoisted_28 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-alert-triangle ti-fw" });
import { ref, computed } from "vue";
import { toUnicode } from "punycode.js";
import * as config from "@@/js/config.js";
import MkButton from "./MkButton.vue";
import MkInput from "./MkInput.vue";
import MkCaptcha from "@/components/MkCaptcha.vue";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { instance } from "@/instance.js";
import { i18n } from "@/i18n.js";
import { login } from "@/accounts.js";
export default {
  __name: "MkSignupDialog.form",
  props: { autoSet: {
    type: Boolean,
    required: false,
    default: false
  } },
  emits: ["signup", "signupEmailPending"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const host = toUnicode(config.host);
    const hcaptcha = ref();
    const mcaptcha = ref();
    const recaptcha = ref();
    const turnstile = ref();
    const testcaptcha = ref();
    const username = ref("");
    const password = ref("");
    const retypedPassword = ref("");
    const invitationCode = ref("");
    const email = ref("");
    const usernameState = ref(null);
    const emailState = ref(null);
    const passwordStrength = ref("");
    const passwordRetypeState = ref(null);
    const submitting = ref(false);
    const hCaptchaResponse = ref(null);
    const mCaptchaResponse = ref(null);
    const reCaptchaResponse = ref(null);
    const turnstileResponse = ref(null);
    const testcaptchaResponse = ref(null);
    const usernameAbortController = ref(null);
    const emailAbortController = ref(null);
    const shouldDisableSubmitting = computed(() => {
      return submitting.value || instance.enableHcaptcha && !hCaptchaResponse.value || instance.enableMcaptcha && !mCaptchaResponse.value || instance.enableRecaptcha && !reCaptchaResponse.value || instance.enableTurnstile && !turnstileResponse.value || instance.enableTestcaptcha && !testcaptchaResponse.value || instance.emailRequiredForSignup && emailState.value !== "ok" || instance.disableRegistration && invitationCode.value === "" || usernameState.value !== "ok" || passwordRetypeState.value !== "match";
    });
    function getPasswordStrength(source) {
      let strength = 0;
      let power = .018;
      // 英数字
      if (/[a-zA-Z]/.test(source) && /[0-9]/.test(source)) {
        power += .02;
      }
      // 大文字と小文字が混ざってたら
      if (/[a-z]/.test(source) && /[A-Z]/.test(source)) {
        power += .015;
      }
      // 記号が混ざってたら
      if (/[!\x22\#$%&@'()*+,-./_]/.test(source)) {
        power += .02;
      }
      strength = power * source.length;
      return Math.max(0, Math.min(1, strength));
    }
    function onChangeUsername() {
      if (username.value === "") {
        usernameState.value = null;
        return;
      }
      {
        const err = !username.value.match(/^[a-zA-Z0-9_]+$/) ? "invalid-format" : username.value.length < 1 ? "min-range" : username.value.length > 20 ? "max-range" : null;
        if (err) {
          usernameState.value = err;
          return;
        }
      }
      if (usernameAbortController.value != null) {
        usernameAbortController.value.abort();
      }
      usernameState.value = "wait";
      usernameAbortController.value = new AbortController();
      misskeyApi("username/available", { username: username.value }, undefined, usernameAbortController.value.signal).then((result) => {
        usernameState.value = result.available ? "ok" : "unavailable";
      }).catch((err) => {
        if (err.name !== "AbortError") {
          usernameState.value = "error";
        }
      });
    }
    function onChangeEmail() {
      if (email.value === "") {
        emailState.value = null;
        return;
      }
      if (emailAbortController.value != null) {
        emailAbortController.value.abort();
      }
      emailState.value = "wait";
      emailAbortController.value = new AbortController();
      misskeyApi("email-address/available", { emailAddress: email.value }, undefined, emailAbortController.value.signal).then((result) => {
        emailState.value = result.available ? "ok" : result.reason === "used" ? "unavailable:used" : result.reason === "format" ? "unavailable:format" : result.reason === "disposable" ? "unavailable:disposable" : result.reason === "banned" ? "unavailable:banned" : result.reason === "mx" ? "unavailable:mx" : result.reason === "smtp" ? "unavailable:smtp" : "unavailable";
      }).catch((err) => {
        if (err.name !== "AbortError") {
          emailState.value = "error";
        }
      });
    }
    function onChangePassword() {
      if (password.value === "") {
        passwordStrength.value = "";
        return;
      }
      const strength = getPasswordStrength(password.value);
      passwordStrength.value = strength > .7 ? "high" : strength > .3 ? "medium" : "low";
    }
    function onChangePasswordRetype() {
      if (retypedPassword.value === "") {
        passwordRetypeState.value = null;
        return;
      }
      passwordRetypeState.value = password.value === retypedPassword.value ? "match" : "not-match";
    }
    async function onSubmit() {
      if (submitting.value) return;
      submitting.value = true;
      const signupPayload = {
        username: username.value,
        password: password.value,
        emailAddress: email.value,
        invitationCode: invitationCode.value,
        "hcaptcha-response": hCaptchaResponse.value,
        "m-captcha-response": mCaptchaResponse.value,
        "g-recaptcha-response": reCaptchaResponse.value,
        "turnstile-response": turnstileResponse.value,
        "testcaptcha-response": testcaptchaResponse.value
      };
      const res = await window.fetch(`${config.apiUrl}/signup`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(signupPayload)
      }).catch(() => {
        onSignupApiError();
        return null;
      });
      if (res && res.ok) {
        if (res.status === 204 || instance.emailRequiredForSignup) {
          os.alert({
            type: "success",
            title: i18n.ts._signup.almostThere,
            text: i18n.tsx._signup.emailSent({ email: email.value })
          });
          emit("signupEmailPending");
        } else {
          const resJson = await res.json();
          if (_DEV_) console.log(resJson);
          emit("signup", resJson);
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
        type: "error",
        text: i18n.ts.somethingHappened
      });
    }
    return (_ctx, _cache) => {
      const _component_MkLoading = _resolveComponent("MkLoading");
      const _directive_tooltip = _resolveDirective("tooltip");
      return _openBlock(), _createElementBlock("div", null, [_createElementVNode(
        "div",
        { class: _normalizeClass(_ctx.$style.banner) },
        [_hoisted_1],
        2
        /* CLASS */
      ), _createElementVNode("div", {
        class: "_spacer",
        style: "--MI_SPACER-min: 20px; --MI_SPACER-max: 32px;"
      }, [_createElementVNode(
        "form",
        {
          class: "_gaps_m",
          autocomplete: "new-password",
          onSubmit: _withModifiers(onSubmit, ["prevent"])
        },
        [
          _unref(instance).disableRegistration ? (_openBlock(), _createBlock(MkInput, {
            key: 0,
            type: "text",
            spellcheck: false,
            required: "",
            "data-cy-signup-invitation-code": "",
            modelValue: invitationCode.value,
            "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => invitationCode.value = $event)
          }, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.invitationCode),
              1
              /* TEXT */
            )]),
            prefix: _withCtx(() => [_hoisted_2]),
            _: 1
          }, 8, ["spellcheck", "modelValue"])) : _createCommentVNode("v-if", true),
          _createVNode(MkInput, {
            type: "text",
            pattern: "^[a-zA-Z0-9_]{1,20}$",
            spellcheck: false,
            autocomplete: "username",
            required: "",
            "data-cy-signup-username": "",
            "onUpdate:modelValue": [onChangeUsername, ($event) => username.value = $event],
            modelValue: username.value
          }, {
            label: _withCtx(() => [
              _createTextVNode(
                _toDisplayString(_unref(i18n).ts.username),
                1
                /* TEXT */
              ),
              _createTextVNode(" "),
              _withDirectives(_createElementVNode("div", { class: "_button _help" }, [_hoisted_3]), [[
                _directive_tooltip,
                _unref(i18n).ts.usernameInfo,
                "dialog"
              ]])
            ]),
            prefix: _withCtx(() => [_createTextVNode("@")]),
            suffix: _withCtx(() => [_createTextVNode(
              "@" + _toDisplayString(_unref(host)),
              1
              /* TEXT */
            )]),
            caption: _withCtx(() => [_createElementVNode("div", null, [_hoisted_4, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts.cannotBeChangedLater),
              1
              /* TEXT */
            )]), usernameState.value === "wait" ? (_openBlock(), _createElementBlock("span", {
              key: 0,
              style: "color:#999"
            }, [_createVNode(_component_MkLoading, { em: true }, null, 8, ["em"]), _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts.checking),
              1
              /* TEXT */
            )])) : usernameState.value === "ok" ? (_openBlock(), _createElementBlock("span", {
              key: 1,
              style: "color: var(--MI_THEME-success)"
            }, [_hoisted_5, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts.available),
              1
              /* TEXT */
            )])) : usernameState.value === "unavailable" ? (_openBlock(), _createElementBlock("span", {
              key: 2,
              style: "color: var(--MI_THEME-error)"
            }, [_hoisted_6, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts.unavailable),
              1
              /* TEXT */
            )])) : usernameState.value === "error" ? (_openBlock(), _createElementBlock("span", {
              key: 3,
              style: "color: var(--MI_THEME-error)"
            }, [_hoisted_7, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts.error),
              1
              /* TEXT */
            )])) : usernameState.value === "invalid-format" ? (_openBlock(), _createElementBlock("span", {
              key: 4,
              style: "color: var(--MI_THEME-error)"
            }, [_hoisted_8, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts.usernameInvalidFormat),
              1
              /* TEXT */
            )])) : usernameState.value === "min-range" ? (_openBlock(), _createElementBlock("span", {
              key: 5,
              style: "color: var(--MI_THEME-error)"
            }, [_hoisted_9, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts.tooShort),
              1
              /* TEXT */
            )])) : usernameState.value === "max-range" ? (_openBlock(), _createElementBlock("span", {
              key: 6,
              style: "color: var(--MI_THEME-error)"
            }, [_hoisted_10, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts.tooLong),
              1
              /* TEXT */
            )])) : _createCommentVNode("v-if", true)]),
            _: 1
          }, 8, ["spellcheck", "modelValue"]),
          _unref(instance).emailRequiredForSignup ? (_openBlock(), _createBlock(MkInput, {
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
              _createTextVNode(
                _toDisplayString(_unref(i18n).ts.emailAddress),
                1
                /* TEXT */
              ),
              _createTextVNode(" "),
              _withDirectives(_createElementVNode("div", { class: "_button _help" }, [_hoisted_11]), [[
                _directive_tooltip,
                _unref(i18n).ts._signup.emailAddressInfo,
                "dialog"
              ]])
            ]),
            prefix: _withCtx(() => [_hoisted_12]),
            caption: _withCtx(() => [emailState.value === "wait" ? (_openBlock(), _createElementBlock("span", {
              key: 0,
              style: "color:#999"
            }, [_createVNode(_component_MkLoading, { em: true }, null, 8, ["em"]), _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts.checking),
              1
              /* TEXT */
            )])) : emailState.value === "ok" ? (_openBlock(), _createElementBlock("span", {
              key: 1,
              style: "color: var(--MI_THEME-success)"
            }, [_hoisted_13, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts.available),
              1
              /* TEXT */
            )])) : emailState.value === "unavailable:used" ? (_openBlock(), _createElementBlock("span", {
              key: 2,
              style: "color: var(--MI_THEME-error)"
            }, [_hoisted_14, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts._emailUnavailable.used),
              1
              /* TEXT */
            )])) : emailState.value === "unavailable:format" ? (_openBlock(), _createElementBlock("span", {
              key: 3,
              style: "color: var(--MI_THEME-error)"
            }, [_hoisted_15, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts._emailUnavailable.format),
              1
              /* TEXT */
            )])) : emailState.value === "unavailable:disposable" ? (_openBlock(), _createElementBlock("span", {
              key: 4,
              style: "color: var(--MI_THEME-error)"
            }, [_hoisted_16, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts._emailUnavailable.disposable),
              1
              /* TEXT */
            )])) : emailState.value === "unavailable:banned" ? (_openBlock(), _createElementBlock("span", {
              key: 5,
              style: "color: var(--MI_THEME-error)"
            }, [_hoisted_17, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts._emailUnavailable.banned),
              1
              /* TEXT */
            )])) : emailState.value === "unavailable:mx" ? (_openBlock(), _createElementBlock("span", {
              key: 6,
              style: "color: var(--MI_THEME-error)"
            }, [_hoisted_18, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts._emailUnavailable.mx),
              1
              /* TEXT */
            )])) : emailState.value === "unavailable:smtp" ? (_openBlock(), _createElementBlock("span", {
              key: 7,
              style: "color: var(--MI_THEME-error)"
            }, [_hoisted_19, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts._emailUnavailable.smtp),
              1
              /* TEXT */
            )])) : emailState.value === "unavailable" ? (_openBlock(), _createElementBlock("span", {
              key: 8,
              style: "color: var(--MI_THEME-error)"
            }, [_hoisted_20, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts.unavailable),
              1
              /* TEXT */
            )])) : emailState.value === "error" ? (_openBlock(), _createElementBlock("span", {
              key: 9,
              style: "color: var(--MI_THEME-error)"
            }, [_hoisted_21, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts.error),
              1
              /* TEXT */
            )])) : _createCommentVNode("v-if", true)]),
            _: 1
          }, 8, [
            "debounce",
            "spellcheck",
            "modelValue"
          ])) : _createCommentVNode("v-if", true),
          _createVNode(MkInput, {
            type: "password",
            autocomplete: "new-password",
            required: "",
            "data-cy-signup-password": "",
            "onUpdate:modelValue": [onChangePassword, ($event) => password.value = $event],
            modelValue: password.value
          }, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.password),
              1
              /* TEXT */
            )]),
            prefix: _withCtx(() => [_hoisted_22]),
            caption: _withCtx(() => [
              passwordStrength.value == "low" ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                style: "color: var(--MI_THEME-error)"
              }, [_hoisted_23, _createTextVNode(
                " " + _toDisplayString(_unref(i18n).ts.weakPassword),
                1
                /* TEXT */
              )])) : _createCommentVNode("v-if", true),
              passwordStrength.value == "medium" ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                style: "color: var(--MI_THEME-warn)"
              }, [_hoisted_24, _createTextVNode(
                " " + _toDisplayString(_unref(i18n).ts.normalPassword),
                1
                /* TEXT */
              )])) : _createCommentVNode("v-if", true),
              passwordStrength.value == "high" ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                style: "color: var(--MI_THEME-success)"
              }, [_hoisted_25, _createTextVNode(
                " " + _toDisplayString(_unref(i18n).ts.strongPassword),
                1
                /* TEXT */
              )])) : _createCommentVNode("v-if", true)
            ]),
            _: 1
          }, 8, ["modelValue"]),
          _createVNode(MkInput, {
            type: "password",
            autocomplete: "new-password",
            required: "",
            "data-cy-signup-password-retype": "",
            "onUpdate:modelValue": [onChangePasswordRetype, ($event) => retypedPassword.value = $event],
            modelValue: retypedPassword.value
          }, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.password) + " (" + _toDisplayString(_unref(i18n).ts.retype) + ")",
              1
              /* TEXT */
            )]),
            prefix: _withCtx(() => [_hoisted_26]),
            caption: _withCtx(() => [passwordRetypeState.value == "match" ? (_openBlock(), _createElementBlock("span", {
              key: 0,
              style: "color: var(--MI_THEME-success)"
            }, [_hoisted_27, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts.passwordMatched),
              1
              /* TEXT */
            )])) : _createCommentVNode("v-if", true), passwordRetypeState.value == "not-match" ? (_openBlock(), _createElementBlock("span", {
              key: 0,
              style: "color: var(--MI_THEME-error)"
            }, [_hoisted_28, _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts.passwordNotMatched),
              1
              /* TEXT */
            )])) : _createCommentVNode("v-if", true)]),
            _: 1
          }, 8, ["modelValue"]),
          _unref(instance).enableHcaptcha ? (_openBlock(), _createBlock(MkCaptcha, {
            key: 0,
            ref_key: "hcaptcha",
            ref: hcaptcha,
            class: _normalizeClass(_ctx.$style.captcha),
            provider: "hcaptcha",
            sitekey: _unref(instance).hcaptchaSiteKey,
            modelValue: hCaptchaResponse.value,
            "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event) => hCaptchaResponse.value = $event)
          }, null, 10, ["sitekey", "modelValue"])) : _createCommentVNode("v-if", true),
          _unref(instance).enableMcaptcha ? (_openBlock(), _createBlock(MkCaptcha, {
            key: 0,
            ref_key: "mcaptcha",
            ref: mcaptcha,
            class: _normalizeClass(_ctx.$style.captcha),
            provider: "mcaptcha",
            sitekey: _unref(instance).mcaptchaSiteKey,
            instanceUrl: _unref(instance).mcaptchaInstanceUrl,
            modelValue: mCaptchaResponse.value,
            "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event) => mCaptchaResponse.value = $event)
          }, null, 10, [
            "sitekey",
            "instanceUrl",
            "modelValue"
          ])) : _createCommentVNode("v-if", true),
          _unref(instance).enableRecaptcha ? (_openBlock(), _createBlock(MkCaptcha, {
            key: 0,
            ref_key: "recaptcha",
            ref: recaptcha,
            class: _normalizeClass(_ctx.$style.captcha),
            provider: "recaptcha",
            sitekey: _unref(instance).recaptchaSiteKey,
            modelValue: reCaptchaResponse.value,
            "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event) => reCaptchaResponse.value = $event)
          }, null, 10, ["sitekey", "modelValue"])) : _createCommentVNode("v-if", true),
          _unref(instance).enableTurnstile ? (_openBlock(), _createBlock(MkCaptcha, {
            key: 0,
            ref_key: "turnstile",
            ref: turnstile,
            class: _normalizeClass(_ctx.$style.captcha),
            provider: "turnstile",
            sitekey: _unref(instance).turnstileSiteKey,
            modelValue: turnstileResponse.value,
            "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event) => turnstileResponse.value = $event)
          }, null, 10, ["sitekey", "modelValue"])) : _createCommentVNode("v-if", true),
          _unref(instance).enableTestcaptcha ? (_openBlock(), _createBlock(MkCaptcha, {
            key: 0,
            ref_key: "testcaptcha",
            ref: testcaptcha,
            class: _normalizeClass(_ctx.$style.captcha),
            provider: "testcaptcha",
            sitekey: null,
            modelValue: testcaptchaResponse.value,
            "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event) => testcaptchaResponse.value = $event)
          }, null, 10, ["sitekey", "modelValue"])) : _createCommentVNode("v-if", true),
          _createVNode(MkButton, {
            type: "submit",
            disabled: shouldDisableSubmitting.value,
            large: "",
            gradate: "",
            rounded: "",
            "data-cy-signup-submit": "",
            style: "margin: 0 auto;"
          }, {
            default: _withCtx(() => [submitting.value ? (_openBlock(), _createBlock(_component_MkLoading, {
              key: 0,
              em: true,
              colored: false
            }, null, 8, ["em", "colored"])) : (_openBlock(), _createElementBlock(
              _Fragment,
              { key: 1 },
              [_createTextVNode(
                _toDisplayString(_unref(i18n).ts.start),
                1
                /* TEXT */
              )],
              64
              /* STABLE_FRAGMENT */
            ))]),
            _: 2
          }, 1032, ["disabled"])
        ],
        32
        /* NEED_HYDRATION */
      )])]);
    };
  }
};
