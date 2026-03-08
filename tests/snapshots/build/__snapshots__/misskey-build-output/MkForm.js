import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, createSlots as _createSlots, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"

import { computed, ref, watch } from 'vue'
import XFile from '@/components/MkForm.file.vue'
import MkInput from '@/components/MkInput.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkSelect from '@/components/MkSelect.vue'
import MkRange from '@/components/MkRange.vue'
import MkButton from '@/components/MkButton.vue'
import MkRadios from '@/components/MkRadios.vue'
import { i18n } from '@/i18n.js'
import type { MkSelectItem } from '@/components/MkSelect.vue'
import type { MkRadiosOption } from '@/components/MkRadios.vue'
import type { Form, EnumFormItem, RadioFormItem } from '@/utility/form.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkForm',
  props: {
    form: { type: null, required: true },
    "modelValue": { required: true }
  },
  emits: ["canSaveStateChange", "update:modelValue"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const values = _useModel(__props, "modelValue")
// TODO: ジェネリックにしたい
// 保存可能状態の管理
const inputSavingStates = ref<Record<string, { changed: boolean; invalid: boolean }>>({});
function onSavingStateChange(key: string, changed: boolean, invalid: boolean) {
	inputSavingStates.value[key] = { changed, invalid };
}
const canSave = computed(() => {
	for (const key in inputSavingStates.value) {
		const state = inputSavingStates.value[key];
		if (
			('manualSave' in props.form[key] && props.form[key].manualSave && state.changed) ||
			state.invalid
	 	) {
			return false;
		}
		if ('required' in props.form[key] && props.form[key].required) {
			const val = values.value[key];
			if (val === null || val === undefined || val === '') {
				return false;
			}
		}
	}
	return true;
});
watch(canSave, (newCanSave) => {
	emit('canSaveStateChange', newCanSave);
}, { immediate: true });
function getMkSelectDef(def: EnumFormItem): MkSelectItem[] {
	return def.enum.map((v) => {
		if (typeof v === 'string') {
			return { value: v, label: v };
		} else {
			return { value: v.value, label: v.label };
		}
	});
}
function getRadioOptionsDef(def: RadioFormItem): MkRadiosOption[] {
	return def.options.map<MkRadiosOption>((v) => {
		if (typeof v === 'string') {
			return { value: v, label: v };
		} else {
			return { value: v.value, label: v.label };
		}
	});
}

return (_ctx: any,_cache: any) => {
  const _component_MkResult = _resolveComponent("MkResult")

  return (Object.keys(__props.form).filter(item => !__props.form[item].hidden).length > 0)
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        class: "_gaps_m"
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.form, (v, k) => {
          return (_openBlock(), _createElementBlock(_Fragment, null, [
            (typeof _ctx.v.hidden == 'function' ? _ctx.v.hidden(values.value) : _ctx.v.hidden)
              ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
              ], 64 /* STABLE_FRAGMENT */))
              : (_ctx.v.type === 'number')
                ? (_openBlock(), _createBlock(MkInput, {
                  key: 1,
                  type: "number",
                  step: _ctx.v.step || 1,
                  manualSave: _ctx.v.manualSave,
                  onSavingStateChange: _cache[0] || (_cache[0] = (changed, invalid) => onSavingStateChange(_ctx.k, changed, invalid)),
                  modelValue: values.value[_ctx.k],
                  "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((values.value[_ctx.k]) = $event))
                }, _createSlots({ _: 2 /* DYNAMIC */ }, [
                  {
                    name: "label",
                    fn: _withCtx(() => [
                      _createElementVNode("span", {
                        textContent: _toDisplayString(_ctx.v.label || _ctx.k)
                      }, null, 8 /* PROPS */, ["textContent"]),
                      (_ctx.v.required === false)
                        ? (_openBlock(), _createElementBlock("span", { key: 0 }, " (" + _toDisplayString(_unref(i18n).ts.optional) + ")", 1 /* TEXT */))
                        : _createCommentVNode("v-if", true)
                    ])
                  },
                  (_ctx.v.description)
                    ? {
                      name: "caption",
                      fn: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_ctx.v.description), 1 /* TEXT */)
                      ]),
                      key: "0"
                    }
                  : undefined
                ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["step", "manualSave", "modelValue"]))
              : (_ctx.v.type === 'string' && !_ctx.v.multiline)
                ? (_openBlock(), _createBlock(MkInput, {
                  key: 2,
                  type: "text",
                  mfmAutocomplete: _ctx.v.treatAsMfm,
                  manualSave: _ctx.v.manualSave,
                  onSavingStateChange: _cache[2] || (_cache[2] = (changed, invalid) => onSavingStateChange(_ctx.k, changed, invalid)),
                  modelValue: values.value[_ctx.k],
                  "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((values.value[_ctx.k]) = $event))
                }, _createSlots({ _: 2 /* DYNAMIC */ }, [
                  {
                    name: "label",
                    fn: _withCtx(() => [
                      _createElementVNode("span", {
                        textContent: _toDisplayString(_ctx.v.label || _ctx.k)
                      }, null, 8 /* PROPS */, ["textContent"]),
                      (_ctx.v.required === false)
                        ? (_openBlock(), _createElementBlock("span", { key: 0 }, " (" + _toDisplayString(_unref(i18n).ts.optional) + ")", 1 /* TEXT */))
                        : _createCommentVNode("v-if", true)
                    ])
                  },
                  (_ctx.v.description)
                    ? {
                      name: "caption",
                      fn: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_ctx.v.description), 1 /* TEXT */)
                      ]),
                      key: "0"
                    }
                  : undefined
                ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["mfmAutocomplete", "manualSave", "modelValue"]))
              : (_ctx.v.type === 'string' && _ctx.v.multiline)
                ? (_openBlock(), _createBlock(MkTextarea, {
                  key: 3,
                  mfmAutocomplete: _ctx.v.treatAsMfm,
                  mfmPreview: _ctx.v.treatAsMfm,
                  manualSave: _ctx.v.manualSave,
                  onSavingStateChange: _cache[4] || (_cache[4] = (changed, invalid) => onSavingStateChange(_ctx.k, changed, invalid)),
                  modelValue: values.value[_ctx.k],
                  "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((values.value[_ctx.k]) = $event))
                }, _createSlots({ _: 2 /* DYNAMIC */ }, [
                  {
                    name: "label",
                    fn: _withCtx(() => [
                      _createElementVNode("span", {
                        textContent: _toDisplayString(_ctx.v.label || _ctx.k)
                      }, null, 8 /* PROPS */, ["textContent"]),
                      (_ctx.v.required === false)
                        ? (_openBlock(), _createElementBlock("span", { key: 0 }, " (" + _toDisplayString(_unref(i18n).ts.optional) + ")", 1 /* TEXT */))
                        : _createCommentVNode("v-if", true)
                    ])
                  },
                  (_ctx.v.description)
                    ? {
                      name: "caption",
                      fn: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_ctx.v.description), 1 /* TEXT */)
                      ]),
                      key: "0"
                    }
                  : undefined
                ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["mfmAutocomplete", "mfmPreview", "manualSave", "modelValue"]))
              : (_ctx.v.type === 'boolean')
                ? (_openBlock(), _createBlock(MkSwitch, {
                  key: 4,
                  modelValue: values.value[_ctx.k],
                  "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((values.value[_ctx.k]) = $event))
                }, _createSlots({ _: 2 /* DYNAMIC */ }, [
                  (_ctx.v.description)
                    ? {
                      name: "caption",
                      fn: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_ctx.v.description), 1 /* TEXT */)
                      ]),
                      key: "0"
                    }
                  : undefined
                ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["modelValue"]))
              : (_ctx.v.type === 'enum')
                ? (_openBlock(), _createBlock(MkSelect, {
                  key: 5,
                  items: getMkSelectDef(_ctx.v),
                  modelValue: values.value[_ctx.k],
                  "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => ((values.value[_ctx.k]) = $event))
                }, {
                  label: _withCtx(() => [
                    _createElementVNode("span", {
                      textContent: _toDisplayString(_ctx.v.label || _ctx.k)
                    }, null, 8 /* PROPS */, ["textContent"]),
                    (_ctx.v.required === false)
                      ? (_openBlock(), _createElementBlock("span", { key: 0 }, " (" + _toDisplayString(_unref(i18n).ts.optional) + ")", 1 /* TEXT */))
                      : _createCommentVNode("v-if", true)
                  ]),
                  _: 2 /* DYNAMIC */
                }, 8 /* PROPS */, ["items", "modelValue"]))
              : (_ctx.v.type === 'radio')
                ? (_openBlock(), _createBlock(MkRadios, {
                  key: 6,
                  options: getRadioOptionsDef(_ctx.v),
                  modelValue: values.value[_ctx.k],
                  "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event: any) => ((values.value[_ctx.k]) = $event))
                }, {
                  label: _withCtx(() => [
                    _createElementVNode("span", {
                      textContent: _toDisplayString(_ctx.v.label || _ctx.k)
                    }, null, 8 /* PROPS */, ["textContent"]),
                    (_ctx.v.required === false)
                      ? (_openBlock(), _createElementBlock("span", { key: 0 }, " (" + _toDisplayString(_unref(i18n).ts.optional) + ")", 1 /* TEXT */))
                      : _createCommentVNode("v-if", true)
                  ]),
                  _: 2 /* DYNAMIC */
                }, 8 /* PROPS */, ["options", "modelValue"]))
              : (_ctx.v.type === 'range')
                ? (_openBlock(), _createBlock(MkRange, {
                  key: 7,
                  min: _ctx.v.min,
                  max: _ctx.v.max,
                  step: _ctx.v.step,
                  textConverter: _ctx.v.textConverter,
                  modelValue: values.value[_ctx.k],
                  "onUpdate:modelValue": _cache[9] || (_cache[9] = ($event: any) => ((values.value[_ctx.k]) = $event))
                }, _createSlots({ _: 2 /* DYNAMIC */ }, [
                  {
                    name: "label",
                    fn: _withCtx(() => [
                      _createElementVNode("span", {
                        textContent: _toDisplayString(_ctx.v.label || _ctx.k)
                      }, null, 8 /* PROPS */, ["textContent"]),
                      (_ctx.v.required === false)
                        ? (_openBlock(), _createElementBlock("span", { key: 0 }, " (" + _toDisplayString(_unref(i18n).ts.optional) + ")", 1 /* TEXT */))
                        : _createCommentVNode("v-if", true)
                    ])
                  },
                  (_ctx.v.description)
                    ? {
                      name: "caption",
                      fn: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_ctx.v.description), 1 /* TEXT */)
                      ]),
                      key: "0"
                    }
                  : undefined
                ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["min", "max", "step", "textConverter", "modelValue"]))
              : (_ctx.v.type === 'button')
                ? (_openBlock(), _createBlock(MkButton, {
                  key: 8,
                  onClick: _cache[10] || (_cache[10] = ($event: any) => (_ctx.v.action($event, values.value)))
                }, {
                  default: _withCtx(() => [
                    _createElementVNode("span", {
                      textContent: _toDisplayString(_ctx.v.content || _ctx.k)
                    }, null, 8 /* PROPS */, ["textContent"])
                  ]),
                  _: 2 /* DYNAMIC */
                }))
              : (_ctx.v.type === 'drive-file')
                ? (_openBlock(), _createBlock(XFile, {
                  key: 9,
                  fileId: _ctx.v.defaultFileId,
                  validate: async f => !_ctx.v.validate || await _ctx.v.validate(f),
                  onUpdate: _cache[11] || (_cache[11] = f => values.value[_ctx.k] = f)
                }, null, 8 /* PROPS */, ["fileId", "validate"]))
              : _createCommentVNode("v-if", true)
          ], 64 /* STABLE_FRAGMENT */))
        }), 256 /* UNKEYED_FRAGMENT */)) ]))
      : (_openBlock(), _createBlock(_component_MkResult, {
        key: 1,
        type: "empty",
        text: _unref(i18n).ts.nothingToConfigure
      }, null, 8 /* PROPS */, ["text"]))
}
}

})
