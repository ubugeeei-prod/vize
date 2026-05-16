import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderSlot as _renderSlot, normalizeClass as _normalizeClass, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "text-lg": "true", "i-ri:arrow-left-line": "true", class: "rtl-flip" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { "sm:hidden": "true", "h-7": "true", "w-1px": "true" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("div", { hidden: "true" })

export default /*@__PURE__*/_defineComponent({
  __name: 'MainContent',
  props: {
    back: { type: [Boolean, String], required: false, default: false },
    backOnSmallScreen: { type: Boolean, required: false },
    noOverflowHidden: { type: Boolean, required: false }
  },
  setup(__props: any) {

const container = ref()
const route = useRoute()
const userSettings = useUserSettings()
const { height: windowHeight } = useWindowSize()
const { height: containerHeight } = useElementBounding(container)
const wideLayout = computed(() => route.meta.wideLayout ?? false)
const sticky = computed(() => route.path?.startsWith('/settings/'))
const containerClass = computed(() => {
  // we keep original behavior when not in settings page and when the window height is smaller than the container height
  if (!isHydrated.value || !sticky.value || (windowHeight.value < containerHeight.value))
    return null

  return 'lg:sticky lg:top-0'
})
const showBackButton = computed(() => {
  switch (__props.back) {
    case 'small-only':
      return isSmallOrMediumScreen.value
    case true:
      return !isExtraLargeScreen.value
    default:
      return false
  }
})

return (_ctx: any,_cache: any) => {
  const _component_PwaBadge = _resolveComponent("PwaBadge")
  const _component_NavUser = _resolveComponent("NavUser")
  const _component_NavUserSkeleton = _resolveComponent("NavUserSkeleton")
  const _component_PwaInstallPrompt = _resolveComponent("PwaInstallPrompt")

  return (_openBlock(), _createElementBlock("div", {
      ref_key: "container", ref: container,
      class: _normalizeClass(containerClass.value)
    }, [ _createElementVNode("div", {
        sticky: "",
        "top-0": "",
        "z-20": "",
        pt: "[env(safe-area-inset-top,0)]",
        bg: "[rgba(var(--rgb-bg-base),0.7)]",
        class: _normalizeClass({
          'backdrop-blur': !_ctx.getPreferences(_unref(userSettings), 'optimizeForLowPerformanceDevice'),
        })
      }, [ _createElementVNode("div", {
          flex: "~ justify-between",
          "min-h-53px": "",
          "px-2": "",
          "py-1": "",
          class: _normalizeClass({ 'xl:hidden': _ctx.$route.name !== 'tag' }),
          border: "b base"
        }, [ _createElementVNode("div", {
            flex: "~ items-center",
            "w-full": ""
          }, [ (__props.backOnSmallScreen || showBackButton.value) ? (_openBlock(), _createElementBlock("button", {
                key: 0,
                "btn-text": "",
                flex: "",
                "items-center": "",
                "p-3": "",
                "xl:hidden": "",
                "aria-label": _ctx.$t('nav.back'),
                onClick: _cache[0] || (_cache[0] = ($event: any) => (_ctx.$router.go(-1)))
              }, [ _hoisted_1 ])) : _createCommentVNode("v-if", true), _createElementVNode("div", {
              flex: "",
              "w-full": ""
            }, [ _renderSlot(_ctx.$slots, "title") ]), _hoisted_2 ]), _createElementVNode("div", {
            flex: "~ items-center shrink-0 gap-x-2",
            "px-3": ""
          }, [ _renderSlot(_ctx.$slots, "actions"), _createVNode(_component_PwaBadge, { "xl:hidden": "" }), (_ctx.isHydrated) ? (_openBlock(), _createBlock(_component_NavUser, { key: 0 })) : (_openBlock(), _createBlock(_component_NavUserSkeleton, { key: 1 })) ]) ]), _renderSlot(_ctx.$slots, "header", {}, () => [ _hoisted_3 ]) ], 2 /* CLASS */), _createVNode(_component_PwaInstallPrompt, { "xl:hidden": "" }), _createElementVNode("div", {
        class: _normalizeClass(_ctx.isHydrated && wideLayout.value ? 'xl:w-full sm:max-w-600px' : 'sm:max-w-600px md:shrink-0'),
        "m-auto": ""
      }, [ _createElementVNode("div", {
          hidden: "",
          class: _normalizeClass({ 'xl:block': _ctx.$route.name !== 'tag' && !_ctx.$slots.header }),
          "h-6": ""
        }), _renderSlot(_ctx.$slots, "default") ], 2 /* CLASS */) ], 2 /* CLASS */))
}
}

})
