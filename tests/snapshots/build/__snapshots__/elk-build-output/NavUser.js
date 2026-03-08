import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { block: "true", "i-ri:loader-2-fill": "true", "aria-hidden": "true" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", block: "true", "i-ri:login-circle-line": "true", class: "rtl-flip" })

export default /*@__PURE__*/_defineComponent({
  __name: 'NavUser',
  setup(__props) {

const { busy, oauth, singleInstanceServer } = useSignIn()

return (_ctx: any,_cache: any) => {
  const _component_AccountAvatar = _resolveComponent("AccountAvatar")
  const _component_UserSwitcher = _resolveComponent("UserSwitcher")
  const _component_VDropdown = _resolveComponent("VDropdown")
  const _component_i18n_t = _resolveComponent("i18n-t")

  return (_ctx.isHydrated && _ctx.currentUser)
      ? (_openBlock(), _createBlock(_component_VDropdown, {
        key: 0,
        "sm:hidden": ""
      }, {
        popper: _withCtx(({ hide }) => [
          _createVNode(_component_UserSwitcher, {
            onClick: _cache[0] || (_cache[0] = ($event: any) => (_ctx.hide()))
          })
        ]),
        default: _withCtx(() => [
          _createElementVNode("div", { style: "-webkit-touch-callout: none;" }, [
            _createVNode(_component_AccountAvatar, {
              account: _ctx.currentUser.account,
              "h-8": "",
              "w-8": "",
              draggable: false,
              square: ""
            }, null, 8 /* PROPS */, ["account", "draggable"])
          ])
        ]),
        _: 1 /* STABLE */
      }))
      : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ (_unref(singleInstanceServer)) ? (_openBlock(), _createElementBlock("button", {
            key: 0,
            flex: "~ row",
            "gap-x-1": "",
            "items-center": "",
            "justify-center": "",
            "btn-solid": "",
            "text-sm": "",
            "px-2": "",
            "py-1": "",
            "xl:hidden": "",
            disabled: _unref(busy),
            onClick: _cache[1] || (_cache[1] = ($event: any) => (_unref(oauth)()))
          }, [ (_unref(busy)) ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                "aria-hidden": "true",
                block: "",
                animate: "",
                "animate-spin": "",
                "preserve-3d": "",
                class: "rtl-flip"
              }, [ _hoisted_1 ])) : (_openBlock(), _createElementBlock("span", {
                key: 1,
                "aria-hidden": "true",
                block: "",
                "i-ri:login-circle-line": "",
                class: "rtl-flip"
              })), _createVNode(_component_i18n_t, { keypath: "action.sign_in_to" }, {
              default: _withCtx(() => [
                _createElementVNode("strong", null, _toDisplayString(_ctx.currentServer), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }) ])) : (_openBlock(), _createElementBlock("button", {
            key: 1,
            flex: "~ row",
            "gap-x-1": "",
            "items-center": "",
            "justify-center": "",
            "btn-solid": "",
            "text-sm": "",
            "px-2": "",
            "py-1": "",
            "xl:hidden": "",
            onClick: _cache[2] || (_cache[2] = ($event: any) => (_ctx.openSigninDialog()))
          }, [ _hoisted_2, _createTextVNode("\n      "), _toDisplayString(_ctx.$t('action.sign_in')) ])) ], 64 /* STABLE_FRAGMENT */))
}
}

})
