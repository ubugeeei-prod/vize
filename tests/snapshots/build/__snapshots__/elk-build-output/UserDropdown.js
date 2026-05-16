import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'UserDropdown',
  setup(__props) {

const mask = useMask()

return (_ctx: any,_cache: any) => {
  const _component_AccountAvatar = _resolveComponent("AccountAvatar")
  const _component_UserSwitcher = _resolveComponent("UserSwitcher")
  const _component_VDropdown = _resolveComponent("VDropdown")

  return (_openBlock(), _createBlock(_component_VDropdown, {
      distance: 0,
      placement: "top-start",
      strategy: "fixed",
      onApplyShow: _cache[0] || (_cache[0] = ($event: any) => (_unref(mask).show())),
      onApplyHide: _cache[1] || (_cache[1] = ($event: any) => (_unref(mask).hide()))
    }, {
      popper: _withCtx(({ hide }) => [
        _createVNode(_component_UserSwitcher, {
          onClick: _cache[2] || (_cache[2] = (...args) => (_ctx.hide && _ctx.hide(...args)))
        })
      ]),
      default: _withCtx(() => [
        _createElementVNode("button", {
          "btn-action-icon": "",
          "aria-label": _ctx.$t('action.switch_account')
        }, [
          _createElementVNode("div", {
            class: _normalizeClass({ 'hidden xl:block': _ctx.currentUser }),
            "i-ri:more-2-line": ""
          }),
          (_ctx.currentUser)
            ? (_openBlock(), _createBlock(_component_AccountAvatar, {
              key: 0,
              "xl:hidden": "",
              account: _ctx.currentUser.account,
              "w-9": "",
              "h-9": "",
              square: ""
            }, null, 8 /* PROPS */, ["account"]))
            : _createCommentVNode("v-if", true)
        ], 8 /* PROPS */, ["aria-label"])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["distance"]))
}
}

})
