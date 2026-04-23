import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("img", {
  src: "/client-assets/drop-and-fusion/logo.png",
  style: "display: block; max-width: 100%; max-height: 200px; margin: auto;"
});
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("img", {
  src: "/client-assets/reversi/logo.png",
  style: "display: block; max-width: 100%; max-height: 200px; margin: auto;"
});
import { definePage } from "@/page.js";
export default {
  __name: "games",
  setup(__props) {
    definePage(() => ({
      title: "Misskey Games",
      icon: "ti ti-device-gamepad"
    }));
    return (_ctx, _cache) => {
      const _component_MkA = _resolveComponent("MkA");
      const _component_PageWithHeader = _resolveComponent("PageWithHeader");
      return _openBlock(), _createBlock(_component_PageWithHeader, null, {
        default: _withCtx(() => [_createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 800px;"
        }, [_createElementVNode("div", { class: "_gaps" }, [_createElementVNode(
          "div",
          { class: _normalizeClass(["_panel", _ctx.$style.link]) },
          [_createVNode(_component_MkA, { to: "/bubble-game" }, {
            default: _withCtx(() => [_hoisted_1]),
            _: 1
          })],
          2
          /* CLASS */
        ), _createElementVNode(
          "div",
          { class: _normalizeClass(["_panel", _ctx.$style.link]) },
          [_createVNode(_component_MkA, { to: "/reversi" }, {
            default: _withCtx(() => [_hoisted_2]),
            _: 1
          })],
          2
          /* CLASS */
        )])])]),
        _: 1
      });
    };
  }
};
