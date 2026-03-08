import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vModelText as _vModelText } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-floppy" })
import { onMounted, onUnmounted, nextTick, ref, watch, computed, toRefs, useTemplateRef } from 'vue'
import { debounce } from 'throttle-debounce'
import type { SuggestionType } from '@/utility/autocomplete.js'
import MkButton from '@/components/MkButton.vue'
import { i18n } from '@/i18n.js'
import { Autocomplete } from '@/utility/autocomplete.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkTextarea',
  props: {
    modelValue: { type: String, required: true },
    required: { type: Boolean, required: false },
    readonly: { type: Boolean, required: false },
    disabled: { type: Boolean, required: false },
    pattern: { type: String, required: false },
    placeholder: { type: String, required: false },
    autofocus: { type: Boolean, required: false },
    autocomplete: { type: String, required: false },
    mfmAutocomplete: { type: [Boolean, Array], required: false },
    mfmPreview: { type: Boolean, required: false },
    spellcheck: { type: Boolean, required: false },
    debounce: { type: Boolean, required: false },
    manualSave: { type: Boolean, required: false },
    code: { type: Boolean, required: false },
    tall: { type: Boolean, required: false },
    pre: { type: Boolean, required: false }
  },
  emits: ["change", "keydown", "enter", "update:modelValue", "savingStateChange"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const { modelValue, autofocus } = toRefs(props);
const v = ref<string>(modelValue.value ?? '');
const focused = ref(false);
const changed = ref(false);
const invalid = ref(false);
const filled = computed(() => v.value !== '' && v.value != null);
const inputEl = useTemplateRef('inputEl');
const preview = ref(false);
let autocompleteWorker: Autocomplete | null = null;
function focus() {
	inputEl.value?.focus();
}
function onInput(ev: InputEvent) {
	changed.value = true;
	emit('change', ev);
}
function onKeydown(ev: KeyboardEvent) {
	if (ev.isComposing || ev.key === 'Process' || ev.keyCode === 229) return;
	emit('keydown', ev);
	if (ev.code === 'Enter') {
		emit('enter');
	}
	if (props.code && ev.key === 'Tab') {
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
	emit('update:modelValue', v.value ?? '');
}
const debouncedUpdated = debounce(1000, updated);
watch(modelValue, newValue => {
	v.value = newValue ?? '';
});
watch(v, () => {
	if (!props.manualSave) {
		if (props.debounce) {
			debouncedUpdated();
		} else {
			updated();
		}
	}
	invalid.value = inputEl.value?.validity.badInput ?? true;
});
watch([changed, invalid], ([newChanged, newInvalid]) => {
	emit('savingStateChange', newChanged, newInvalid);
}, { immediate: true });
onMounted(() => {
	nextTick(() => {
		if (autofocus.value) {
			focus();
		}
	});
	if (props.mfmAutocomplete && inputEl.value) {
		autocompleteWorker = new Autocomplete(inputEl.value, v, props.mfmAutocomplete === true ? undefined : props.mfmAutocomplete);
	}
});
onUnmounted(() => {
	if (autocompleteWorker) {
		autocompleteWorker.detach();
	}
});

return (_ctx: any,_cache: any) => {
  const _component_Mfm = _resolveComponent("Mfm")
  const _directive_adaptive_border = _resolveDirective("adaptive-border")
  const _directive_panel = _resolveDirective("panel")

  return (_openBlock(), _createElementBlock("div", { class: "_selectable" }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.label),
        onClick: focus
      }, [ _renderSlot(_ctx.$slots, "label") ]), _createElementVNode("div", {
        class: _normalizeClass({ [_ctx.$style.disabled]: __props.disabled, [_ctx.$style.focused]: focused.value, [_ctx.$style.tall]: __props.tall, [_ctx.$style.pre]: __props.pre }),
        style: "position: relative;"
      }, [ _withDirectives(_createElementVNode("textarea", {
          ref_key: "inputEl", ref: inputEl,
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((v).value = $event)),
          class: _normalizeClass([_ctx.$style.textarea, { _monospace: __props.code }]),
          disabled: __props.disabled,
          required: __props.required,
          readonly: __props.readonly,
          placeholder: __props.placeholder,
          pattern: __props.pattern,
          autocomplete: __props.autocomplete,
          spellcheck: __props.spellcheck,
          onFocus: _cache[1] || (_cache[1] = ($event: any) => (focused.value = true)),
          onBlur: _cache[2] || (_cache[2] = ($event: any) => (focused.value = false)),
          onKeydown: _cache[3] || (_cache[3] = ($event: any) => (onKeydown($event))),
          onInput: onInput
        }, null, 42 /* CLASS, PROPS, NEED_HYDRATION */, ["disabled", "required", "readonly", "placeholder", "pattern", "autocomplete", "spellcheck"]), [ [_vModelText, v.value] ]) ], 2 /* CLASS */), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.caption)
      }, [ _renderSlot(_ctx.$slots, "caption") ]), (__props.mfmPreview) ? (_openBlock(), _createElementBlock("button", {
          key: 0,
          style: "font-size: 0.85em;",
          class: "_textButton",
          type: "button",
          onClick: _cache[4] || (_cache[4] = ($event: any) => (preview.value = !preview.value))
        }, _toDisplayString(_unref(i18n).ts.preview), 1 /* TEXT */)) : _createCommentVNode("v-if", true), (__props.mfmPreview) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: _normalizeClass(_ctx.$style.mfmPreview)
        }, [ _createVNode(_component_Mfm, { text: v.value }, null, 8 /* PROPS */, ["text"]) ])) : _createCommentVNode("v-if", true), (__props.manualSave && changed.value) ? (_openBlock(), _createBlock(MkButton, {
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
