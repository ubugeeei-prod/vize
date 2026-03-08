import { defineComponent as _defineComponent } from 'vue'
import { Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-mail-check" })
import { ref } from 'vue'
import MkButton from '@/components/MkButton.vue'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'verify-email',
  props: {
    code: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const submitting = ref(false);
const succeeded = ref(false);
function submit() {
	if (submitting.value) return;
	submitting.value = true;
	misskeyApi('verify-email', {
		code: props.code,
	}).then(() => {
		succeeded.value = true;
		submitting.value = false;
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
            _createVNode(_Transition, {
              mode: "out-in",
              enterActiveClass: _ctx.$style.transition_enterActive,
              leaveActiveClass: _ctx.$style.transition_leaveActive,
              enterFromClass: _ctx.$style.transition_enterFrom,
              leaveToClass: _ctx.$style.transition_leaveTo
            }, {
              default: _withCtx(() => [
                (!succeeded.value)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: "input",
                    class: "_gaps_m",
                    style: "padding: 32px;"
                  }, [
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.mainText)
                    }, _toDisplayString(_unref(i18n).tsx.clickToFinishEmailVerification({ ok: _unref(i18n).ts.gotIt })), 1 /* TEXT */),
                    _createElementVNode("div", null, [
                      _createVNode(MkButton, {
                        gradate: "",
                        large: "",
                        rounded: "",
                        type: "submit",
                        disabled: submitting.value,
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
                  ]))
                  : (_openBlock(), _createElementBlock("div", {
                    key: "success",
                    class: "_gaps_m",
                    style: "padding: 32px;"
                  }, [
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.mainText)
                    }, _toDisplayString(_unref(i18n).ts.emailVerified), 1 /* TEXT */),
                    _createElementVNode("div", null, [
                      _createVNode(MkButton, {
                        large: "",
                        rounded: "",
                        link: "",
                        to: "/",
                        linkBehavior: "browser",
                        style: "margin: 0 auto;"
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.goToMisskey), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      })
                    ])
                  ]))
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass"])
          ], 32 /* NEED_HYDRATION */)
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
