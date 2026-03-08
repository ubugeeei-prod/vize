import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vModelText as _vModelText } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
import { onMounted, onUnmounted, nextTick, ref, useTemplateRef, watch, computed, toRefs } from 'vue'
import { debounce } from 'throttle-debounce'
import { useInterval } from '@@/js/use-interval.js'
import type { InputHTMLAttributes } from 'vue'
import type { SuggestionType } from '@/utility/autocomplete.js'
import MkButton from '@/components/MkButton.vue'
import { i18n } from '@/i18n.js'
import { Autocomplete } from '@/utility/autocomplete.js'
import { genId } from '@/utility/id.js'

type SupportedTypes = 'text' | 'password' | 'email' | 'url' | 'tel' | 'number' | 'search' | 'date' | 'time' | 'datetime-local' | 'color';
type ModelValueType<T extends SupportedTypes> =
	T extends 'number' ? number :
	T extends 'text' | 'password' | 'email' | 'url' | 'tel' | 'search' | 'date' | 'time' | 'datetime-local' | 'color' ? string :
	never;

export default /*@__PURE__*/_defineComponent({
  __name: 'MkInput',
  props: {
    modelValue: { type: [Object, String], required: true },
    type: { type: null, required: false },
    required: { type: Boolean, required: false },
    readonly: { type: Boolean, required: false },
    disabled: { type: Boolean, required: false },
    pattern: { type: String, required: false },
    placeholder: { type: String, required: false },
    autofocus: { type: Boolean, required: false },
    autocomplete: { type: String, required: false },
    mfmAutocomplete: { type: [Boolean, Array], required: false },
    autocapitalize: { type: String, required: false },
    spellcheck: { type: Boolean, required: false },
    inputmode: { type: null, required: false },
    step: { type: null, required: false },
    datalist: { type: Array, required: false },
    min: { type: Number, required: false },
    max: { type: Number, required: false },
    inline: { type: Boolean, required: false },
    debounce: { type: Boolean, required: false },
    manualSave: { type: Boolean, required: false },
    small: { type: Boolean, required: false },
    large: { type: Boolean, required: false }
  },
  emits: ["change", "keydown", "enter", "update:modelValue", "savingStateChange"],
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const { modelValue } = toRefs(props);
const v = ref<ModelValueType<T> | null>(modelValue.value);
const id = genId();
const focused = ref(false);
const changed = ref(false);
const invalid = ref(false);
const filled = computed(() => v.value !== '' && v.value != null);
const inputEl = useTemplateRef('inputEl');
const prefixEl = useTemplateRef('prefixEl');
const suffixEl = useTemplateRef('suffixEl');
const height =
	props.small ? 33 :
	props.large ? 39 :
	36;
let autocompleteWorker: Autocomplete | null = null;
const focus = () => inputEl.value?.focus();
const onInput = (event: InputEvent) => {
	changed.value = true;
	emit('change', event);
};
const onKeydown = (ev: KeyboardEvent) => {
	if (ev.isComposing || ev.key === 'Process' || ev.keyCode === 229) return;

	emit('keydown', ev);

	if (ev.code === 'Enter') {
		emit('enter', ev);
	}
};
const updated = () => {
	changed.value = false;
	if (props.type === 'number') {
		emit('update:modelValue', typeof v.value === 'number' ? v.value as ModelValueType<T> : parseFloat(v.value ?? '0') as ModelValueType<T>);
	} else {
		emit('update:modelValue', v.value ?? '');
	}
};
const debouncedUpdated = debounce(1000, updated);
watch(modelValue, newValue => {
	v.value = newValue;
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
// このコンポーネントが作成された時、非表示状態である場合がある
// 非表示状態だと要素の幅などは0になってしまうので、定期的に計算する
useInterval(() => {
	if (inputEl.value == null) return;
	if (prefixEl.value) {
		if (prefixEl.value.offsetWidth) {
			inputEl.value.style.paddingLeft = prefixEl.value.offsetWidth + 'px';
		}
	}
	if (suffixEl.value) {
		if (suffixEl.value.offsetWidth) {
			inputEl.value.style.paddingRight = suffixEl.value.offsetWidth + 'px';
		}
	}
}, 100, {
	immediate: true,
	afterMounted: true,
});
onMounted(() => {
	nextTick(() => {
		if (props.autofocus) {
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
__expose({
	focus,
})

return (_ctx: any,_cache: any) => {
  const _directive_adaptive_border = _resolveDirective("adaptive-border")

  return (_openBlock(), _createElementBlock("div", { class: "_selectable" }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.label),
        onClick: focus
      }, [ _renderSlot(_ctx.$slots, "label") ]), _createElementVNode("div", {
        class: _normalizeClass([_ctx.$style.input, { [_ctx.$style.inline]: __props.inline, [_ctx.$style.disabled]: __props.disabled, [_ctx.$style.focused]: focused.value }])
      }, [ _createElementVNode("div", {
          ref_key: "prefixEl", ref: prefixEl,
          class: _normalizeClass(_ctx.$style.prefix)
        }, [ _renderSlot(_ctx.$slots, "prefix") ], 512 /* NEED_PATCH */), _withDirectives(_createElementVNode("input", {
          ref_key: "inputEl", ref: inputEl,
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((v).value = $event)),
          class: _normalizeClass(_ctx.$style.inputCore),
          type: __props.type,
          disabled: __props.disabled,
          required: __props.required,
          readonly: __props.readonly,
          placeholder: __props.placeholder,
          pattern: __props.pattern,
          autocomplete: __props.autocomplete,
          autocapitalize: __props.autocapitalize,
          spellcheck: __props.spellcheck,
          inputmode: __props.inputmode,
          step: __props.step,
          list: _unref(id),
          min: __props.min,
          max: __props.max,
          onFocus: _cache[1] || (_cache[1] = ($event: any) => (focused.value = true)),
          onBlur: _cache[2] || (_cache[2] = ($event: any) => (focused.value = false)),
          onKeydown: _cache[3] || (_cache[3] = ($event: any) => (onKeydown($event))),
          onInput: onInput
        }, null, 40 /* PROPS, NEED_HYDRATION */, ["type", "disabled", "required", "readonly", "placeholder", "pattern", "autocomplete", "autocapitalize", "spellcheck", "inputmode", "step", "list", "min", "max"]), [ [_vModelText, v.value] ]), (__props.datalist) ? (_openBlock(), _createElementBlock("datalist", {
            key: 0,
            id: _unref(id)
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.datalist, (data) => {
              return (_openBlock(), _createElementBlock("option", {
                key: data,
                value: data
              }, 8 /* PROPS */, ["value"]))
            }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), _createElementVNode("div", {
          ref_key: "suffixEl", ref: suffixEl,
          class: _normalizeClass(_ctx.$style.suffix)
        }, [ _renderSlot(_ctx.$slots, "suffix") ], 512 /* NEED_PATCH */) ], 2 /* CLASS */), _createElementVNode("div", {
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
