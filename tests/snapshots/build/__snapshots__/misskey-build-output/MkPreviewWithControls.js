import { useSlots as _useSlots } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderSlot as _renderSlot, normalizeClass as _normalizeClass, unref as _unref } from "vue"

import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkPreviewWithControls',
  props: {
    previewLoading: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.container)
      }, [ _createElementVNode("div", {
          class: _normalizeClass([_ctx.$style.preview, _unref(prefer).s.animation ? _ctx.$style.animatedBg : null])
        }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.previewContent)
          }, [ _renderSlot(_ctx.$slots, "preview") ]), (__props.previewLoading) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(_ctx.$style.previewLoading)
            }, [ _createVNode(_component_MkLoading) ])) : _createCommentVNode("v-if", true) ], 2 /* CLASS */), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.controls)
        }, [ _renderSlot(_ctx.$slots, "controls") ]) ]) ]))
}
}

})
