import { defineComponent as _defineComponent } from 'vue'
import { Suspense as _Suspense, openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withCtx as _withCtx } from "vue"

import { useTemplateRef } from 'vue'
import MkModalWindow from '@/components/MkModalWindow.vue'
import MkServerSetupWizard from '@/components/MkServerSetupWizard.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkServerSetupWizardDialog',
  emits: ["closed"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const windowEl = useTemplateRef('windowEl');
function onWizardFinished() {
	windowEl.value?.close();
}
function onCloseModalWindow() {
	windowEl.value?.close();
}

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createBlock(MkModalWindow, {
      ref_key: "windowEl", ref: windowEl,
      withOkButton: false,
      okButtonDisabled: false,
      width: 500,
      height: 600,
      onClose: onCloseModalWindow,
      onClosed: _cache[0] || (_cache[0] = ($event: any) => (emit('closed')))
    }, {
      header: _withCtx(() => [
        _createTextVNode("Server setup wizard")
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-min: 20px; --MI_SPACER-max: 28px;"
        }, [
          _createVNode(_Suspense, null, {
            default: _withCtx(() => [
              _createVNode(MkServerSetupWizard, { onFinished: onWizardFinished })
            ]),
            fallback: _withCtx(() => [
              _createVNode(_component_MkLoading)
            ]),
            _: 1 /* STABLE */
          })
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["withOkButton", "okButtonDisabled", "width", "height"]))
}
}

})
