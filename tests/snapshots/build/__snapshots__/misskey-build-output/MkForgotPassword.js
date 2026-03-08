import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"

import { ref, useTemplateRef } from 'vue'
import MkModalWindow from '@/components/MkModalWindow.vue'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import MkInfo from '@/components/MkInfo.vue'
import * as os from '@/os.js'
import { instance } from '@/instance.js'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkForgotPassword',
  emits: ["done", "closed"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const dialog = useTemplateRef('dialog');
const username = ref('');
const email = ref('');
const processing = ref(false);
async function onSubmit() {
	processing.value = true;
	await os.apiWithDialog('request-reset-password', {
		username: username.value,
		email: email.value,
	});
	emit('done');
	dialog.value?.close();
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkModalWindow, {
      ref_key: "dialog", ref: dialog,
      width: 370,
      height: 400,
      onClose: _cache[0] || (_cache[0] = ($event: any) => (_unref(dialog)?.close())),
      onClosed: _cache[1] || (_cache[1] = ($event: any) => (emit('closed')))
    }, {
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(i18n).ts.forgotPassword), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-min: 20px; --MI_SPACER-max: 28px;"
        }, [
          (_unref(instance).enableEmail)
            ? (_openBlock(), _createElementBlock("form", {
              key: 0,
              onSubmit: _withModifiers(onSubmit, ["prevent"])
            }, [
              _createElementVNode("div", { class: "_gaps_m" }, [
                _createVNode(MkInput, {
                  type: "text",
                  pattern: "^[a-zA-Z0-9_]+$",
                  spellcheck: false,
                  autofocus: "",
                  required: "",
                  modelValue: username.value,
                  "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((username).value = $event))
                }, {
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.username), 1 /* TEXT */)
                  ]),
                  prefix: _withCtx(() => [
                    _createTextVNode("@")
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["spellcheck", "modelValue"]),
                _createVNode(MkInput, {
                  type: "email",
                  spellcheck: false,
                  required: "",
                  modelValue: email.value,
                  "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((email).value = $event))
                }, {
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.emailAddress), 1 /* TEXT */)
                  ]),
                  caption: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._forgotPassword.enterEmail), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["spellcheck", "modelValue"]),
                _createVNode(MkButton, {
                  type: "submit",
                  rounded: "",
                  disabled: processing.value,
                  primary: "",
                  style: "margin: 0 auto;"
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.send), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["disabled"]),
                _createVNode(MkInfo, null, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._forgotPassword.ifNoEmail), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                })
              ])
            ]))
            : (_openBlock(), _createElementBlock("div", { key: 1 }, _toDisplayString(_unref(i18n).ts._forgotPassword.contactAdmin), 1 /* TEXT */))
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["width", "height"]))
}
}

})
