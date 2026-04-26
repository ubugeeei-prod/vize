import { useModel as _useModel } from "vue";
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderList as _renderList, createSlots as _createSlots, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
import MkInput from "@/components/MkInput.vue";
import MkRadios from "@/components/MkRadios.vue";
import MkSwitch from "@/components/MkSwitch.vue";
import MkRange from "@/components/MkRange.vue";
import { i18n } from "@/i18n.js";
export default {
  __name: "MkImageEffectorFxForm",
  props: {
    paramDefs: {
      type: null,
      required: true
    },
    "modelValue": { required: true }
  },
  emits: ["update:modelValue"],
  setup(__props) {
    const params = _useModel(__props, "modelValue");
    function getHex(c) {
      return `#${c.map((x) => Math.round(x * 255).toString(16).padStart(2, "0")).join("")}`;
    }
    function getRgb(hex) {
      if (typeof hex === "number" || typeof hex !== "string" || !/^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})$/.test(hex)) {
        return null;
      }
      const m = hex.slice(1).match(/[0-9a-fA-F]{2}/g);
      if (m == null) return [
        0,
        0,
        0
      ];
      return m.map((x) => parseInt(x, 16) / 255);
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock("div", { class: "_gaps" }, [(_openBlock(true), _createElementBlock(
        _Fragment,
        null,
        _renderList(__props.paramDefs, (v, k) => {
          return _openBlock(), _createElementBlock("div", { key: k }, [v.type === "boolean" ? (_openBlock(), _createBlock(MkSwitch, {
            key: 0,
            modelValue: params.value[k],
            "onUpdate:modelValue": ($event) => params.value[k] = $event
          }, _createSlots({ _: 2 }, [{
            name: "label",
            fn: _withCtx(() => [_createTextVNode(
              _toDisplayString(v.label ?? k),
              1
              /* TEXT */
            )])
          }, v.caption != null ? {
            name: "caption",
            fn: _withCtx(() => [_createTextVNode(
              _toDisplayString(v.caption),
              1
              /* TEXT */
            )]),
            key: "0"
          } : undefined]), 1032, ["modelValue", "onUpdate:modelValue"])) : v.type === "number" ? (_openBlock(), _createBlock(MkRange, {
            key: 1,
            continuousUpdate: "",
            min: v.min,
            max: v.max,
            step: v.step,
            textConverter: v.toViewValue,
            onThumbDoubleClicked: () => {
              params.value[k] = v.default;
            },
            modelValue: params.value[k],
            "onUpdate:modelValue": ($event) => params.value[k] = $event
          }, _createSlots({ _: 2 }, [{
            name: "label",
            fn: _withCtx(() => [_createTextVNode(
              _toDisplayString(v.label ?? k),
              1
              /* TEXT */
            )])
          }, v.caption != null ? {
            name: "caption",
            fn: _withCtx(() => [_createTextVNode(
              _toDisplayString(v.caption),
              1
              /* TEXT */
            )]),
            key: "0"
          } : undefined]), 1032, [
            "min",
            "max",
            "step",
            "textConverter",
            "onThumbDoubleClicked",
            "modelValue",
            "onUpdate:modelValue"
          ])) : v.type === "number:enum" ? (_openBlock(), _createBlock(MkRadios, {
            key: 2,
            options: v.enum,
            modelValue: params.value[k],
            "onUpdate:modelValue": ($event) => params.value[k] = $event
          }, _createSlots({ _: 2 }, [{
            name: "label",
            fn: _withCtx(() => [_createTextVNode(
              _toDisplayString(v.label ?? k),
              1
              /* TEXT */
            )])
          }, v.caption != null ? {
            name: "caption",
            fn: _withCtx(() => [_createTextVNode(
              _toDisplayString(v.caption),
              1
              /* TEXT */
            )]),
            key: "0"
          } : undefined]), 1032, [
            "options",
            "modelValue",
            "onUpdate:modelValue"
          ])) : v.type === "seed" ? (_openBlock(), _createElementBlock("div", { key: 3 }, [_createVNode(MkRange, {
            continuousUpdate: "",
            type: "number",
            min: 0,
            max: 1e4,
            step: 1,
            modelValue: params.value[k],
            "onUpdate:modelValue": ($event) => params.value[k] = $event
          }, _createSlots({ _: 2 }, [{
            name: "label",
            fn: _withCtx(() => [_createTextVNode(
              _toDisplayString(v.label ?? k),
              1
              /* TEXT */
            )])
          }, v.caption != null ? {
            name: "caption",
            fn: _withCtx(() => [_createTextVNode(
              _toDisplayString(v.caption),
              1
              /* TEXT */
            )]),
            key: "0"
          } : undefined]), 1032, [
            "min",
            "max",
            "step",
            "modelValue",
            "onUpdate:modelValue"
          ])])) : v.type === "color" ? (_openBlock(), _createBlock(MkInput, {
            key: 4,
            modelValue: getHex(params.value[k]),
            type: "color",
            "onUpdate:modelValue": (v) => {
              const c = getRgb(v);
              if (_ctx.c != null) params.value[k] = _ctx.c;
            }
          }, _createSlots({ _: 2 }, [{
            name: "label",
            fn: _withCtx(() => [_createTextVNode(
              _toDisplayString(v.label ?? k),
              1
              /* TEXT */
            )])
          }, v.caption != null ? {
            name: "caption",
            fn: _withCtx(() => [_createTextVNode(
              _toDisplayString(v.caption),
              1
              /* TEXT */
            )]),
            key: "0"
          } : undefined]), 1032, ["modelValue", "onUpdate:modelValue"])) : _createCommentVNode("v-if", true)]);
        }),
        128
        /* KEYED_FRAGMENT */
      )), Object.keys(__props.paramDefs).length === 0 ? (_openBlock(), _createElementBlock(
        "div",
        {
          key: 0,
          class: _normalizeClass(_ctx.$style.nothingToConfigure)
        },
        _toDisplayString(_unref(i18n).ts.nothingToConfigure),
        3
        /* TEXT, CLASS */
      )) : _createCommentVNode("v-if", true)]);
    };
  }
};
