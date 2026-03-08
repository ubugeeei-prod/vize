import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("img", { src: "/client-assets/locked_with_key_3d.png", alt: "🔐", style: "display: block; margin: 0 auto; width: 48px;" })
const _hoisted_2 = { style: "margin-top: 16px;" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-password" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-lock-open" })
import { onMounted, useTemplateRef, ref } from 'vue'
import MkInput from '@/components/MkInput.vue'
import MkButton from '@/components/MkButton.vue'
import MkModalWindow from '@/components/MkModalWindow.vue'
import { i18n } from '@/i18n.js'
import { ensureSignin } from '@/i.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkPasswordDialog',
  emits: ["done", "closed", "cancelled"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const $i = ensureSignin();
const dialog = useTemplateRef('dialog');
const passwordInput = useTemplateRef('passwordInput');
const password = ref('');
const isBackupCode = ref(false);
const token = ref<string | null>(null);
function onClose() {
	emit('cancelled');
	if (dialog.value) dialog.value.close();
}
function done() {
	emit('done', { password: password.value, token: token.value });
	if (dialog.value) dialog.value.close();
}
onMounted(() => {
	if (passwordInput.value) passwordInput.value.focus();
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkModalWindow, {
      ref_key: "dialog", ref: dialog,
      width: 370,
      height: 400,
      onClose: onClose,
      onClosed: _cache[0] || (_cache[0] = ($event: any) => (emit('closed')))
    }, {
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(i18n).ts.authentication), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-min: 20px; --MI_SPACER-max: 28px;"
        }, [
          _createElementVNode("div", { style: "padding: 0 0 16px 0; text-align: center;" }, [
            _hoisted_1,
            _createElementVNode("div", _hoisted_2, _toDisplayString(_unref(i18n).ts.authenticationRequiredToContinue), 1 /* TEXT */)
          ]),
          _createElementVNode("form", {
            onSubmit: _withModifiers(done, ["prevent"])
          }, [
            _createElementVNode("div", { class: "_gaps" }, [
              _createVNode(MkInput, {
                ref_key: "passwordInput", ref: passwordInput,
                placeholder: _unref(i18n).ts.password,
                type: "password",
                autocomplete: "current-password webauthn",
                required: "",
                withPasswordToggle: true,
                modelValue: password.value,
                "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((password).value = $event))
              }, {
                prefix: _withCtx(() => [
                  _hoisted_3
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["placeholder", "withPasswordToggle", "modelValue"]),
              (_unref($i).twoFactorEnabled)
                ? (_openBlock(), _createBlock(MkInput, {
                  key: 0,
                  type: "text",
                  pattern: isBackupCode.value ? '^[A-Z0-9]{32}$' :'^[0-9]{6}$',
                  autocomplete: "one-time-code",
                  required: "",
                  spellcheck: false,
                  inputmode: isBackupCode.value ? undefined : 'numeric',
                  modelValue: token.value,
                  "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((token).value = $event))
                }, {
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.token) + " (" + _toDisplayString(_unref(i18n).ts['2fa']) + ")", 1 /* TEXT */)
                  ]),
                  prefix: _withCtx(() => [
                    (isBackupCode.value)
                      ? (_openBlock(), _createElementBlock("i", {
                        key: 0,
                        class: "ti ti-key"
                      }))
                      : (_openBlock(), _createElementBlock("i", {
                        key: 1,
                        class: "ti ti-123"
                      }))
                  ]),
                  caption: _withCtx(() => [
                    _createElementVNode("button", {
                      class: "_textButton",
                      type: "button",
                      onClick: _cache[3] || (_cache[3] = ($event: any) => (isBackupCode.value = !isBackupCode.value))
                    }, _toDisplayString(isBackupCode.value ? _unref(i18n).ts.useTotp : _unref(i18n).ts.useBackupCode), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["pattern", "spellcheck", "inputmode", "modelValue"]))
                : _createCommentVNode("v-if", true),
              _createVNode(MkButton, {
                disabled: (password.value ?? '') == '' || (_unref($i).twoFactorEnabled && (token.value ?? '') == ''),
                type: "submit",
                primary: "",
                rounded: "",
                style: "margin: 0 auto;"
              }, {
                default: _withCtx(() => [
                  _hoisted_4,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.continue), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["disabled"])
            ])
          ], 32 /* NEED_HYDRATION */)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["width", "height"]))
}
}

})
