import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, vModelText as _vModelText } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("img", {
  src: "/client-assets/testcaptcha.png",
  style: "width: 60px; height: 60px; "
});
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("div", { style: "color: green;" }, "Test captcha passed!");
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("div", { style: "font-size: 13px; margin-bottom: 4px;" }, "Type \"ai-chan-kawaii\" to pass captcha");
import { ref, useTemplateRef, computed, onMounted, onBeforeUnmount, watch, onUnmounted } from "vue";
import { store } from "@/store.js";
export default {
  __name: "MkCaptcha",
  props: {
    provider: {
      type: String,
      required: true
    },
    sitekey: {
      type: [String, null],
      required: true
    },
    secretKey: {
      type: [String, null],
      required: false
    },
    instanceUrl: {
      type: [String, null],
      required: false
    },
    modelValue: {
      type: [String, null],
      required: false
    }
  },
  emits: ["update:modelValue"],
  setup(__props, { expose: __expose, emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const available = ref(false);
    const captchaEl = useTemplateRef("captchaEl");
    const captchaWidgetId = ref(undefined);
    let mCaptchaReciever = null;
    const mCaptchaIframe = useTemplateRef("mCaptchaIframe");
    const mCaptchaRemoveState = ref(false);
    const mCaptchaIframeUrl = computed(() => {
      if (props.provider === "mcaptcha" && !mCaptchaRemoveState.value && props.instanceUrl && props.sitekey) {
        const url = new URL("/widget", props.instanceUrl);
        url.searchParams.set("sitekey", props.sitekey);
        return url.toString();
      }
      return null;
    });
    const testcaptchaInput = ref("");
    const testcaptchaPassed = ref(false);
    const variable = computed(() => {
      switch (props.provider) {
        case "hcaptcha": return "hcaptcha";
        case "recaptcha": return "grecaptcha";
        case "turnstile": return "turnstile";
        case "mcaptcha": return "mcaptcha";
        case "testcaptcha": return "testcaptcha";
      }
    });
    const loaded = !!window[variable.value];
    const src = computed(() => {
      switch (props.provider) {
        case "hcaptcha": return "https://js.hcaptcha.com/1/api.js?render=explicit&recaptchacompat=off";
        case "recaptcha": return "https://www.recaptcha.net/recaptcha/api.js?render=explicit";
        case "turnstile": return "https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit";
        case "mcaptcha": return null;
        case "testcaptcha": return null;
      }
    });
    const scriptId = computed(() => `script-${props.provider}`);
    const captcha = computed(() => window[variable.value] ?? {});
    watch(() => [
      props.instanceUrl,
      props.sitekey,
      props.secretKey
    ], async () => {
      // 変更があったときはリフレッシュと再レンダリングをしておかないと、変更後の値で再検証が出来ない
      if (available.value) {
        callback(undefined);
        clearWidget();
        await requestRender();
      }
    });
    if (loaded || props.provider === "mcaptcha" || props.provider === "testcaptcha") {
      available.value = true;
    } else if (src.value !== null) {
      (window.document.getElementById(scriptId.value) ?? window.document.head.appendChild(Object.assign(window.document.createElement("script"), {
        async: true,
        id: scriptId.value,
        src: src.value
      }))).addEventListener("load", () => available.value = true);
    }
    function reset() {
      if (captcha.value.reset && captchaWidgetId.value !== undefined) {
        try {
          captcha.value.reset(captchaWidgetId.value);
        } catch (error) {
          // ignore
          if (_DEV_) console.warn(error);
        }
      }
      testcaptchaPassed.value = false;
      testcaptchaInput.value = "";
      if (mCaptchaReciever != null) {
        mCaptchaReciever.destroy();
        mCaptchaReciever = null;
      }
    }
    function remove() {
      if (captcha.value.remove && captchaWidgetId.value) {
        try {
          if (_DEV_) console.log("remove", props.provider, captchaWidgetId.value);
          captcha.value.remove(captchaWidgetId.value);
        } catch (error) {
          // ignore
          if (_DEV_) console.warn(error);
        }
      }
      if (props.provider === "mcaptcha") {
        mCaptchaRemoveState.value = true;
      }
    }
    async function requestRender() {
      if (captcha.value.render && captchaEl.value instanceof Element && props.sitekey) {
        // reCAPTCHAのレンダリング重複判定を回避するため、captchaEl配下に仮のdivを用意する.
        // （同じdivに対して複数回renderを呼び出すとreCAPTCHAはエラーを返すので）
        const elem = window.document.createElement("div");
        captchaEl.value.appendChild(elem);
        captchaWidgetId.value = captcha.value.render(elem, {
          sitekey: props.sitekey,
          theme: store.s.darkMode ? "dark" : "light",
          callback,
          "expired-callback": () => callback(undefined),
          "error-callback": () => callback(undefined)
        });
      } else if (props.provider === "mcaptcha" && props.instanceUrl && props.sitekey) {
        const { default: Reciever } = await import("@mcaptcha/core-glue");
        mCaptchaReciever = new Reciever({ siteKey: {
          key: props.sitekey,
          instanceUrl: new URL(props.instanceUrl)
        } }, (token) => {
          callback(token);
        });
        mCaptchaReciever.listen();
        mCaptchaRemoveState.value = false;
      } else {
        window.setTimeout(requestRender, 50);
      }
    }
    function clearWidget() {
      reset();
      remove();
      if (captchaEl.value) {
        // レンダリング先のコンテナの中身を掃除し、フォームが増殖するのを抑止
        captchaEl.value.innerHTML = "";
      }
    }
    function callback(response) {
      emit("update:modelValue", typeof response === "string" ? response : null);
    }
    function onReceivedMessage(message) {
      if (message.data.token) {
        if (props.instanceUrl && new URL(message.origin).host === new URL(props.instanceUrl).host) {
          callback(message.data.token);
        }
      }
    }
    function testcaptchaSubmit() {
      testcaptchaPassed.value = testcaptchaInput.value === "ai-chan-kawaii";
      callback(testcaptchaPassed.value ? "testcaptcha-passed" : undefined);
      if (!testcaptchaPassed.value) testcaptchaInput.value = "";
    }
    onMounted(() => {
      if (available.value) {
        window.addEventListener("message", onReceivedMessage);
        requestRender();
      } else {
        watch(available, requestRender);
      }
    });
    onUnmounted(() => {
      window.removeEventListener("message", onReceivedMessage);
    });
    onBeforeUnmount(() => {
      clearWidget();
    });
    __expose({ reset });
    return (_ctx, _cache) => {
      const _component_MkEllipsis = _resolveComponent("MkEllipsis");
      return _openBlock(), _createElementBlock("div", null, [
        !available.value ? (_openBlock(), _createElementBlock("span", { key: 0 }, [_createTextVNode("Loading"), _createVNode(_component_MkEllipsis)])) : _createCommentVNode("v-if", true),
        props.provider == "mcaptcha" ? (_openBlock(), _createElementBlock("div", { key: 0 }, [mCaptchaIframeUrl.value != null ? (_openBlock(), _createElementBlock("iframe", {
          key: 0,
          ref_key: "mCaptchaIframe",
          ref: mCaptchaIframe,
          src: mCaptchaIframeUrl.value,
          style: "border: none; max-width: 320px; width: 100%; height: 100%; max-height: 80px;"
        }, null, 8, ["src"])) : _createCommentVNode("v-if", true)])) : _createCommentVNode("v-if", true),
        props.provider == "testcaptcha" ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          style: "background: #eee; border: solid 1px #888; padding: 8px; color: #000; max-width: 320px; display: flex; gap: 10px; align-items: center; box-shadow: 2px 2px 6px #0004; border-radius: 4px;"
        }, [_hoisted_1, testcaptchaPassed.value ? (_openBlock(), _createElementBlock("div", { key: 0 }, [_hoisted_2])) : (_openBlock(), _createElementBlock("div", { key: 1 }, [
          _hoisted_3,
          _withDirectives(_createElementVNode(
            "input",
            {
              "onUpdate:modelValue": [
                ($event) => testcaptchaInput.value = $event,
                ($event) => testcaptchaInput.value = $event,
                ($event) => testcaptchaInput.value = $event
              ],
              "data-cy-testcaptcha-input": ""
            },
            null,
            512
            /* NEED_PATCH */
          ), [[_vModelText, testcaptchaInput.value]]),
          _createElementVNode("button", {
            type: "button",
            "data-cy-testcaptcha-submit": "",
            onClick: testcaptchaSubmit
          }, "Submit")
        ]))])) : (_openBlock(), _createElementBlock(
          "div",
          {
            key: 1,
            ref_key: "captchaEl",
            ref: captchaEl
          },
          null,
          512
          /* NEED_PATCH */
        ))
      ]);
    };
  }
};
