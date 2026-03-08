import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-user-check" })
import { ref } from 'vue'
import MkButton from '@/components/MkButton.vue'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { login } from '@/accounts.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'signup-complete',
  props: {
    code: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const submitting = ref(false);
function submit() {
	if (submitting.value) return;
	submitting.value = true;
	misskeyApi('signup-pending', {
		code: props.code,
	}).then(res => {
		return login(res.i, '/');
	}).catch(() => {
		submitting.value = false;
		os.alert({
			type: 'error',
			title: i18n.ts.somethingHappened,
			text: i18n.ts.emailVerificationFailedError,
		});
	});
}

return (_ctx: any,_cache: any) => {
  const _component_MkEllipsis = _resolveComponent("MkEllipsis")
  const _component_PageWithAnimBg = _resolveComponent("PageWithAnimBg")

  return (_openBlock(), _createBlock(_component_PageWithAnimBg, null, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.formContainer)
        }, [
          _createElementVNode("form", {
            class: _normalizeClass(["_panel", _ctx.$style.form]),
            onSubmit: _cache[0] || (_cache[0] = _withModifiers(($event: any) => (submit()), ["prevent"]))
          }, [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.banner)
            }, [
              _hoisted_1
            ]),
            _createElementVNode("div", {
              class: "_gaps_m",
              style: "padding: 32px;"
            }, [
              _createElementVNode("div", null, _toDisplayString(_unref(i18n).tsx.clickToFinishEmailVerification({ ok: _unref(i18n).ts.gotIt })), 1 /* TEXT */),
              _createElementVNode("div", null, [
                _createVNode(MkButton, {
                  gradate: "",
                  large: "",
                  rounded: "",
                  type: "submit",
                  disabled: submitting.value,
                  "data-cy-admin-ok": "",
                  style: "margin: 0 auto;"
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(submitting.value ? _unref(i18n).ts.processing : _unref(i18n).ts.gotIt), 1 /* TEXT */),
                    (submitting.value)
                      ? (_openBlock(), _createBlock(_component_MkEllipsis, { key: 0 }))
                      : _createCommentVNode("v-if", true)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["disabled"])
              ])
            ])
          ], 32 /* NEED_HYDRATION */)
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
