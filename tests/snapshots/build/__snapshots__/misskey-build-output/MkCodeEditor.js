import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, withDirectives as _withDirectives, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vModelText as _vModelText } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-floppy" })
import { ref, watch, toRefs, useTemplateRef, nextTick } from 'vue'
import { debounce } from 'throttle-debounce'
import MkButton from '@/components/MkButton.vue'
import { i18n } from '@/i18n.js'
import XCode from '@/components/MkCode.core.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkCodeEditor',
  props: {
    modelValue: { type: String, required: true },
    lang: { type: String, required: false, default: 'js' },
    required: { type: Boolean, required: false },
    readonly: { type: Boolean, required: false },
    disabled: { type: Boolean, required: false },
    debounce: { type: Boolean, required: false },
    manualSave: { type: Boolean, required: false }
  },
  emits: ["change", "keydown", "enter", "update:modelValue"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const { modelValue } = toRefs(props);
const v = ref<string>(modelValue.value ?? '');
const focused = ref(false);
const changed = ref(false);
const inputEl = useTemplateRef('inputEl');
function focus() {
	inputEl.value?.focus();
}
function onInput(ev: InputEvent) {
	v.value = (inputEl.value?.value) ?? '';
	changed.value = true;
	emit('change', ev);
}
function onKeydown(ev: KeyboardEvent) {
	if (ev.isComposing || ev.key === 'Process' || ev.keyCode === 229) return;
	emit('keydown', ev);
	if (ev.code === 'Enter') {
		const pos = inputEl.value?.selectionStart ?? 0;
		const posEnd = inputEl.value?.selectionEnd ?? v.value.length;
		if (pos === posEnd) {
			const lines = v.value.slice(0, pos).split('\n');
			const currentLine = lines[lines.length - 1];
			const currentLineSpaces = currentLine.match(/^\s+/);
			const posDelta = currentLineSpaces ? currentLineSpaces[0].length : 0;
			ev.preventDefault();
			v.value = v.value.slice(0, pos) + '\n' + (currentLineSpaces ? currentLineSpaces[0] : '') + v.value.slice(pos);
			nextTick(() => {
				inputEl.value?.setSelectionRange(pos + 1 + posDelta, pos + 1 + posDelta);
			});
		}
		emit('enter');
	}
	if (ev.key === 'Tab') {
		const pos = inputEl.value?.selectionStart ?? 0;
		const posEnd = inputEl.value?.selectionEnd ?? v.value.length;
		v.value = v.value.slice(0, pos) + '\t' + v.value.slice(posEnd);
		nextTick(() => {
			inputEl.value?.setSelectionRange(pos + 1, pos + 1);
		});
		ev.preventDefault();
	}
}
function updated() {
	changed.value = false;
	emit('update:modelValue', v.value);
}
const debouncedUpdated = debounce(1000, updated);
watch(modelValue, newValue => {
	v.value = newValue ?? '';
});
watch(v, newValue => {
	if (!props.manualSave) {
		if (props.debounce) {
			debouncedUpdated();
		} else {
			updated();
		}
	}
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.label),
        onClick: focus
      }, [ _renderSlot(_ctx.$slots, "label") ]), _createElementVNode("div", {
        class: _normalizeClass([_ctx.$style.codeEditorRoot, { [_ctx.$style.focused]: focused.value }])
      }, [ _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.codeEditorScroller)
        }, [ _withDirectives(_createElementVNode("textarea", {
            ref_key: "inputEl", ref: inputEl,
            "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((v).value = $event)),
            class: _normalizeClass([_ctx.$style.textarea]),
            disabled: __props.disabled,
            required: __props.required,
            readonly: __props.readonly,
            autocomplete: "off",
            wrap: "off",
            spellcheck: "false",
            onFocus: _cache[1] || (_cache[1] = ($event: any) => (focused.value = true)),
            onBlur: _cache[2] || (_cache[2] = ($event: any) => (focused.value = false)),
            onKeydown: _cache[3] || (_cache[3] = ($event: any) => (onKeydown($event))),
            onInput: onInput
          }, null, 40 /* PROPS, NEED_HYDRATION */, ["disabled", "required", "readonly"]), [ [_vModelText, v.value] ]), _createVNode(XCode, {
            class: _normalizeClass(_ctx.$style.codeEditorHighlighter),
            codeEditor: true,
            code: v.value,
            lang: __props.lang
          }, null, 8 /* PROPS */, ["codeEditor", "code", "lang"]) ]) ], 2 /* CLASS */), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.caption)
      }, [ _renderSlot(_ctx.$slots, "caption") ]), (__props.manualSave && changed.value) ? (_openBlock(), _createBlock(MkButton, {
          key: 0,
          primary: "",
          class: _normalizeClass(_ctx.$style.save),
          onClick: updated
        }, {
          default: _withCtx(() => [
            _hoisted_1,
            _createTextVNode(" "),
            _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })) : _createCommentVNode("v-if", true) ]))
}
}

})
