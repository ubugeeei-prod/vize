import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "flex-auto": "true" })
import type { UserLogin } from '#shared/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'UserSwitcher',
  emits: ["click"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const all = useUsers()
const { singleInstanceServer, oauth } = useSignIn()
const sorted = computed(() => {
  return [
    currentUser.value!,
    ...all.value.filter(account => account.token !== currentUser.value?.token),
  ].filter(Boolean)
})
const router = useRouter()
function clickUser(user: UserLogin) {
  if (user.account.id === currentUser.value?.account.id)
    router.push(getAccountRoute(user.account))
  else
    switchUser(user)
}
function processSignIn() {
  if (singleInstanceServer)
    oauth()
  else
    openSigninDialog()
}

return (_ctx: any,_cache: any) => {
  const _component_AccountInfo = _resolveComponent("AccountInfo")
  const _component_CommonDropdownItem = _resolveComponent("CommonDropdownItem")

  return (_openBlock(), _createElementBlock("div", {
      "sm:min-w-80": "",
      "max-w-100vw": "",
      mxa: "",
      py2: "",
      flex: "~ col",
      onClick: _cache[0] || (_cache[0] = ($event: any) => (emit('click')))
    }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(sorted.value, (user) => {
        return (_openBlock(), _createElementBlock("button", {
          key: user.id,
          flex: "",
          rounded: "",
          px4: "",
          py3: "",
          "text-left": "",
          "hover:bg-active": "",
          "cursor-pointer": "",
          "transition-100": "",
          "aria-label": _ctx.$t('action.switch_account'),
          onClick: _cache[1] || (_cache[1] = ($event: any) => (clickUser(user)))
        }, [
          _createVNode(_component_AccountInfo, {
            account: user.account,
            "hover-card": false,
            square: ""
          }, null, 8 /* PROPS */, ["account", "hover-card"]),
          _hoisted_1,
          (user.token === _ctx.currentUser?.token)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              "i-ri:check-line": "",
              "text-primary": "",
              mya: "",
              "text-2xl": ""
            }))
            : _createCommentVNode("v-if", true)
        ], 8 /* PROPS */, ["aria-label"]))
      }), 128 /* KEYED_FRAGMENT */)), _createElementVNode("div", {
        border: "t base",
        pt2: ""
      }, [ _createVNode(_component_CommonDropdownItem, {
          is: "button",
          text: _ctx.$t('user.add_existing'),
          icon: "i-ri:user-add-line",
          "w-full": "",
          onClick: processSignIn
        }, null, 8 /* PROPS */, ["text"]), (_ctx.isHydrated && _ctx.currentUser) ? (_openBlock(), _createBlock(_component_CommonDropdownItem, {
            key: 0,
            is: "button",
            text: _ctx.$t('user.sign_out_account', [_ctx.getFullHandle(_ctx.currentUser.account)]),
            icon: "i-ri:logout-box-line rtl-flip",
            "w-full": "",
            onClick: _cache[2] || (_cache[2] = (...args) => (_ctx.signOut && _ctx.signOut(...args)))
          }, null, 8 /* PROPS */, ["text"])) : _createCommentVNode("v-if", true) ]) ]))
}
}

})
