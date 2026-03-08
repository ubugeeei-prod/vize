import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, mergeProps as _mergeProps, withCtx as _withCtx } from "vue"

import { useTemplateRef } from 'vue'
import type { PostFormProps } from '@/types/post-form.js'
import MkModal from '@/components/MkModal.vue'
import MkPostForm from '@/components/MkPostForm.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkPostFormDialog',
  props: {
    instant: { type: Boolean, required: false },
    fixed: { type: Boolean, required: false },
    autofocus: { type: Boolean, required: false }
  },
  emits: ["closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const modal = useTemplateRef('modal');
const form = useTemplateRef('form');
function onPosted() {
	modal.value?.close({
		useSendAnimation: true,
	});
}
async function _close() {
	const canClose = await form.value?.canClose();
	if (!canClose) return;
	form.value?.abortUploader();
	modal.value?.close();
}
function onEsc() {
	_close();
}
function onBgClick() {
	_close();
}
function onModalClosed() {
	emit('closed');
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkModal, {
      ref_key: "modal", ref: modal,
      preferType: 'dialog',
      onClick: _cache[0] || (_cache[0] = ($event: any) => (onBgClick())),
      onClosed: _cache[1] || (_cache[1] = ($event: any) => (onModalClosed())),
      onEsc: onEsc
    }, {
      default: _withCtx(() => [
        _createVNode(MkPostForm, _mergeProps(props, {
          ref_key: "form", ref: form,
          class: ["_popup", _ctx.$style.form],
          autofocus: "",
          freezeAfterPosted: "",
          onPosted: onPosted,
          onCancel: _cache[2] || (_cache[2] = ($event: any) => (_close())),
          onEsc: _cache[3] || (_cache[3] = ($event: any) => (_close()))
        }), null, 16 /* FULL_PROPS */)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["preferType"]))
}
}

})
