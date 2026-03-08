import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-key" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
import { ref } from 'vue'
import { i18n } from '@/i18n.js'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSignin.totp',
  emits: ["totpSubmitted"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const token = ref('');
const isBackupCode = ref(false);

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.wrapper)
    }, [ _createElementVNode("div", {
        class: _normalizeClass(["_gaps", _ctx.$style.root])
      }, [ _createElementVNode("div", { class: "_gaps_s" }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.totpIcon)
          }, [ _hoisted_1 ]), _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.totpDescription)
          }, _toDisplayString(_unref(i18n).ts['2fa']), 1 /* TEXT */) ]), _createTextVNode("\n\n\t\t" + "\n\t\t"), _createElementVNode("form", {
          class: "_gaps_s",
          onSubmit: _cache[0] || (_cache[0] = _withModifiers(($event: any) => (emit('totpSubmitted', token.value)), ["prevent"]))
        }, [ _createVNode(MkInput, {
            type: "text",
            pattern: isBackupCode.value ? '^[A-Z0-9]{32}$' :'^[0-9]{6}$',
            autocomplete: "one-time-code",
            required: "",
            autofocus: "",
            spellcheck: false,
            inputmode: isBackupCode.value ? undefined : 'numeric',
            modelValue: token.value,
            "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((token).value = $event))
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
                onClick: _cache[2] || (_cache[2] = ($event: any) => (isBackupCode.value = !isBackupCode.value))
              }, _toDisplayString(isBackupCode.value ? _unref(i18n).ts.useTotp : _unref(i18n).ts.useBackupCode), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["pattern", "spellcheck", "inputmode", "modelValue"]), _createVNode(MkButton, {
            type: "submit",
            large: "",
            primary: "",
            rounded: "",
            style: "margin: 0 auto;"
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.continue), 1 /* TEXT */),
              _createTextVNode(" "),
              _hoisted_2
            ]),
            _: 1 /* STABLE */
          }) ], 32 /* NEED_HYDRATION */) ]) ]))
}
}

})
