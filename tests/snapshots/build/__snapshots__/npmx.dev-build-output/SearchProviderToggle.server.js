import {
  openBlock as _openBlock,
  createElementBlock as _createElementBlock,
  createElementVNode as _createElementVNode,
} from "vue";

const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", {
  class: "i-lucide:settings w-4 h-4",
  "aria-hidden": "true",
});

export function render(_ctx, _cache) {
  return (
    _openBlock(),
    _createElementBlock("div", { class: "relative" }, [
      _createElementVNode(
        "div",
        { class: "flex items-center justify-center w-8 h-8 rounded-md text-fg-subtle" },
        [_hoisted_1],
      ),
    ])
  );
}
