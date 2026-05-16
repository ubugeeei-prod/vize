import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Teleport as _Teleport, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderSlot as _renderSlot, mergeProps as _mergeProps, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"

import { useFocusTrap } from '@vueuse/integrations/useFocusTrap'

export default /*@__PURE__*/Object.assign({
  inheritAttrs: false,
}, {
  __name: 'ModalDialog',
  props: {
    zIndex: { type: Number, required: false, default: 100 },
    closeByMask: { type: Boolean, required: false, default: true },
    useVIf: { type: Boolean, required: false, default: true },
    keepAlive: { type: Boolean, required: false, default: false },
    dialogLabelledBy: { type: String, required: false },
    focusFirstElement: { type: Boolean, required: false, default: true },
    "modelValue": { required: true }
  },
  emits: ["close", "update:modelValue"],
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const visible = _useModel(__props, "modelValue")
const deactivated = useDeactivated()
const route = useRoute()
const userSettings = useUserSettings()
/** scrollable HTML element */
const elDialogMain = ref<HTMLDivElement>()
const elDialogRoot = ref<HTMLDivElement>()
const { activate } = useFocusTrap(elDialogRoot, {
  immediate: false,
  allowOutsideClick: true,
  clickOutsideDeactivates: true,
  escapeDeactivates: true,
  preventScroll: true,
  returnFocusOnDeactivate: true,
  initialFocus: __props.focusFirstElement ? undefined : false,
})
/** close the dialog */
function close() {
  if (!visible.value)
    return
  visible.value = false
  emit('close')
}
function clickMask() {
  if (__props.closeByMask)
    close()
}
const routePath = ref(route.path)
watch(visible, (value) => {
  if (value)
    routePath.value = route.path
})
const notInCurrentPage = computed(() => deactivated.value || routePath.value !== route.path)
watch(notInCurrentPage, (value) => {
  if (__props.keepAlive)
    return
  if (value)
    close()
})
// controls the state of v-if.
// when useVIf is toggled, v-if has the same state as modelValue, otherwise v-if is true
const isVIf = computed(() => {
  return __props.useVIf
    ? visible.value
    : true
})
// controls the state of v-show.
// when useVIf is toggled, v-show is true, otherwise it has the same state as modelValue
const isVShow = computed(() => {
  return !__props.useVIf
    ? visible.value
    : true
})
function bindTypeToAny($attrs: any) {
  return $attrs as any
}
function trapFocusDialog() {
  if (isVShow.value)
    nextTick().then(() => activate())
}
useEventListener('keydown', (e: KeyboardEvent) => {
  if (!visible.value)
    return
  if (e.key === 'Escape') {
    close()
    e.preventDefault()
  }
})
__expose({
  elDialogRoot,
  elDialogMain,
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(_Teleport, { to: "body" }, [ _createVNode(_Transition, {
        name: "dialog-visible",
        onTransitionend: trapFocusDialog
      }, {
        default: _withCtx(() => [
          (isVIf.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              ref: "elDialogRoot",
              "aria-modal": "true",
              "aria-labelledby": __props.dialogLabelledBy,
              style: _normalizeStyle({
            'z-index': __props.zIndex,
          }),
              fixed: "",
              "inset-0": "",
              "of-y-auto": "",
              "scrollbar-hide": "",
              "overscroll-none": ""
            }, [
              _createElementVNode("div", {
                class: _normalizeClass(["dialog-mask", {
              'backdrop-blur-sm': !_ctx.getPreferences(_unref(userSettings), 'optimizeForLowPerformanceDevice'),
            }]),
                absolute: "",
                "inset-0": "",
                "z-0": "",
                "bg-transparent": "",
                "opacity-100": "",
                "backdrop-filter": "",
                "touch-none": ""
              }, null, 2 /* CLASS */),
              _createTextVNode("\n        "),
              _createTextVNode("\n        "),
              _createElementVNode("div", {
                class: "dialog-mask",
                absolute: "",
                "inset-0": "",
                "z-0": "",
                "bg-black": "",
                "opacity-48": "",
                "touch-none": "",
                h: "[calc(100%+0.5px)]",
                onClick: clickMask
              }),
              _createTextVNode("\n        "),
              _createTextVNode("\n        "),
              _createElementVNode("div", {
                class: "p-safe-area",
                absolute: "",
                "inset-0": "",
                "z-1": "",
                "pointer-events-none": "",
                "opacity-100": "",
                flex: ""
              }, [
                _createElementVNode("div", {
                  "flex-1": "",
                  flex: "",
                  "items-center": "",
                  "justify-center": "",
                  "p-4": ""
                }, [
                  _createElementVNode("div", _mergeProps(bindTypeToAny(_ctx.$attrs), {
                    ref_key: "elDialogMain", ref: elDialogMain,
                    class: "dialog-main rounded shadow-lg pointer-events-auto isolate bg-base border-base border-1px border-solid w-full max-h-full of-y-auto overscroll-contain touch-pan-y touch-pan-x"
                  }), [
                    _renderSlot(_ctx.$slots, "default")
                  ], 16 /* FULL_PROPS */)
                ])
              ])
            ]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }) ]))
}
}

})
