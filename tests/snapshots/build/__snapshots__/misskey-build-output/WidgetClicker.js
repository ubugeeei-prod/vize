import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-cookie" });
import { useWidgetPropsManager } from "./widget.js";
import { i18n } from "@/i18n.js";
import MkContainer from "@/components/MkContainer.vue";
import MkClickerGame from "@/components/MkClickerGame.vue";
const name = "clicker";
export default {
  __name: "WidgetClicker",
  setup(__props, { expose: __expose, emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const widgetPropsDef = { showHeader: {
      type: "boolean",
      label: i18n.ts._widgetOptions.showHeader,
      default: false
    } };
    const { widgetProps, configure } = useWidgetPropsManager(name, widgetPropsDef, props, emit);
    __expose({
      name,
      configure,
      id: props.widget ? props.widget.id : null
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkContainer, {
        showHeader: _unref(widgetProps).showHeader,
        class: "mkw-clicker"
      }, {
        icon: _withCtx(() => [_hoisted_1]),
        header: _withCtx(() => [_createTextVNode("Clicker")]),
        default: _withCtx(() => [_createVNode(MkClickerGame)]),
        _: 1
      }, 8, ["showHeader"]);
    };
  }
};
