import { useModel as _useModel } from "vue";
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, createSlots as _createSlots, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
import { computed, ref, watch } from "vue";
import XFile from "@/components/MkForm.file.vue";
import MkInput from "@/components/MkInput.vue";
import MkTextarea from "@/components/MkTextarea.vue";
import MkSwitch from "@/components/MkSwitch.vue";
import MkSelect from "@/components/MkSelect.vue";
import MkRange from "@/components/MkRange.vue";
import MkButton from "@/components/MkButton.vue";
import MkRadios from "@/components/MkRadios.vue";
import { i18n } from "@/i18n.js";
export default {
  __name: "MkForm",
  props: {
    form: {
      type: null,
      required: true
    },
    "modelValue": { required: true }
  },
  emits: ["canSaveStateChange", "update:modelValue"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const values = _useModel(__props, "modelValue");
    // TODO: ジェネリックにしたい
    // 保存可能状態の管理
    const inputSavingStates = ref({});
    function onSavingStateChange(key, changed, invalid) {
      inputSavingStates.value[key] = {
        changed,
        invalid
      };
    }
    const canSave = computed(() => {
      for (const key in inputSavingStates.value) {
        const state = inputSavingStates.value[key];
        if ("manualSave" in props.form[key] && props.form[key].manualSave && state.changed || state.invalid) {
          return false;
        }
        if ("required" in props.form[key] && props.form[key].required) {
          const val = values.value[key];
          if (val === null || val === undefined || val === "") {
            return false;
          }
        }
      }
      return true;
    });
    watch(canSave, (newCanSave) => {
      emit("canSaveStateChange", newCanSave);
    }, { immediate: true });
    function getMkSelectDef(def) {
      return def.enum.map((v) => {
        if (typeof v === "string") {
          return {
            value: v,
            label: v
          };
        } else {
          return {
            value: v.value,
            label: v.label
          };
        }
      });
    }
    function getRadioOptionsDef(def) {
      return def.options.map((v) => {
        if (typeof v === "string") {
          return {
            value: v,
            label: v
          };
        } else {
          return {
            value: v.value,
            label: v.label
          };
        }
      });
    }
    return (_ctx, _cache) => {
      const _component_MkResult = _resolveComponent("MkResult");
      return Object.keys(__props.form).filter((item) => !__props.form[item].hidden).length > 0 ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        class: "_gaps_m"
      }, [(_openBlock(true), _createElementBlock(
        _Fragment,
        null,
        _renderList(__props.form, (v, k) => {
          return _openBlock(), _createElementBlock(
            _Fragment,
            null,
            [(typeof v.hidden == "function" ? v.hidden(values.value) : v.hidden) ? (_openBlock(), _createElementBlock(
              _Fragment,
              { key: 0 },
              null,
              64
              /* STABLE_FRAGMENT */
            )) : v.type === "number" ? (_openBlock(), _createBlock(MkInput, {
              key: 1,
              type: "number",
              step: v.step || 1,
              manualSave: v.manualSave,
              onSavingStateChange: (changed, invalid) => onSavingStateChange(k, changed, invalid),
              modelValue: values.value[k],
              "onUpdate:modelValue": ($event) => values.value[k] = $event
            }, _createSlots({ _: 2 }, [{
              name: "label",
              fn: _withCtx(() => [_createElementVNode("span", { textContent: _toDisplayString(v.label || k) }, null, 8, ["textContent"]), v.required === false ? (_openBlock(), _createElementBlock(
                "span",
                { key: 0 },
                " (" + _toDisplayString(_unref(i18n).ts.optional) + ")",
                1
                /* TEXT */
              )) : _createCommentVNode("v-if", true)])
            }, v.description ? {
              name: "caption",
              fn: _withCtx(() => [_createTextVNode(
                _toDisplayString(v.description),
                1
                /* TEXT */
              )]),
              key: "0"
            } : undefined]), 1032, [
              "step",
              "manualSave",
              "onSavingStateChange",
              "modelValue",
              "onUpdate:modelValue"
            ])) : v.type === "string" && !v.multiline ? (_openBlock(), _createBlock(MkInput, {
              key: 2,
              type: "text",
              mfmAutocomplete: v.treatAsMfm,
              manualSave: v.manualSave,
              onSavingStateChange: (changed, invalid) => onSavingStateChange(k, changed, invalid),
              modelValue: values.value[k],
              "onUpdate:modelValue": ($event) => values.value[k] = $event
            }, _createSlots({ _: 2 }, [{
              name: "label",
              fn: _withCtx(() => [_createElementVNode("span", { textContent: _toDisplayString(v.label || k) }, null, 8, ["textContent"]), v.required === false ? (_openBlock(), _createElementBlock(
                "span",
                { key: 0 },
                " (" + _toDisplayString(_unref(i18n).ts.optional) + ")",
                1
                /* TEXT */
              )) : _createCommentVNode("v-if", true)])
            }, v.description ? {
              name: "caption",
              fn: _withCtx(() => [_createTextVNode(
                _toDisplayString(v.description),
                1
                /* TEXT */
              )]),
              key: "0"
            } : undefined]), 1032, [
              "mfmAutocomplete",
              "manualSave",
              "onSavingStateChange",
              "modelValue",
              "onUpdate:modelValue"
            ])) : v.type === "string" && v.multiline ? (_openBlock(), _createBlock(MkTextarea, {
              key: 3,
              mfmAutocomplete: v.treatAsMfm,
              mfmPreview: v.treatAsMfm,
              manualSave: v.manualSave,
              onSavingStateChange: (changed, invalid) => onSavingStateChange(k, changed, invalid),
              modelValue: values.value[k],
              "onUpdate:modelValue": ($event) => values.value[k] = $event
            }, _createSlots({ _: 2 }, [{
              name: "label",
              fn: _withCtx(() => [_createElementVNode("span", { textContent: _toDisplayString(v.label || k) }, null, 8, ["textContent"]), v.required === false ? (_openBlock(), _createElementBlock(
                "span",
                { key: 0 },
                " (" + _toDisplayString(_unref(i18n).ts.optional) + ")",
                1
                /* TEXT */
              )) : _createCommentVNode("v-if", true)])
            }, v.description ? {
              name: "caption",
              fn: _withCtx(() => [_createTextVNode(
                _toDisplayString(v.description),
                1
                /* TEXT */
              )]),
              key: "0"
            } : undefined]), 1032, [
              "mfmAutocomplete",
              "mfmPreview",
              "manualSave",
              "onSavingStateChange",
              "modelValue",
              "onUpdate:modelValue"
            ])) : v.type === "boolean" ? (_openBlock(), _createBlock(MkSwitch, {
              key: 4,
              modelValue: values.value[k],
              "onUpdate:modelValue": ($event) => values.value[k] = $event
            }, _createSlots({ _: 2 }, [v.description ? {
              name: "caption",
              fn: _withCtx(() => [_createTextVNode(
                _toDisplayString(v.description),
                1
                /* TEXT */
              )]),
              key: "0"
            } : undefined]), 1032, ["modelValue", "onUpdate:modelValue"])) : v.type === "enum" ? (_openBlock(), _createBlock(MkSelect, {
              key: 5,
              items: getMkSelectDef(v),
              modelValue: values.value[k],
              "onUpdate:modelValue": ($event) => values.value[k] = $event
            }, {
              label: _withCtx(() => [_createElementVNode("span", { textContent: _toDisplayString(v.label || k) }, null, 8, ["textContent"]), v.required === false ? (_openBlock(), _createElementBlock(
                "span",
                { key: 0 },
                " (" + _toDisplayString(_unref(i18n).ts.optional) + ")",
                1
                /* TEXT */
              )) : _createCommentVNode("v-if", true)]),
              _: 2
            }, 8, [
              "items",
              "modelValue",
              "onUpdate:modelValue"
            ])) : v.type === "radio" ? (_openBlock(), _createBlock(MkRadios, {
              key: 6,
              options: getRadioOptionsDef(v),
              modelValue: values.value[k],
              "onUpdate:modelValue": ($event) => values.value[k] = $event
            }, {
              label: _withCtx(() => [_createElementVNode("span", { textContent: _toDisplayString(v.label || k) }, null, 8, ["textContent"]), v.required === false ? (_openBlock(), _createElementBlock(
                "span",
                { key: 0 },
                " (" + _toDisplayString(_unref(i18n).ts.optional) + ")",
                1
                /* TEXT */
              )) : _createCommentVNode("v-if", true)]),
              _: 2
            }, 8, [
              "options",
              "modelValue",
              "onUpdate:modelValue"
            ])) : v.type === "range" ? (_openBlock(), _createBlock(MkRange, {
              key: 7,
              min: v.min,
              max: v.max,
              step: v.step,
              textConverter: v.textConverter,
              modelValue: values.value[k],
              "onUpdate:modelValue": ($event) => values.value[k] = $event
            }, _createSlots({ _: 2 }, [{
              name: "label",
              fn: _withCtx(() => [_createElementVNode("span", { textContent: _toDisplayString(v.label || k) }, null, 8, ["textContent"]), v.required === false ? (_openBlock(), _createElementBlock(
                "span",
                { key: 0 },
                " (" + _toDisplayString(_unref(i18n).ts.optional) + ")",
                1
                /* TEXT */
              )) : _createCommentVNode("v-if", true)])
            }, v.description ? {
              name: "caption",
              fn: _withCtx(() => [_createTextVNode(
                _toDisplayString(v.description),
                1
                /* TEXT */
              )]),
              key: "0"
            } : undefined]), 1032, [
              "min",
              "max",
              "step",
              "textConverter",
              "modelValue",
              "onUpdate:modelValue"
            ])) : v.type === "button" ? (_openBlock(), _createBlock(MkButton, {
              key: 8,
              onClick: ($event) => v.action($event, values.value)
            }, {
              default: _withCtx(() => [_createElementVNode("span", { textContent: _toDisplayString(v.content || k) }, null, 8, ["textContent"])]),
              _: 2
            }, 8, ["onClick"])) : v.type === "drive-file" ? (_openBlock(), _createBlock(XFile, {
              key: 9,
              fileId: v.defaultFileId,
              validate: async (f) => !v.validate || await v.validate(f),
              onUpdate: (f) => values.value[k] = f
            }, null, 8, [
              "fileId",
              "validate",
              "onUpdate"
            ])) : _createCommentVNode("v-if", true)],
            64
            /* STABLE_FRAGMENT */
          );
        }),
        256
        /* UNKEYED_FRAGMENT */
      ))])) : (_openBlock(), _createBlock(_component_MkResult, {
        key: 1,
        type: "empty",
        text: _unref(i18n).ts.nothingToConfigure
      }, null, 8, ["text"]));
    };
  }
};
