import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, renderList as _renderList, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import type { UserLogin } from '#shared/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'UserPicker',
  setup(__props) {

const all = useUsers()
const router = useRouter()
function clickUser(user: UserLogin) {
  if (user.account.acct === currentUser.value?.account.acct)
    router.push(getAccountRoute(user.account))
  else
    switchUser(user)
}

return (_ctx: any,_cache: any) => {
  const _component_AccountAvatar = _resolveComponent("AccountAvatar")
  const _component_AccountDisplayName = _resolveComponent("AccountDisplayName")
  const _component_AccountHandle = _resolveComponent("AccountHandle")
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")
  const _component_UserDropdown = _resolveComponent("UserDropdown")

  return (_openBlock(), _createElementBlock("div", {
      flex: "",
      "justify-start": "",
      "items-end": "",
      "px-2": "",
      "gap-5": ""
    }, [ _createElementVNode("div", {
        flex: "~ wrap-reverse",
        "gap-5": ""
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(all), (user) => {
          return (_openBlock(), _createElementBlock(_Fragment, { key: user.id }, [
            _createVNode(_component_CommonTooltip, {
              distance: 8,
              delay: { show: 300, hide: 100 }
            }, {
              popper: _withCtx(() => [
                _createElementVNode("div", { "text-center": "" }, [
                  _createElementVNode("span", { "text-4": "" }, [
                    _createVNode(_component_AccountDisplayName, { account: user.account }, null, 8 /* PROPS */, ["account"])
                  ]),
                  _createVNode(_component_AccountHandle, { account: user.account }, null, 8 /* PROPS */, ["account"])
                ])
              ]),
              default: _withCtx(() => [
                _createElementVNode("button", {
                  flex: "",
                  rounded: "",
                  "cursor-pointer": "",
                  "aria-label": _ctx.$t('action.switch_account'),
                  class: _normalizeClass(user.account.acct === _ctx.currentUser?.account.acct ? '' : 'op25 grayscale'),
                  hover: "filter-none op100",
                  onClick: _cache[0] || (_cache[0] = ($event: any) => (clickUser(user)))
                }, [
                  _createVNode(_component_AccountAvatar, {
                    "w-13": "",
                    "h-13": "",
                    account: user.account,
                    square: ""
                  }, null, 8 /* PROPS */, ["account"])
                ], 10 /* CLASS, PROPS */, ["aria-label"])
              ]),
              _: 2 /* DYNAMIC */
            }, 8 /* PROPS */, ["distance", "delay"])
          ], 64 /* STABLE_FRAGMENT */))
        }), 128 /* KEYED_FRAGMENT */)) ]), _createElementVNode("div", {
        flex: "",
        "items-center": "",
        "justify-center": "",
        "w-13": "",
        "h-13": ""
      }, [ _createVNode(_component_UserDropdown) ]) ]))
}
}

})
