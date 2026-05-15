import {
  openBlock as _openBlock,
  createElementBlock as _createElementBlock,
  createElementVNode as _createElementVNode,
  createCommentVNode as _createCommentVNode,
  toDisplayString as _toDisplayString,
  mergeProps as _mergeProps,
} from "vue";

const _hoisted_1 = { flex: "~ gap-2", "items-center": "true" };
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", {
  "i-material-symbols:install-desktop-rounded": "true",
  absolute: "true",
  "text-6em": "true",
  "bottom--2": "true",
  "inset-ie--2": "true",
  "text-primary": "true",
  "dark:text-white": "true",
  op10: "true",
  class: "-z-1 rtl-flip",
});

export function render(_ctx, _cache) {
  return _ctx.useNuxtApp().$pwa?.showInstallPrompt && !_ctx.useNuxtApp().$pwa?.needRefresh
    ? (_openBlock(),
      _createElementBlock(
        "div",
        _mergeProps(_ctx.$attrs, {
          key: 0,
          "m-2": "",
          p5: "",
          bg: "primary-fade",
          relative: "",
          "rounded-lg": "",
          "of-hidden": "",
          flex: "~ col gap-3",
        }),
        [
          _createElementVNode(
            "h2",
            _hoisted_1,
            _toDisplayString(_ctx.$t("pwa.install_title")),
            1 /* TEXT */,
          ),
          _createElementVNode("div", { flex: "~ gap-1" }, [
            _createElementVNode(
              "button",
              {
                type: "button",
                "btn-solid": "",
                "px-4": "",
                "py-1": "",
                "text-center": "",
                "text-sm": "",
                onClick: ($event) => _ctx.useNuxtApp().$pwa?.install(),
              },
              _toDisplayString(_ctx.$t("pwa.install")),
              9 /* TEXT, PROPS */,
              ["onClick"],
            ),
            _createElementVNode(
              "button",
              {
                type: "button",
                "btn-text": "",
                "filter-saturate-0": "",
                "px-4": "",
                "py-1": "",
                "text-center": "",
                "text-sm": "",
                onClick: ($event) => _ctx.useNuxtApp().$pwa?.cancelInstall(),
              },
              _toDisplayString(_ctx.$t("pwa.dismiss")),
              9 /* TEXT, PROPS */,
              ["onClick"],
            ),
          ]),
          _hoisted_2,
        ],
      ))
    : _createCommentVNode("v-if", true);
}
