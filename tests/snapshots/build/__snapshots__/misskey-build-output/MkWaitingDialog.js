import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"

import { watch, useTemplateRef } from 'vue'
import MkModal from '@/components/MkModal.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkWaitingDialog',
  props: {
    success: { type: Boolean, required: true },
    showing: { type: Boolean, required: true },
    text: { type: String, required: false }
  },
  emits: ["done", "closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const modal = useTemplateRef('modal');
function done() {
	emit('done');
	modal.value?.close();
}
watch(() => props.showing, () => {
	if (!props.showing) done();
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_MkEllipsis = _resolveComponent("MkEllipsis")

  return (_openBlock(), _createBlock(MkModal, {
      ref_key: "modal", ref: modal,
      preferType: 'dialog',
      zPriority: 'high',
      onClick: _cache[0] || (_cache[0] = __props.success ? done() : () => {}),
      onClosed: _cache[1] || (_cache[1] = ($event: any) => (emit('closed')))
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.iconOnly]: (__props.text == null) || __props.success }])
        }, [
          (__props.success)
            ? (_openBlock(), _createElementBlock("i", {
              key: 0,
              class: _normalizeClass(["ti ti-check", [_ctx.$style.icon, _ctx.$style.success]])
            }))
            : (_openBlock(), _createBlock(_component_MkLoading, {
              key: 1,
              class: _normalizeClass([_ctx.$style.icon, _ctx.$style.waiting]),
              em: true
            }, null, 8 /* PROPS */, ["em"])),
          (__props.text && !__props.success)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(_ctx.$style.text)
            }, [
              _toDisplayString(__props.text),
              _createVNode(_component_MkEllipsis)
            ]))
            : _createCommentVNode("v-if", true)
        ], 2 /* CLASS */)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["preferType", "zPriority"]))
}
}

})
