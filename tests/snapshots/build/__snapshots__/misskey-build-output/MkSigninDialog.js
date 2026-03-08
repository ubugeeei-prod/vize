import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-login-2" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
import * as Misskey from 'misskey-js'
import { useTemplateRef } from 'vue'
import type { OpenOnRemoteOptions } from '@/utility/please-login.js'
import MkSignin from '@/components/MkSignin.vue'
import MkModal from '@/components/MkModal.vue'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSigninDialog',
  props: {
    autoSet: { type: Boolean, required: false, default: false },
    message: { type: String, required: false, default: '' },
    openOnRemote: { type: null, required: false, default: undefined },
    initialUsername: { type: String, required: false, default: undefined }
  },
  emits: ["done", "closed", "cancelled"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const modal = useTemplateRef('modal');
function onClose() {
	emit('cancelled');
	if (modal.value) modal.value.close();
}
function onLogin(res: Misskey.entities.SigninFlowResponse & { finished: true }) {
	emit('done', res);
	if (modal.value) modal.value.close();
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkModal, {
      ref_key: "modal", ref: modal,
      preferType: 'dialog',
      onClick: onClose,
      onClosed: _cache[0] || (_cache[0] = ($event: any) => (emit('closed')))
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.root)
        }, [
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.header)
          }, [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.headerText)
            }, [
              _hoisted_1,
              _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.login), 1 /* TEXT */)
            ]),
            _createElementVNode("button", {
              class: _normalizeClass(["_button", _ctx.$style.closeButton]),
              onClick: onClose
            }, [
              _hoisted_2
            ])
          ]),
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.content)
          }, [
            _createVNode(MkSignin, {
              autoSet: __props.autoSet,
              message: __props.message,
              openOnRemote: __props.openOnRemote,
              initialUsername: __props.initialUsername,
              onLogin: onLogin
            }, null, 8 /* PROPS */, ["autoSet", "message", "openOnRemote", "initialUsername"])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["preferType"]))
}
}

})
