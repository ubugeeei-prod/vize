import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { "text-sm": "true", "text-secondary": "true" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { block: "true", "i-ri:loader-2-fill": "true", "aria-hidden": "true" })

export default /*@__PURE__*/_defineComponent({
  __name: 'UserSignInEntry',
  setup(__props) {

const { busy, oauth, singleInstanceServer } = useSignIn()

return (_ctx: any,_cache: any) => {
  const _component_i18n_t = _resolveComponent("i18n-t")

  return (_openBlock(), _createElementBlock("div", {
      p8: "",
      "lg:flex": "~ col gap2",
      hidden: ""
    }, [ (_ctx.isHydrated) ? (_openBlock(), _createElementBlock("p", {
          key: 0,
          "text-sm": ""
        }, [ _createVNode(_component_i18n_t, { keypath: "user.sign_in_notice_title" }, {
            default: _withCtx(() => [
              _createElementVNode("strong", null, _toDisplayString(_ctx.currentServer), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }) ])) : _createCommentVNode("v-if", true), _createElementVNode("p", _hoisted_1, _toDisplayString(_ctx.$t(_unref(singleInstanceServer) ? 'user.single_instance_sign_in_desc' : 'user.sign_in_desc')), 1 /* TEXT */), (_unref(singleInstanceServer)) ? (_openBlock(), _createElementBlock("button", {
          key: 0,
          flex: "~ row",
          "gap-x-2": "",
          "items-center": "",
          "justify-center": "",
          "btn-solid": "",
          "text-center": "",
          "rounded-3": "",
          disabled: _unref(busy),
          onClick: _cache[0] || (_cache[0] = ($event: any) => (_unref(oauth)()))
        }, [ (_unref(busy)) ? (_openBlock(), _createElementBlock("span", {
              key: 0,
              "aria-hidden": "true",
              block: "",
              animate: "",
              "animate-spin": "",
              "preserve-3d": "",
              class: "rtl-flip"
            }, [ _hoisted_2 ])) : (_openBlock(), _createElementBlock("span", {
              key: 1,
              "aria-hidden": "true",
              block: "",
              "i-ri:login-circle-line": "",
              class: "rtl-flip"
            })), _createTextVNode("\n      "), _toDisplayString(_ctx.$t('action.sign_in')) ])) : (_openBlock(), _createElementBlock("button", {
          key: 1,
          "btn-solid": "",
          "rounded-3": "",
          "text-center": "",
          "mt-2": "",
          "select-none": "",
          onClick: _cache[1] || (_cache[1] = ($event: any) => (_ctx.openSigninDialog()))
        }, _toDisplayString(_ctx.$t('action.sign_in')), 1 /* TEXT */)) ]))
}
}

})
