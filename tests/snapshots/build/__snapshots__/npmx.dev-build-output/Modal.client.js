import { defineComponent as _defineComponent } from 'vue'
import { Teleport as _Teleport, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderSlot as _renderSlot, toDisplayString as _toDisplayString, mergeProps as _mergeProps } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'Modal.client',
  props: {
    modalTitle: { type: String, required: true },
    modalSubtitle: { type: String, required: false }
  },
  emits: ["transitioned"],
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const dialogRef = useTemplateRef('dialogRef')
const modalTitleId = computed(() => {
  const id = getCurrentInstance()?.attrs.id
  return id ? `${id}-title` : undefined
})
const modalSubtitleId = computed(() => {
  if (!props.modalSubtitle) return undefined
  const id = getCurrentInstance()?.attrs.id
  return id ? `${id}-subtitle` : undefined
})
function handleModalClose() {
  dialogRef.value?.close()
}
/**
 * Emits `transitioned` once the dialog has finished its open opacity transition.
 * This is used by consumers that need to run layout-sensitive logic (for example
 * dispatching a resize) only after the modal is fully displayed.
 */
function onDialogTransitionEnd(event: TransitionEvent) {
  const el = dialogRef.value
  if (!el) return
  if (!el.open) return
  if (event.target !== el) return
  if (event.propertyName !== 'opacity') return
  emit('transitioned')
}
__expose({
  showModal: () => dialogRef.value?.showModal(),
  close: () => dialogRef.value?.close(),
})

return (_ctx: any,_cache: any) => {
  const _component_ButtonBase = _resolveComponent("ButtonBase")

  return (_openBlock(), _createBlock(_Teleport, { to: "body" }, [ _createElementVNode("dialog", _mergeProps(_ctx.$attrs, {
        ref_key: "dialogRef", ref: dialogRef,
        closedby: "any",
        class: "w-[calc(100%-2rem)] bg-bg border border-border rounded-lg shadow-xl max-h-[90vh] overflow-y-auto overscroll-contain m-0 m-auto p-6 text-fg focus-visible:outline focus-visible:outline-accent/70",
        "aria-labelledby": modalTitleId.value,
        "aria-describedby": modalSubtitleId.value,
        onTransitionend: onDialogTransitionEnd
      }), [ _createElementVNode("div", { class: "flex items-center justify-between mb-6" }, [ _createElementVNode("div", null, [ _createElementVNode("h2", {
              id: modalTitleId.value,
              class: "font-mono text-lg font-medium"
            }, _toDisplayString(__props.modalTitle), 9 /* TEXT, PROPS */, ["id"]), (__props.modalSubtitle) ? (_openBlock(), _createElementBlock("p", {
                key: 0,
                id: modalSubtitleId.value,
                class: "text-xs text-fg-subtle"
              }, _toDisplayString(__props.modalSubtitle), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]), _createVNode(_component_ButtonBase, {
            type: "button",
            "aria-label": _ctx.$t('common.close'),
            onClick: handleModalClose,
            classicon: "i-lucide:x"
          }, null, 8 /* PROPS */, ["aria-label"]) ]), _createTextVNode("\n      " + "\n      "), _renderSlot(_ctx.$slots, "default") ], 48 /* FULL_PROPS, NEED_HYDRATION */, ["aria-labelledby", "aria-describedby"]) ]))
}
}

})
