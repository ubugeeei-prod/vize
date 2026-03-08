import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode } from "vue"

const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "w-12": "true", "h-12": "true", "rounded-full": "true", class: "skeleton-loading-bg" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { flex: "true", class: "skeleton-loading-bg", "h-5": "true", "w-20": "true", rounded: "true" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("div", { flex: "true", class: "skeleton-loading-bg", "h-4": "true", "w-full": "true", rounded: "true" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("div", { flex: "true", class: "skeleton-loading-bg", "h-4": "true", w: "4/5", rounded: "true" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("div", { flex: "true", class: "skeleton-loading-bg", "h-4": "true", w: "2/5", rounded: "true" })

export function render(_ctx, _cache) {
  return (_openBlock(), _createElementBlock("div", {
    flex: "",
    "flex-col": "",
    "gap-2": "",
    "px-4": "",
    "py-3": ""
  }, [
    _createElementVNode("div", {
      flex: "",
      "gap-4": ""
    }, [
      _createElementVNode("div", null, [
        _hoisted_1
      ]),
      _createElementVNode("div", {
        flex: "~ col 1 gap-2",
        pb2: "",
        "min-w-0": ""
      }, [
        _hoisted_2,
        _hoisted_3,
        _hoisted_4,
        _hoisted_5
      ])
    ])
  ]))
}
