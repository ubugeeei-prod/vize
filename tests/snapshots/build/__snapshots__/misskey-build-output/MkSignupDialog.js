import { defineComponent as _defineComponent } from 'vue'
import { Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"

import { useTemplateRef, ref } from 'vue'
import * as Misskey from 'misskey-js'
import XSignup from '@/components/MkSignupDialog.form.vue'
import XServerRules from '@/components/MkSignupDialog.rules.vue'
import MkModalWindow from '@/components/MkModalWindow.vue'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSignupDialog',
  props: {
    autoSet: { type: Boolean, required: false, default: false }
  },
  emits: ["done", "cancelled", "closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const dialog = useTemplateRef('dialog');
const isAcceptedServerRule = ref(false);
function onClose() {
	emit('cancelled');
	dialog.value?.close();
}
function onSignup(res: Misskey.entities.SignupResponse) {
	emit('done', res);
	dialog.value?.close();
}
function onSignupEmailPending() {
	dialog.value?.close();
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkModalWindow, {
      ref_key: "dialog", ref: dialog,
      width: 500,
      height: 600,
      onClose: onClose,
      onClosed: _cache[0] || (_cache[0] = ($event: any) => (emit('closed')))
    }, {
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(i18n).ts.signup), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", { style: "overflow-x: clip;" }, [
          _createVNode(_Transition, {
            mode: "out-in",
            enterActiveClass: _ctx.$style.transition_x_enterActive,
            leaveActiveClass: _ctx.$style.transition_x_leaveActive,
            enterFromClass: _ctx.$style.transition_x_enterFrom,
            leaveToClass: _ctx.$style.transition_x_leaveTo
          }, {
            default: _withCtx(() => [
              (!isAcceptedServerRule.value)
                ? (_openBlock(), _createBlock(XServerRules, {
                  key: 0,
                  onDone: _cache[1] || (_cache[1] = ($event: any) => (isAcceptedServerRule.value = true)),
                  onCancel: onClose
                }))
                : (_openBlock(), _createBlock(XSignup, {
                  key: 1,
                  autoSet: __props.autoSet,
                  onSignup: onSignup,
                  onSignupEmailPending: onSignupEmailPending
                }, null, 8 /* PROPS */, ["autoSet"]))
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass"])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["width", "height"]))
}
}

})
