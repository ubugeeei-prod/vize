import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:arrow-right-s-line": "true", "text-white": "true" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:arrow-left-s-line": "true", "text-white": "true" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:close-line": "true", "text-white": "true" })

export default /*@__PURE__*/_defineComponent({
  __name: 'ModalMediaPreview',
  emits: ["close"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const locked = useScrollLock(document.body)
// Use to avoid strange error when directlying assigning to v-model on ModelMediaPreviewCarousel
const index = mediaPreviewIndex
const current = computed(() => mediaPreviewList.value[mediaPreviewIndex.value])
const hasNext = computed(() => index.value < mediaPreviewList.value.length - 1)
const hasPrev = computed(() => index.value > 0)
const keys = useMagicKeys()
whenever(keys.arrowLeft, prev)
whenever(keys.arrowRight, next)
function next() {
  if (hasNext.value)
    index.value++
}
function prev() {
  if (hasPrev.value)
    index.value--
}
function onClick(e: MouseEvent) {
  const path = e.composedPath() as HTMLElement[]
  const el = path.find(el => ['A', 'BUTTON', 'IMG', 'VIDEO', 'P'].includes(el.tagName?.toUpperCase()))
  if (!el)
    emit('close')
}
onMounted(() => locked.value = true)
onUnmounted(() => locked.value = false)

return (_ctx: any,_cache: any) => {
  const _component_ModalMediaPreviewCarousel = _resolveComponent("ModalMediaPreviewCarousel")

  return (_openBlock(), _createElementBlock("div", {
      relative: "",
      "h-full": "",
      "w-full": "",
      flex: "",
      "pt-12": "",
      onClick: onClick
    }, [ (hasNext.value) ? (_openBlock(), _createElementBlock("button", {
          key: 0,
          "pointer-events-auto": "",
          "btn-action-icon": "",
          bg: "black/20",
          "aria-label": _ctx.$t('action.next'),
          "hover:bg": "black/40",
          "dark:bg": "white/30",
          "dark-hover:bg": "white/20",
          absolute: "",
          top: "1/2",
          "right-1": "",
          z5: "",
          title: _ctx.$t('action.next'),
          onClick: next
        }, [ _hoisted_1 ])) : _createCommentVNode("v-if", true), (hasPrev.value) ? (_openBlock(), _createElementBlock("button", {
          key: 0,
          "pointer-events-auto": "",
          "btn-action-icon": "",
          bg: "black/20",
          "aria-label": _ctx.$t('action.prev'),
          "hover:bg": "black/40",
          "dark:bg": "white/30",
          "dark:hover-bg": "white/20",
          absolute: "",
          top: "1/2",
          "left-1": "",
          z5: "",
          title: _ctx.$t('action.prev'),
          onClick: prev
        }, [ _hoisted_2 ])) : _createCommentVNode("v-if", true), _createElementVNode("div", {
        flex: "~ col center",
        "h-full": "",
        "w-full": ""
      }, [ _createVNode(_component_ModalMediaPreviewCarousel, {
          media: _ctx.mediaPreviewList,
          onClose: _cache[0] || (_cache[0] = ($event: any) => (emit('close'))),
          modelValue: _unref(index),
          "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((index).value = $event))
        }, null, 8 /* PROPS */, ["media", "modelValue"]), _createElementVNode("div", {
          bg: "black/30",
          "dark:bg": "white/10",
          "mb-6": "",
          "mt-4": "",
          "text-white": "",
          "rounded-full": "",
          flex: "~ center shrink-0",
          "overflow-hidden": ""
        }, [ (_ctx.mediaPreviewList.length > 1) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              p: "y-1 x-3",
              "rounded-r-0": "",
              "shrink-0": ""
            }, _toDisplayString(_unref(index) + 1) + " / " + _toDisplayString(_ctx.mediaPreviewList.length), 1 /* TEXT */)) : _createCommentVNode("v-if", true), (current.value.description) ? (_openBlock(), _createElementBlock("p", {
              key: 0,
              bg: "dark/30",
              "dark:bg": "white/10",
              p: "y-1 x-3",
              "rounded-ie-full": "",
              "line-clamp-1": "",
              "ws-pre-wrap": "",
              "break-all": "",
              title: current.value.description,
              "w-full": ""
            }, _toDisplayString(current.value.description), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]) ]), _createElementVNode("div", {
        absolute: "",
        "top-0": "",
        "w-full": "",
        flex: "",
        "justify-end": ""
      }, [ _createElementVNode("button", {
          "btn-action-icon": "",
          bg: "black/30",
          "aria-label": _ctx.$t('action.close'),
          "hover:bg": "black/40",
          "dark:bg": "white/30",
          "dark:hover-bg": "white/20",
          "pointer-events-auto": "",
          "shrink-0": "",
          onClick: _cache[2] || (_cache[2] = ($event: any) => (emit('close')))
        }, [ _hoisted_3 ], 8 /* PROPS */, ["aria-label"]) ]) ]))
}
}

})
