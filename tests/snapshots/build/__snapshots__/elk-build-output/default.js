import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "flex-auto": "true" })
const _hoisted_2 = { "text-size-lg": "true", "text-primary": "true", "font-bold": "true" }
const _hoisted_3 = { "text-secondary": "true" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("div", { "flex-auto": "true" })
import { usePreferences } from '~/composables/settings'

export default /*@__PURE__*/_defineComponent({
  __name: 'default',
  setup(__props) {

const route = useRoute()
const info = useBuildInfo()
const wideLayout = computed(() => route.meta.wideLayout ?? false)
const showUserPicker = logicAnd(
  usePreferences('experimentalUserPicker'),
  () => useUsers().value.length > 1,
)
const isGrayscale = usePreferences('grayscaleMode')
const instance = computed(() => instanceStorage.value[currentServer.value])

return (_ctx: any,_cache: any) => {
  const _component_NavTitle = _resolveComponent("NavTitle")
  const _component_NavSide = _resolveComponent("NavSide")
  const _component_UserSignInEntry = _resolveComponent("UserSignInEntry")
  const _component_UserPicker = _resolveComponent("UserPicker")
  const _component_AccountInfo = _resolveComponent("AccountInfo")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_UserDropdown = _resolveComponent("UserDropdown")
  const _component_CommonOfflineChecker = _resolveComponent("CommonOfflineChecker")
  const _component_NavBottom = _resolveComponent("NavBottom")
  const _component_SearchWidget = _resolveComponent("SearchWidget")
  const _component_PwaPrompt = _resolveComponent("PwaPrompt")
  const _component_PwaInstallPrompt = _resolveComponent("PwaInstallPrompt")
  const _component_LazyCommonPreviewPrompt = _resolveComponent("LazyCommonPreviewPrompt")
  const _component_NavFooter = _resolveComponent("NavFooter")
  const _component_ModalContainer = _resolveComponent("ModalContainer")

  return (_openBlock(), _createElementBlock("div", {
      "h-full": "",
      "data-mode": _ctx.isHydrated && _unref(isGrayscale) ? 'grayscale' : ''
    }, [ _createElementVNode("main", {
        flex: "",
        "w-full": "",
        mxa: "",
        "lg:max-w-80rem": ""
      }, [ _createElementVNode("aside", {
          class: "w-1/8 md:w-1/6 lg:w-1/5 xl:w-1/4 zen-hide",
          hidden: "",
          "sm:flex": "",
          "justify-end": "",
          "xl:me-4": "",
          relative: ""
        }, [ _createElementVNode("div", {
            sticky: "",
            "top-0": "",
            "w-20": "",
            "xl:w-100": "",
            "h-100dvh": "",
            flex: "~ col",
            "lt-xl-items-center": ""
          }, [ _renderSlot(_ctx.$slots, "left", {}, () => [ _createElementVNode("div", {
                flex: "~ col",
                "overflow-y-auto": "",
                "justify-between": "",
                "h-full": "",
                "max-w-full": "",
                "overflow-x-hidden": ""
              }, [ _createVNode(_component_NavTitle), _createVNode(_component_NavSide, { command: "" }), _hoisted_1, (_ctx.isHydrated) ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    flex: "",
                    "flex-col": "",
                    sticky: "",
                    "bottom-0": "",
                    "bg-base": ""
                  }, [ _createElementVNode("div", {
                      hidden: "",
                      "xl:block": ""
                    }, [ (!_ctx.currentUser) ? (_openBlock(), _createBlock(_component_UserSignInEntry, { key: 0 })) : _createCommentVNode("v-if", true) ]), (_ctx.currentUser) ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        p6: "",
                        pb8: "",
                        "w-full": ""
                      }, [ _createElementVNode("div", {
                          hidden: "",
                          "xl-block": ""
                        }, [ (_unref(showUserPicker)) ? (_openBlock(), _createBlock(_component_UserPicker, { key: 0 })) : (_openBlock(), _createElementBlock("div", {
                              key: 1,
                              flex: "~",
                              "items-center": "",
                              "justify-between": ""
                            }, [ _createVNode(_component_NuxtLink, {
                                hidden: "",
                                "xl:block": "",
                                "rounded-3": "",
                                "text-primary": "",
                                "text-start": "",
                                "w-full": "",
                                "hover:bg-active": "",
                                "cursor-pointer": "",
                                "transition-100": "",
                                to: _ctx.getAccountRoute(_ctx.currentUser.account)
                              }, {
                                default: _withCtx(() => [
                                  _createVNode(_component_AccountInfo, {
                                    account: _ctx.currentUser.account,
                                    "md:break-words": "",
                                    square: ""
                                  }, null, 8 /* PROPS */, ["account"])
                                ]),
                                _: 1 /* STABLE */
                              }, 8 /* PROPS */, ["to"]), _createVNode(_component_UserDropdown) ])) ]), _createVNode(_component_UserDropdown, { "xl:hidden": "" }) ])) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true) ]) ]) ]) ]), _createElementVNode("div", {
          "w-full": "",
          "min-h-screen": "",
          class: _normalizeClass(_ctx.isHydrated && wideLayout.value ? 'xl:w-full sm:w-600px' : 'sm:w-600px md:shrink-0'),
          "border-base": ""
        }, [ _createElementVNode("div", {
            "min-h": "[calc(100vh-3.5rem)]",
            "sm:min-h-screen": ""
          }, [ _renderSlot(_ctx.$slots, "default") ]), _createElementVNode("div", {
            sticky: "",
            "left-0": "",
            "right-0": "",
            "bottom-0": "",
            "z-10": "",
            "bg-base": "",
            pb: "[env(safe-area-inset-bottom)]",
            transition: "padding 20"
          }, [ (_ctx.isHydrated) ? (_openBlock(), _createBlock(_component_CommonOfflineChecker, { key: 0 })) : _createCommentVNode("v-if", true), (_ctx.isHydrated) ? (_openBlock(), _createBlock(_component_NavBottom, {
                key: 0,
                "sm:hidden": ""
              })) : _createCommentVNode("v-if", true) ]) ], 2 /* CLASS */), (_ctx.isHydrated && !wideLayout.value) ? (_openBlock(), _createElementBlock("aside", {
            key: 0,
            class: "hidden lg:w-1/5 xl:w-1/4 sm:none xl:block zen-hide"
          }, [ _createElementVNode("div", {
              sticky: "",
              "top-0": "",
              "h-100dvh": "",
              flex: "~ col",
              "gap-2": "",
              py3: "",
              "ms-2": ""
            }, [ _renderSlot(_ctx.$slots, "right", {}, () => [ _createVNode(_component_SearchWidget, {
                  "mt-4": "",
                  "mx-1": "",
                  hidden: "",
                  "xl:block": ""
                }), _createTextVNode("\n\n            "), _createTextVNode("\n            "), (!_ctx.currentUser && instance.value) ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    grid: "",
                    "gap-3": "",
                    m3: ""
                  }, [ _createElementVNode("span", _hoisted_2, _toDisplayString(instance.value.title), 1 /* TEXT */), _createElementVNode("img", {
                      "rounded-3": "",
                      src: instance.value.thumbnail?.url
                    }, null, 8 /* PROPS */, ["src"]), _createElementVNode("p", _hoisted_3, _toDisplayString(instance.value.description), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true), _hoisted_4, _createVNode(_component_PwaPrompt), _createVNode(_component_PwaInstallPrompt), (_unref(info).env === 'preview') ? (_openBlock(), _createBlock(_component_LazyCommonPreviewPrompt, { key: 0 })) : _createCommentVNode("v-if", true), _createVNode(_component_NavFooter) ]) ]) ])) : _createCommentVNode("v-if", true) ]), _createVNode(_component_ModalContainer) ], 8 /* PROPS */, ["data-mode"]))
}
}

})
