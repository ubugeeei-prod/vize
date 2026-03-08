import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-fingerprint" })
import { ref, onMounted } from 'vue'
import { get as webAuthnRequest } from '@github/webauthn-json/browser-ponyfill'
import { i18n } from '@/i18n.js'
import MkButton from '@/components/MkButton.vue'
import type { AuthenticationPublicKeyCredential } from '@github/webauthn-json/browser-ponyfill'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSignin.passkey',
  props: {
    credentialRequest: { type: null, required: true },
    isPerformingPasswordlessLogin: { type: Boolean, required: false }
  },
  emits: ["done", "useTotp"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const queryingKey = ref(true);
async function queryKey() {
	queryingKey.value = true;
	await webAuthnRequest(props.credentialRequest)
		.catch(() => {
			return Promise.reject(null);
		})
		.then((credential) => {
			emit('done', credential);
		})
		.finally(() => {
			queryingKey.value = false;
		});
}
onMounted(() => {
	queryKey();
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.wrapper)
    }, [ _createElementVNode("div", {
        class: _normalizeClass(["_gaps", _ctx.$style.root])
      }, [ _createElementVNode("div", { class: "_gaps_s" }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.passkeyIcon)
          }, [ _hoisted_1 ]), _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.passkeyDescription)
          }, _toDisplayString(_unref(i18n).ts.useSecurityKey), 1 /* TEXT */) ]), _createVNode(MkButton, {
          large: "",
          primary: "",
          rounded: "",
          disabled: queryingKey.value,
          style: "margin: 0 auto;",
          onClick: queryKey
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.retry), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["disabled"]), (__props.isPerformingPasswordlessLogin !== true) ? (_openBlock(), _createBlock(MkButton, {
            key: 0,
            transparent: "",
            rounded: "",
            disabled: queryingKey.value,
            style: "margin: 0 auto;",
            onClick: _cache[0] || (_cache[0] = ($event: any) => (emit('useTotp')))
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.useTotp), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["disabled"])) : _createCommentVNode("v-if", true) ]) ]))
}
}

})
