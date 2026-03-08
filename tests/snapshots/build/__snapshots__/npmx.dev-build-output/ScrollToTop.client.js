import { defineComponent as _defineComponent } from 'vue'
import { Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:arrow-up w-5 h-5", "aria-hidden": "true" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:arrow-up w-5 h-5", "aria-hidden": "true" })

export default /*@__PURE__*/_defineComponent({
  __name: 'ScrollToTop.client',
  setup(__props) {

const route = useRoute()
// Pages where scroll-to-top should NOT be shown
const excludedRoutes = new Set(['index', 'code'])
const isPackagePage = computed(() => route.name === 'package' || route.name === 'package-version')
const isActive = computed(() => !excludedRoutes.has(route.name as string) && !isPackagePage.value)
const isMounted = useMounted()
const { scrollToTop, isTouchDeviceClient } = useScrollToTop()
const { y: scrollTop } = useScroll(window)
const isVisible = computed(() => {
  if (supportsScrollStateQueries.value) return false
  return scrollTop.value > SCROLL_TO_TOP_THRESHOLD
})
const { isSupported: supportsScrollStateQueries } = useCssSupports(
  'container-type',
  'scroll-state',
  { ssrValue: false },
)
const shouldShowButton = computed(() => isActive.value && isTouchDeviceClient.value)

return (_ctx: any,_cache: any) => {
  return (shouldShowButton.value && _unref(supportsScrollStateQueries))
      ? (_openBlock(), _createElementBlock("button", {
        key: 0,
        type: "button",
        class: "scroll-to-top-css fixed bottom-4 inset-ie-4 z-50 w-12 h-12 bg-bg-elevated border border-border rounded-full shadow-lg flex items-center justify-center text-fg-muted hover:text-fg transition-colors active:scale-95",
        "aria-label": _ctx.$t('common.scroll_to_top'),
        onClick: _cache[0] || (_cache[0] = (...args) => (scrollToTop && scrollToTop(...args)))
      }, [ _hoisted_1 ]))
      : (_openBlock(), _createBlock(_Transition, {
        key: 1,
        "enter-active-class": "transition-all duration-200",
        "enter-from-class": "opacity-0 translate-y-2",
        "enter-to-class": "opacity-100 translate-y-0",
        "leave-active-class": "transition-all duration-200",
        "leave-from-class": "opacity-100 translate-y-0",
        "leave-to-class": "opacity-0 translate-y-2"
      }, {
        default: _withCtx(() => [
          (shouldShowButton.value && _unref(isMounted) && isVisible.value)
            ? (_openBlock(), _createElementBlock("button", {
              key: 0,
              type: "button",
              class: "fixed bottom-4 inset-ie-4 z-50 w-12 h-12 bg-bg-elevated border border-border rounded-full shadow-lg flex items-center justify-center text-fg-muted hover:text-fg transition-colors active:scale-95",
              "aria-label": _ctx.$t('common.scroll_to_top'),
              onClick: _cache[1] || (_cache[1] = () => _unref(scrollToTop)())
            }, [
              _hoisted_2
            ]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }))
}
}

})
