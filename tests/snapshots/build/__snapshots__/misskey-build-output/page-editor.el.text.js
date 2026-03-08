import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vModelText as _vModelText } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-align-left" })
import { watch, ref, useTemplateRef, onMounted, onUnmounted } from 'vue'
import * as Misskey from 'misskey-js'
import XContainer from '../page-editor.container.vue'
import { i18n } from '@/i18n.js'
import { Autocomplete } from '@/utility/autocomplete.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'page-editor.el.text',
  props: {
    modelValue: { type: null, required: true }
  },
  emits: ["update:modelValue", "text", "remove"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
let autocomplete: Autocomplete;
const text = ref(props.modelValue.text ?? '');
const inputEl = useTemplateRef('inputEl');
watch(text, () => {
	emit('update:modelValue', {
		...props.modelValue,
		text: text.value,
	});
});
onMounted(() => {
	if (inputEl.value == null) return;
	autocomplete = new Autocomplete(inputEl.value, text);
});
onUnmounted(() => {
	autocomplete.detach();
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(XContainer, {
      draggable: true,
      onRemove: _cache[0] || (_cache[0] = () => emit('remove'))
    }, {
      header: _withCtx(() => [
        _hoisted_1,
        _createTextVNode(" "),
        _createTextVNode(_toDisplayString(_unref(i18n).ts._pages.blocks.text), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createElementVNode("section", null, [
          _withDirectives(_createElementVNode("textarea", {
            ref_key: "inputEl", ref: inputEl,
            "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((text).value = $event)),
            class: _normalizeClass(_ctx.$style.textarea)
          }, null, 512 /* NEED_PATCH */), [
            [_vModelText, text.value]
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["draggable"]))
}
}

})
