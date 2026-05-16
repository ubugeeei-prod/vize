import {
  openBlock as _openBlock,
  createElementBlock as _createElementBlock,
  createElementVNode as _createElementVNode,
  toDisplayString as _toDisplayString,
} from "vue";

const _hoisted_1 = { class: "font-mono text-sm text-fg-muted" };
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", {
  class: "i-lucide:chevron-down w-3 h-3 text-fg-muted",
  "aria-hidden": "true",
});

export function render(_ctx, _cache) {
  return (
    _openBlock(),
    _createElementBlock("div", { class: "relative flex min-w-28 justify-end" }, [
      _createElementVNode(
        "div",
        {
          class:
            "inline-flex gap-x-1 items-center justify-center font-mono border border-border rounded-md text-sm px-4 py-2 bg-transparent text-fg border-none",
        },
        [
          _createElementVNode(
            "span",
            _hoisted_1,
            _toDisplayString(_ctx.$t("account_menu.connect")),
            1 /* TEXT */,
          ),
          _hoisted_2,
        ],
      ),
    ])
  );
}
