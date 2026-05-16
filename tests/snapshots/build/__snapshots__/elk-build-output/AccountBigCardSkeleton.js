import {
  openBlock as _openBlock,
  createElementBlock as _createElementBlock,
  createElementVNode as _createElementVNode,
  createTextVNode as _createTextVNode,
} from "vue";

const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", {
  rounded: "true",
  "of-hidden": "true",
  aspect: "3.19",
  class: "flex skeleton-loading-bg",
});
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", {
  class: "flex skeleton-loading-bg",
  "w-full": "true",
  "h-full": "true",
});
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("div", {
  block: "true",
  "sm:hidden": "true",
  class: "skeleton-loading-bg",
  "h-8": "true",
  "w-30": "true",
  "rounded-full": "true",
});
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("div", {
  flex: "true",
  class: "skeleton-loading-bg",
  "h-5": "true",
  "w-20": "true",
  rounded: "true",
});
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("div", {
  flex: "true",
  class: "skeleton-loading-bg",
  "h-4": "true",
  "w-40": "true",
  rounded: "true",
});
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("div", {
  flex: "true",
  class: "skeleton-loading-bg",
  "h-4": "true",
  my3: "true",
  w: "3/5",
  rounded: "true",
});
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("div", {
  flex: "true",
  class: "skeleton-loading-bg",
  "h-4": "true",
  w: "sm:1/2 full",
  rounded: "true",
});
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("div", {
  "sm:flex": "true",
  hidden: "true",
  class: "skeleton-loading-bg",
  "h-8": "true",
  "w-30": "true",
  "rounded-full": "true",
});

export function render(_ctx, _cache) {
  return (
    _openBlock(),
    _createElementBlock("div", null, [
      _createElementVNode(
        "div",
        {
          px2: "",
          pt2: "",
        },
        [
          _hoisted_1,
          _createElementVNode(
            "div",
            {
              "px-4": "",
              "pb-4": "",
              flex: "~ col gap-2",
            },
            [
              _createElementVNode(
                "div",
                {
                  flex: "",
                  "sm:flex-row": "",
                  "flex-col": "",
                  "flex-gap-2": "",
                },
                [
                  _createElementVNode(
                    "div",
                    {
                      flex: "",
                      "items-center": "",
                      "justify-between": "",
                    },
                    [
                      _createElementVNode(
                        "div",
                        {
                          "w-17": "",
                          "h-17": "",
                          "rounded-full": "",
                          "border-4": "",
                          "border-bg-base": "",
                          "z-2": "",
                          "mt--2": "",
                          "ms--1": "",
                          "of-hidden": "",
                          "bg-base": "",
                        },
                        [_hoisted_2],
                      ),
                      _hoisted_3,
                    ],
                  ),
                  _createElementVNode(
                    "div",
                    {
                      "sm:mt-2": "",
                      flex: "~ col 1 gap-2",
                    },
                    [_hoisted_4, _hoisted_5],
                  ),
                ],
              ),
              _createTextVNode("\n        " + "\n        "),
              _hoisted_6,
              _createTextVNode("\n        " + "\n        "),
              _createElementVNode(
                "div",
                {
                  flex: "",
                  "justify-between": "",
                  "items-center": "",
                },
                [_hoisted_7, _hoisted_8],
              ),
            ],
          ),
        ],
      ),
    ])
  );
}
