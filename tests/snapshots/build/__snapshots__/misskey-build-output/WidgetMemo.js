import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, vModelText as _vModelText } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-note" });
import { ref, watch } from "vue";
import { useWidgetPropsManager } from "./widget.js";
import MkContainer from "@/components/MkContainer.vue";
import { store } from "@/store.js";
import { i18n } from "@/i18n.js";
const name = "memo";
export default {
  __name: "WidgetMemo",
  setup(__props, { expose: __expose, emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const widgetPropsDef = {
      showHeader: {
        type: "boolean",
        label: i18n.ts._widgetOptions.showHeader,
        default: true
      },
      height: {
        type: "number",
        label: i18n.ts.height,
        default: 100
      }
    };
    const { widgetProps, configure } = useWidgetPropsManager(name, widgetPropsDef, props, emit);
    const text = ref(store.s.memo);
    const changed = ref(false);
    let timeoutId = null;
    const saveMemo = () => {
      store.set("memo", text.value);
      changed.value = false;
    };
    const onChange = () => {
      changed.value = true;
      if (timeoutId != null) window.clearTimeout(timeoutId);
      timeoutId = window.setTimeout(saveMemo, 1e3);
    };
    watch(() => store.r.memo, (newText) => {
      text.value = newText.value;
    });
    __expose({
      name,
      configure,
      id: props.widget ? props.widget.id : null
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkContainer, {
        showHeader: _unref(widgetProps).showHeader,
        "data-cy-mkw-memo": "",
        class: "mkw-memo"
      }, {
        icon: _withCtx(() => [_hoisted_1]),
        header: _withCtx(() => [_createTextVNode(
          _toDisplayString(_unref(i18n).ts._widgets.memo),
          1
          /* TEXT */
        )]),
        default: _withCtx(() => [_createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.root) },
          [_withDirectives(_createElementVNode("textarea", {
            "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => text.value = $event),
            style: _normalizeStyle(`height: ${_unref(widgetProps).height}px;`),
            class: _normalizeClass(_ctx.$style.textarea),
            placeholder: _unref(i18n).ts.memo,
            onInput: onChange
          }, null, 46, ["placeholder"]), [[_vModelText, text.value]]), _createElementVNode("button", {
            class: _normalizeClass(["_buttonPrimary", _ctx.$style.save]),
            disabled: !changed.value,
            onClick: saveMemo
          }, _toDisplayString(_unref(i18n).ts.save), 11, ["disabled"])],
          2
          /* CLASS */
        )]),
        _: 1
      }, 8, ["showHeader"]);
    };
  }
};
