import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Transition as _Transition, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { absolute: "true", "inset-0": "true", "opacity-0": "true", h: "[calc(100vh+0.5px)]" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { border: "neutral-300 dark:neutral-700 t-1", m: "x-3 y-2" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "i-ri:sun-line dark:i-ri:moon-line flex-shrink-0 text-xl me-4 !align-middle" })
import { invoke } from '@vueuse/core'

export default /*@__PURE__*/_defineComponent({
  __name: 'NavBottomMoreMenu',
  props: {
    "modelValue": { required: true },
    "modelModifiers": {},
  },
  emits: ["update:modelValue"],
  setup(__props) {

const modelValue = _useModel(__props, "modelValue")
const colorMode = useColorMode()
const userSettings = useUserSettings()
const drawerEl = ref<HTMLDivElement>()
function toggleVisible() {
  modelValue.value = !modelValue.value
}
const buttonEl = ref<HTMLDivElement>()
/**
 * Close the drop-down menu if the mouse click is not on the drop-down menu button when the drop-down menu is opened
 * @param mouse
 */
function clickEvent(mouse: MouseEvent) {
  if (mouse.target && !buttonEl.value?.children[0].contains(mouse.target as any)) {
    if (modelValue.value) {
      document.removeEventListener('click', clickEvent)
      modelValue.value = false
    }
  }
}
function toggleDark() {
  colorMode.preference = colorMode.value === 'dark' ? 'light' : 'dark'
}
watch(modelValue, (val) => {
  if (val && typeof document !== 'undefined')
    document.addEventListener('click', clickEvent)
})
onBeforeUnmount(() => {
  document.removeEventListener('click', clickEvent)
})
// Pull down to close
const { dragging, dragDistance } = invoke(() => {
  const triggerDistance = 120

  let scrollTop = 0
  let beforeTouchPointY = 0

  const dragDistance = ref(0)
  const dragging = ref(false)

  useEventListener(drawerEl, 'scroll', (e: Event) => {
    scrollTop = (e.target as HTMLDivElement).scrollTop

    // Prevent the page from scrolling when the drawer is being dragged.
    if (dragDistance.value > 0)
      (e.target as HTMLDivElement).scrollTop = 0
  }, { passive: true })

  useEventListener(drawerEl, 'touchstart', (e: TouchEvent) => {
    if (!modelValue.value)
      return

    beforeTouchPointY = e.touches[0].pageY
    dragDistance.value = 0
  }, { passive: true })

  useEventListener(drawerEl, 'touchmove', (e: TouchEvent) => {
    if (!modelValue.value)
      return

    // Do not move the entire drawer when its contents are not scrolled to the top.
    if (scrollTop > 0 && dragDistance.value <= 0) {
      dragging.value = false
      beforeTouchPointY = e.touches[0].pageY
      return
    }

    const { pageY } = e.touches[0]

    // Calculate the drag distance.
    dragDistance.value += pageY - beforeTouchPointY
    if (dragDistance.value < 0)
      dragDistance.value = 0
    beforeTouchPointY = pageY

    // Marked as dragging.
    if (dragDistance.value > 1)
      dragging.value = true

    // Prevent the page from scrolling when the drawer is being dragged.
    if (dragDistance.value > 0) {
      if (e?.cancelable && e?.preventDefault)
        e.preventDefault()
      e?.stopPropagation()
    }
  }, { passive: true })

  useEventListener(drawerEl, 'touchend', () => {
    if (!modelValue.value)
      return

    if (dragDistance.value >= triggerDistance)
      modelValue.value = false

    dragging.value = false
    // code
  }, { passive: true })

  return {
    dragDistance,
    dragging,
  }
})

return (_ctx: any,_cache: any) => {
  const _component_NavSide = _resolveComponent("NavSide")

  return (_openBlock(), _createElementBlock("div", {
      ref_key: "buttonEl", ref: buttonEl,
      flex: "",
      "items-center": "",
      static: ""
    }, [ _renderSlot(_ctx.$slots, "default", {
        "toggle-visible": toggleVisible,
        show: modelValue.value
      }), _createTextVNode("\n\n    " + "\n    "), _createVNode(_Transition, {
        "enter-active-class": "transition duration-250 ease-out",
        "enter-from-class": "opacity-0 children:(translate-y-full)",
        "enter-to-class": "opacity-100 children:(translate-y-0)",
        "leave-active-class": "transition duration-250 ease-in",
        "leave-from-class": "opacity-100 children:(translate-y-0)",
        "leave-to-class": "opacity-0 children:(translate-y-full)"
      }, {
        default: _withCtx(() => [
          _withDirectives(_createElementVNode("div", {
            absolute: "",
            "inset-x-0": "",
            "top-auto": "",
            "bottom-full": "",
            "z-20": "",
            "h-100vh": "",
            flex: "",
            "items-end": "",
            "of-y-scroll": "",
            "of-x-hidden": "",
            "scrollbar-hide": "",
            "overscroll-none": "",
            bg: "black/50"
          }, [
            _hoisted_1,
            _createElementVNode("div", {
              ref_key: "drawerEl", ref: drawerEl,
              style: _normalizeStyle({
              transform: _unref(dragging) ? `translateY(${_unref(dragDistance)}px)` : '',
            }),
              class: _normalizeClass({
              'duration-0': _unref(dragging),
              'duration-250': !_unref(dragging),
              'backdrop-blur-md': !_ctx.getPreferences(_unref(userSettings), 'optimizeForLowPerformanceDevice'),
            }),
              transition: "transform ease-in",
              "flex-1": "",
              "min-w-48": "",
              "py-6": "",
              mb: "-1px",
              "of-y-auto": "",
              "scrollbar-hide": "",
              "overscroll-none": "",
              "max-h": "[calc(100vh-200px)]",
              "rounded-t-lg": "",
              bg: "white/85 dark:neutral-900/85",
              "backdrop-filter": "",
              "border-t-1": "",
              "border-base": ""
            }, [
              _createVNode(_component_NavSide),
              _createTextVNode("\n\n          " + "\n          "),
              _hoisted_2,
              _createTextVNode("\n\n          " + "\n          "),
              _createElementVNode("div", { flex: "~ col gap2" }, [
                _createElementVNode("button", {
                  flex: "",
                  "flex-row": "",
                  "items-center": "",
                  block: "",
                  "px-5": "",
                  "py-2": "",
                  "focus-blue": "",
                  "w-full": "",
                  "text-sm": "",
                  "text-base": "",
                  capitalize: "",
                  "text-left": "",
                  "whitespace-nowrap": "",
                  "transition-colors": "",
                  "duration-200": "",
                  transform: "",
                  hover: "bg-gray-100 dark:(bg-gray-700 text-white)",
                  onClick: _cache[0] || (_cache[0] = ($event: any) => (toggleDark()))
                }, [
                  _hoisted_3,
                  _createTextVNode("\n              " + _toDisplayString(_unref(colorMode).value === 'light' ? _ctx.$t('menu.toggle_theme.dark') : _ctx.$t('menu.toggle_theme.light')), 1 /* TEXT */)
                ]),
                _createTextVNode("\n\n            " + "\n            "),
                _createElementVNode("button", {
                  flex: "",
                  "flex-row": "",
                  "items-center": "",
                  block: "",
                  "px-5": "",
                  "py-2": "",
                  "focus-blue": "",
                  "w-full": "",
                  "text-sm": "",
                  "text-base": "",
                  capitalize: "",
                  "text-left": "",
                  "whitespace-nowrap": "",
                  "transition-colors": "",
                  "duration-200": "",
                  transform: "",
                  hover: "bg-gray-100 dark:(bg-gray-700 text-white)",
                  "aria-label": _ctx.$t('nav.zen_mode'),
                  onClick: _cache[1] || (_cache[1] = ($event: any) => (_ctx.togglePreferences('zenMode')))
                }, [
                  _createElementVNode("span", {
                    class: _normalizeClass(["flex-shrink-0 text-xl me-4 !align-middle", _ctx.getPreferences(_unref(userSettings), 'zenMode') ? 'i-ri:layout-right-2-line' : 'i-ri:layout-right-line'])
                  }, null, 2 /* CLASS */),
                  _createTextVNode("\n              " + _toDisplayString(_ctx.$t('nav.zen_mode')), 1 /* TEXT */)
                ], 8 /* PROPS */, ["aria-label"])
              ])
            ], 6 /* CLASS, STYLE */)
          ], 512 /* NEED_PATCH */), [
            [_vShow, modelValue.value]
          ])
        ]),
        _: 1 /* STABLE */
      }) ], 512 /* NEED_PATCH */))
}
}

})
