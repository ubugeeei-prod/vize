import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", {
  class: "ti ti-notes",
  style: "margin-right: 0.5em;"
});
import MkWindow from "@/components/MkWindow.vue";
import XRegisterLogs from "@/pages/admin/custom-emojis-manager.logs.vue";
import { i18n } from "@/i18n.js";
export default {
  __name: "custom-emojis-manager.local.list.logs",
  props: { logs: {
    type: Array,
    required: true
  } },
  emits: ["closed"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkWindow, {
        ref: "uiWindow",
        initialWidth: 400,
        initialHeight: 500,
        canResize: true,
        onClosed: _cache[0] || (_cache[0] = ($event) => emit("closed"))
      }, {
        header: _withCtx(() => [
          _hoisted_1,
          _createTextVNode(" "),
          _createTextVNode(
            _toDisplayString(_unref(i18n).ts._customEmojisManager._gridCommon.registrationLogs),
            1
            /* TEXT */
          )
        ]),
        default: _withCtx(() => [_createElementVNode("div", { class: "_spacer" }, [_createVNode(XRegisterLogs, { logs: __props.logs }, null, 8, ["logs"])])]),
        _: 1
      }, 8, [
        "initialWidth",
        "initialHeight",
        "canResize"
      ]);
    };
  }
};
