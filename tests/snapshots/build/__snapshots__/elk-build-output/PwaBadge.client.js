import {
  openBlock as _openBlock,
  createElementBlock as _createElementBlock,
  createElementVNode as _createElementVNode,
  createCommentVNode as _createCommentVNode,
  toDisplayString as _toDisplayString,
} from "vue";

const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", {
  "i-ri-download-cloud-2-line": "true",
});
const _hoisted_2 = { flex: "~ gap-2", "items-center": "true" };

export function render(_ctx, _cache) {
  return _ctx.useNuxtApp().$pwa?.needRefresh
    ? (_openBlock(),
      _createElementBlock(
        "button",
        {
          key: 0,
          bg: "primary-fade",
          relative: "",
          rounded: "",
          flex: "~ gap-1 center",
          px3: "",
          py1: "",
          "text-primary": "",
          onClick: ($event) => _ctx.useNuxtApp().$pwa?.updateServiceWorker(),
        },
        [
          _hoisted_1,
          _createElementVNode(
            "h2",
            _hoisted_2,
            _toDisplayString(_ctx.$t("pwa.update_available_short")),
            1 /* TEXT */,
          ),
        ],
      ))
    : _createCommentVNode("v-if", true);
}
