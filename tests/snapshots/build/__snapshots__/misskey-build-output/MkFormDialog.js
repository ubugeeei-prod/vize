import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"

import { ref, useTemplateRef } from 'vue'
import type { Form } from '@/utility/form.js'
import MkModalWindow from '@/components/MkModalWindow.vue'
import MkForm from '@/components/MkForm.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkFormDialog',
  props: {
    title: { type: String, required: true },
    form: { type: null, required: true }
  },
  emits: ["done", "closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const dialog = useTemplateRef('dialog');
const values = ref((() => {
	const obj: Record<string, any> = {};
	for (const item in props.form) {
		if ('default' in props.form[item]) {
			obj[item] = props.form[item].default ?? null;
		} else {
			obj[item] = null;
		}
	}
	return obj;
})());
const canSave = ref(true);
function onCanSaveStateChanged(newCanSave: boolean) {
	canSave.value = newCanSave;
}
function ok() {
	if (!canSave.value) return;
	emit('done', {
		result: values.value,
	});
	dialog.value?.close();
}
function cancel() {
	emit('done', {
		canceled: true,
	});
	dialog.value?.close();
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkModalWindow, {
      ref_key: "dialog", ref: dialog,
      width: 450,
      canClose: false,
      withOkButton: true,
      okButtonDisabled: !canSave.value,
      onClick: _cache[0] || (_cache[0] = ($event: any) => (cancel())),
      onOk: _cache[1] || (_cache[1] = ($event: any) => (ok())),
      onClose: _cache[2] || (_cache[2] = ($event: any) => (cancel())),
      onClosed: _cache[3] || (_cache[3] = ($event: any) => (emit('closed')))
    }, {
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(__props.title), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-min: 20px; --MI_SPACER-max: 32px;"
        }, [
          _createVNode(MkForm, {
            form: __props.form,
            onCanSaveStateChange: onCanSaveStateChanged,
            modelValue: values.value,
            "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((values).value = $event))
          }, null, 8 /* PROPS */, ["form", "modelValue"])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["width", "canClose", "withOkButton", "okButtonDisabled"]))
}
}

})
