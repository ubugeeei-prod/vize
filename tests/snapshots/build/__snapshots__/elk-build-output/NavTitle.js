import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vShow as _vShow, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "text-xl": "true", "i-ri:arrow-left-line": "true", class: "rtl-flip" })

export default /*@__PURE__*/_defineComponent({
  __name: 'NavTitle',
  setup(__props) {

const { env } = useBuildInfo()
const router = useRouter()
const back = ref<any>('')
const nuxtApp = useNuxtApp()
function onClickLogo() {
  nuxtApp.hooks.callHook('elk-logo:click')
}
onMounted(() => {
  back.value = router.options.history.state.back
})
router.afterEach(() => {
  back.value = router.options.history.state.back
})

return (_ctx: any,_cache: any) => {
  const _component_NavLogo = _resolveComponent("NavLogo")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")

  return (_openBlock(), _createElementBlock("div", {
      flex: "",
      "justify-between": "",
      sticky: "",
      "top-0": "",
      "bg-base": "",
      "z-1": "",
      "py-4": ""
    }, [ _createVNode(_component_NuxtLink, {
        flex: "",
        "items-end": "",
        "gap-3": "",
        py2: "",
        "px-5": "",
        "text-2xl": "",
        "select-none": "",
        "focus-visible:ring": "2 current",
        to: "/home",
        onClick: _withModifiers(onClickLogo, ["prevent"])
      }, {
        default: _withCtx(() => [
          _createVNode(_component_NavLogo, {
            "shrink-0": "",
            aspect: "1/1",
            "sm:h-8": "",
            "xl:h-10": "",
            class: "rtl-flip"
          }),
          _withDirectives(_createElementVNode("div", {
            hidden: "",
            "xl:block": "",
            "text-secondary": ""
          }, [
            _createTextVNode(_toDisplayString(_ctx.$t('app_name')) + " ", 1 /* TEXT */),
            (_unref(env) !== 'release')
              ? (_openBlock(), _createElementBlock("sup", {
                key: 0,
                "text-sm": "",
                italic: "",
                "mt-1": ""
              }, _toDisplayString(_unref(env)), 1 /* TEXT */))
              : _createCommentVNode("v-if", true)
          ], 512 /* NEED_PATCH */), [
            [_vShow, _ctx.isHydrated]
          ])
        ]),
        _: 1 /* STABLE */
      }), _createElementVNode("div", {
        hidden: "",
        "xl:flex": "",
        "items-center": "",
        "me-6": "",
        "mt-2": "",
        "gap-1": ""
      }, [ _createVNode(_component_CommonTooltip, {
          content: _ctx.$t('nav.back'),
          distance: 0
        }, {
          default: _withCtx(() => [
            _createElementVNode("button", {
              type: "button",
              "aria-label": _ctx.$t('nav.back'),
              "btn-text": "",
              "p-3": "",
              class: _normalizeClass({ 'pointer-events-none op0': !back.value || back.value === '/', 'xl:flex': _ctx.$route.name !== 'tag' }),
              onClick: _cache[0] || (_cache[0] = ($event: any) => (_ctx.$router.go(-1)))
            }, [
              _hoisted_1
            ], 10 /* CLASS, PROPS */, ["aria-label"])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["content", "distance"]) ]) ]))
}
}

})
