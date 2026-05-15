import {
  openBlock as _openBlock,
  createElementBlock as _createElementBlock,
  createElementVNode as _createElementVNode,
} from "vue";

const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", {
  flex: "true",
  class: "skeleton-loading-bg",
  "h-5": "true",
  "w-30": "true",
  rounded: "true",
});
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", {
  flex: "true",
  class: "skeleton-loading-bg",
  "h-4": "true",
  "w-45": "true",
  rounded: "true",
});
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("div", {
  flex: "true",
  class: "skeleton-loading-bg",
  "h-9": "true",
  "w-15": "true",
  rounded: "true",
});

export function render(_ctx, _cache) {
  return (
    _openBlock(),
    _createElementBlock(
      "div",
      {
        p4: "",
        flex: "",
        "justify-between": "",
        "gap-4": "",
      },
      [
        _createElementVNode("div", { flex: "~ col 1 gap-2" }, [_hoisted_1, _hoisted_2]),
        _createElementVNode(
          "div",
          {
            flex: "",
            "items-center": "",
          },
          [_hoisted_3],
        ),
      ],
    )
  );
}
